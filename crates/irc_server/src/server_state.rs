use crate::errors::InternalIrcError;
use dashmap::DashMap;
use std::sync::Arc;

use crate::{
    channels_models::{ChannelName, IrcChannel},
    user_state::UserState,
};

#[derive(Clone, Debug)]
pub struct ServerState {
    pub channels: DashMap<String, IrcChannel>,
    pub users: DashMap<usize, UserState>,
    // pub registering_users: Arc<DashSet<>>
}
impl ServerState {
    pub fn new() -> Self {
        ServerState {
            channels: DashMap::<ChannelName, IrcChannel>::new(),
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

    pub fn get_or_create_channel(&self, channel_name: String) -> IrcChannel {
        if let Some(channel) = self.channels.get(&channel_name) {
            channel.clone()
        } else {
            let channel = IrcChannel::new(channel_name.clone());
            self.channels.insert(channel_name, channel.clone());
            channel
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
