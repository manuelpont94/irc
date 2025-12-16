use crate::{
    channels_models::{IrcChannelOperationStatus, SubscriptionControl},
    errors::InternalIrcError,
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
    ) -> Result<IrcChannelOperationStatus, InternalIrcError> {
        // 1. Get or Create the channel (using the Arc approach)
        let channel = self.get_or_create_channel(&channel_name);

        {
            let modes = channel.modes.read().await;
            // if modes.limit.is_some() && channel.members.len() >= modes.limit.unwrap() as usize {
            //     return Err("ERR_CHANNELISFULL".to_string());
            // }
            // // ... check bans, keys ...
        }

        if !channel.add_member(client_id) {
            // User is already in the channel, do nothing
            return Ok(IrcChannelOperationStatus::AlreadyMember);
        }

        let user_caracs = {
            let user = self
                .users
                .get(&client_id)
                .ok_or(InternalIrcError::ServerStateError("User not found"))?;
            user.get_caracs().await
        };

        // broadcast subscription
        if let Some(user) = self.users.get(&client_id) {
            let rx = channel.subscribe();
            let _ = user
                .tx_control
                .send(SubscriptionControl::Subscribe {
                    channel_name: channel_name.clone(),
                    receiver: rx,
                })
                .await;
        }
        Ok(IrcChannelOperationStatus::Ok)
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
