use crate::errors::InternalIrcError;
use dashmap::DashMap;
use std::sync::Arc;

use crate::{
    channels_models::{ChannelName, IrcChannel},
    users::{User, UserState},
};

#[derive(Clone, Debug)]
pub struct ServerState {
    pub channels: Arc<DashMap<String, IrcChannel>>,
    pub users: Arc<DashMap<usize, UserState>>,
    // pub registering_users: Arc<DashSet<>>
}
impl ServerState {
    pub fn new() -> Self {
        ServerState {
            channels: Arc::new(DashMap::<ChannelName, IrcChannel>::new()),
            users: Arc::new(DashMap::<usize, UserState>::new()),
        }
    }

    pub async fn add_connecting_user(
        &self,
        user_state: &UserState,
    ) -> Result<usize, InternalIrcError> {
        let user_id = user_state.get_user_id().await;

        match user_id {
            Some(id) => {
                self.users.insert(id, user_state.clone());
                Ok(id)
            }
            None => Err(InternalIrcError::ServerStateError(
                "Failed to generate user ID",
            )),
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
