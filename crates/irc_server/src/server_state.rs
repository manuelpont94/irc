use crate::{
    channels_models::{ChannelMessage, IrcChannelOperationStatus},
    errors::InternalIrcError,
};
use dashmap::DashMap;
use log::{debug, info};
use std::{collections::HashSet, sync::Arc};

use crate::{
    channels_models::{ChannelName, IrcChannel},
    user_state::UserState,
};

#[derive(Clone, Debug)]
pub struct ServerState {
    pub channels: Arc<DashMap<String, Arc<IrcChannel>>>,
    pub users: Arc<DashMap<usize, UserState>>,
}
impl ServerState {
    pub fn new() -> Self {
        ServerState {
            channels: Arc::new(DashMap::new()),
            users: Arc::new(DashMap::new()),
        }
    }

    pub async fn add_connecting_user(
        &self,
        user_state: &UserState,
    ) -> Result<usize, InternalIrcError> {
        let user_id = user_state.get_user_id().await;
        self.users.insert(user_id, user_state.clone());
        Ok(user_id)
    }

    pub fn channels_exists(&self, channel_name: &str) -> bool {
        self.channels.contains_key(channel_name)
    }

    fn get_or_create_channel(&self, channel_name: &str) -> (Arc<IrcChannel>, bool) {
        let is_new = !self.channels.contains_key(channel_name);
        let channel = self
            .channels
            .entry(channel_name.to_owned())
            .or_insert_with(|| Arc::new(IrcChannel::new(channel_name.to_owned())))
            .clone();

        if is_new {
            debug!(
                "new channel: {} (ptr: {:p}, tx_ptr: {:p})",
                channel_name,
                Arc::as_ptr(&channel),
                &channel.tx
            );
        } else {
            debug!(
                "existing channel: {} (ptr: {:p}, tx_ptr: {:p})",
                channel_name,
                Arc::as_ptr(&channel),
                &channel.tx
            );
        }
        (channel, is_new)
    }

    pub async fn handle_join(
        &self,
        channel_name: String,
        client_id: usize,
        key: Option<String>,
        is_invited: bool,
    ) -> Result<(IrcChannelOperationStatus, Option<Arc<IrcChannel>>), InternalIrcError> {
        let (channel, is_new_channel) = self.get_or_create_channel(&channel_name);
        {
            let modes = channel.modes.read().await;
            if modes.user_limit.is_some() && channel.members.len() >= modes.user_limit.unwrap() {
                return Ok((IrcChannelOperationStatus::ChannelIsFull, None));
            }
            if modes.ban_list.contains(&client_id) && !modes.except_list.contains(&client_id) {
                return Ok((IrcChannelOperationStatus::BannedFromChan, None));
            }
            if modes.invite_only && !is_invited {
                return Ok((IrcChannelOperationStatus::InviteOnlyChan, None));
            }
            if modes.key.is_some() && (modes.key != key) {
                return Ok((IrcChannelOperationStatus::BadChannelKey, None));
            }
        }
        if !channel.add_member(client_id) {
            // User is already in the channel, do nothing
            return Ok((IrcChannelOperationStatus::AlreadyMember, None));
        }
        if is_new_channel {
            channel.add_operator(client_id);
        }
        Ok((IrcChannelOperationStatus::NewJoin, Some(channel)))
    }

    pub async fn handle_quit(&self, client_id: usize, reason: Option<String>) {
        let quit_reason = reason.unwrap_or_else(|| "Client Quit".to_string());

        // 1. Get user details before they are gone
        if let Some((_, user_state)) = self.users.remove(&client_id) {
            let caracs = user_state.get_caracs().await;
            let quit_msg = format!(
                ":{}!{}@{:?} QUIT :{}",
                caracs.nick.unwrap(),
                caracs.user.unwrap(),
                caracs.addr,
                quit_reason
            );
            let quit_channel_message = ChannelMessage::new(quit_msg);

            // 2. Identify all unique neighbors (people who share channels)
            let mut neighbors = HashSet::new();

            // iterate through channels user was in
            for channel_name in caracs.member_of.iter() {
                if let Some(channel) = self.channels.get(channel_name) {
                    // Add all members of this channel to our notification list
                    for member_id in channel.members.iter() {
                        if *member_id != client_id {
                            if neighbors.insert(*member_id) {
                                channel.broadcast_message(quit_channel_message.clone());
                            }
                        }
                    }
                    // Remove the user from the channel
                    channel.remove_member(client_id);
                }
            }
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
