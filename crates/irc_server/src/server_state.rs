use crate::{
    channels_models::{IrcChannel, IrcChannelOperationStatus},
    errors::InternalIrcError,
    message_models::{BroadcastIrcMessage, DirectIrcMessage},
    types::{ChannelName, ClientId, Nickname},
    user_state::UserState,
};
use dashmap::DashMap;
use log::{debug, info};
use std::{collections::HashSet, net::IpAddr, sync::Arc};

#[derive(Clone, Debug)]
pub struct ServerState {
    pub channels: Arc<DashMap<ChannelName, Arc<IrcChannel>>>,
    pub ip_counts: Arc<DashMap<IpAddr, usize>>,
    pub nick: Arc<DashMap<Nickname, ClientId>>,
    // pub nick_user_host_server: Arc<DashMap<(String, String, String, String), ClientId>>,
    pub users: Arc<DashMap<ClientId, UserState>>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            channels: Arc::new(DashMap::new()),
            ip_counts: Arc::new(DashMap::new()),
            nick: Arc::new(DashMap::new()),
            // nick_user_host_server: Arc::new(DashMap::new()),
            users: Arc::new(DashMap::new()),
        }
    }

    pub async fn add_connecting_user(
        &self,
        user_state: &UserState,
    ) -> Result<ClientId, InternalIrcError> {
        let user_data = user_state.user.read().await;
        let user_id = user_data.user_id;
        if let Some(nick) = user_data.nick.clone() {
            self.nick.insert(nick, user_id);
        }
        self.users.insert(user_id, user_state.clone());
        Ok(user_id)
    }

    pub fn handle_nick_change(
        &self,
        client_id: ClientId,
        new_nick: &Nickname,
        old_nick: &Nickname,
    ) {
        // 3. Update the global Nick -> ClientId map
        self.nick.remove(old_nick);
        self.nick.insert(new_nick.clone(), client_id);
    }

    pub fn channels_exists(&self, channel_name: &ChannelName) -> bool {
        self.channels.contains_key(channel_name)
    }

    pub fn get_cliend_id_from_nick(&self, nick: &Nickname) -> Option<ClientId> {
        if let Some(client_ref) = self.nick.get(nick) {
            Some(*client_ref)
        } else {
            None
        }
    }

    pub fn get_user_state_from_client_id(&self, client_id: &ClientId) -> Option<UserState> {
        if let Some(client_ref) = self.users.get(client_id) {
            Some((*client_ref).clone())
        } else {
            None
        }
    }

    pub fn get_user_state_from_nick(&self, nick: &Nickname) -> Option<UserState> {
        let client_id_opt = self.get_cliend_id_from_nick(nick).map(|r| r.clone());
        if let Some(client_id) = client_id_opt {
            let client_ref_opt = self.users.get(&client_id).map(|r| r.clone());
            client_ref_opt
        } else {
            None
        }
    }

    pub fn get_channel(&self, channel: &ChannelName) -> Option<Arc<IrcChannel>> {
        self.channels.get(channel).map(|r| r.clone())
    }

    fn get_or_create_channel(&self, channel_name: &ChannelName) -> (Arc<IrcChannel>, bool) {
        let mut is_new = false;
        let channel = self
            .channels
            .entry(channel_name.clone())
            .or_insert_with(|| {
                is_new = true;
                Arc::new(IrcChannel::new(channel_name.clone()))
            })
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
        channel_name: ChannelName,
        client_id: ClientId,
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
            if modes.invite_only && !is_invited && !modes.invite_exceptions.contains(&client_id) {
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

    pub async fn quit_channel(&self, client_id: &ClientId, channel_name: &ChannelName) {
        let channel_opt = self.get_channel(channel_name);
        if let Some(channel) = channel_opt {
            channel.remove_member(&client_id);
            if channel.members.is_empty() {
                info!("Channel {channel_name} is empty, destroying.");
                self.channels.remove(channel_name);
            }
        }
    }

    pub async fn handle_quit(&self, client_id: ClientId, reason: Option<String>) {
        let quit_reason = reason.unwrap_or_else(|| "Client Quit".to_string());

        if let Some((_, user_state)) = self.users.remove(&client_id) {
            let caracs = user_state.get_caracs().await;
            let quit_msg = format!(
                ":{}!{}@{:?} QUIT :{}",
                caracs.nick.unwrap(),
                caracs.user.unwrap(),
                caracs.addr,
                quit_reason
            );
            let quit_channel_message = DirectIrcMessage::new(quit_msg);
            self.broadcast_to_neighbors(&caracs.member_of, quit_channel_message, Some(client_id))
                .await;
            for channel_name in caracs.member_of.iter() {
                let channel_opt = self.channels.get(channel_name).map(|r| Arc::clone(&r));
                if let Some(channel) = channel_opt {
                    channel.remove_member(&client_id);
                    if channel.members.is_empty() {
                        info!("Channel {channel_name} is empty, destroying.");
                        self.channels.remove(channel_name);
                    }
                }
            }
        }
    }

    async fn get_unique_neighboors(
        &self,
        channel_names: &HashSet<ChannelName>,
        exclude_id: Option<ClientId>, //
    ) -> HashSet<ClientId> {
        let mut unique_neighbors = HashSet::new();
        for name in channel_names {
            let channel_opt = self.channels.get(name).map(|r| Arc::clone(&r));
            if let Some(channel) = channel_opt {
                for member_id in channel.members.iter() {
                    let id = *member_id;
                    if Some(id) != exclude_id {
                        unique_neighbors.insert(id);
                    }
                }
            }
        }
        unique_neighbors
    }

    pub async fn broadcast_to_neighbors(
        &self,
        channel_names: &HashSet<ChannelName>,
        message: DirectIrcMessage,
        exclude_id: Option<ClientId>, // Usually the person changing NICK
    ) {
        let unique_neighbors = self.get_unique_neighboors(channel_names, exclude_id).await;
        for client_id in unique_neighbors {
            let user_opt = self.users.get(&client_id).map(|r| r.clone());
            if let Some(user_state) = user_opt {
                let _ = user_state.tx_outbound.send(message.clone()).await;
            }
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
