use std::sync::Arc;

use dashmap::DashMap;

use crate::{
    channels_models::{ChannelName, IrcChannel},
    users::{User, UserId},
};

#[derive(Clone, Debug)]
pub struct ServerState {
    pub channels: Arc<DashMap<String, IrcChannel>>,
    pub users: Arc<DashMap<UserId, User>>,
    // pub registering_users: Arc<DashSet<>>
}
impl ServerState {
    pub fn new() -> Self {
        ServerState {
            channels: Arc::new(DashMap::<ChannelName, IrcChannel>::new()),
            users: Arc::new(DashMap::<UserId, User>::new()),
        }
    }
}
