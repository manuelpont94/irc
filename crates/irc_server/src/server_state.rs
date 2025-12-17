use crate::{
    channels_models::{ChannelMessage, IrcChannelOperationStatus, SubscriptionControl},
    errors::InternalIrcError,
    replies::IrcReply,
};
use dashmap::DashMap;
use std::sync::Arc;

use crate::{
    channels_models::{ChannelName, IrcChannel},
    user_state::UserState,
};

#[derive(Clone, Debug)]
pub struct ServerState {
    pub channels: DashMap<String, Arc<IrcChannel>>,
    pub users: DashMap<usize, UserState>,
    // pub registering_users: Arc<DashSet<>>
}
impl ServerState {
    pub fn new() -> Self {
        ServerState {
            channels: DashMap::<ChannelName, Arc<IrcChannel>>::new(),
            users: DashMap::<usize, UserState>::new(),
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

    fn get_or_create_channel(&self, channel_name: &str) -> Arc<IrcChannel> {
        if let Some(channel) = self.channels.get(channel_name) {
            return channel.clone();
        }
        self.channels
            .entry(channel_name.to_owned())
            .or_insert_with(|| Arc::new(IrcChannel::new(channel_name.to_owned())))
            .clone()
    }

    pub async fn handle_join(
        &self,
        channel_name: String,
        client_id: usize,
        key: Option<String>,
        is_invited: bool,
    ) -> Result<(IrcChannelOperationStatus, Option<Arc<IrcChannel>>), InternalIrcError> {
        let channel = self.get_or_create_channel(&channel_name);
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

        let user_caracs = {
            let user = self
                .users
                .get(&client_id)
                .ok_or(InternalIrcError::ServerStateError("User not found"))?;
            user.get_caracs().await
        };

        Ok((IrcChannelOperationStatus::NewJoin, Some(channel)))
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
