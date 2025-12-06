use core::net::SocketAddr;
use log::error;
use std::sync::Arc;
use tokio::sync::RwLock;

use dashmap::{DashMap, DashSet};
use tokio::sync::Mutex;

pub type UserId = u64;
type Nick = String;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct User {
    id: UserId,
    nick: Nick,
}

#[derive(Debug, Clone)]
pub struct Client {
    nick: Option<String>,
    user: Option<String>,
    registered: bool,
    addr: SocketAddr,
}

impl Client {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            nick: None,
            user: None,
            registered: false,
            addr,
        }
    }
}

pub struct UserState(Arc<RwLock<Client>>);
impl UserState {
    pub fn new(addr: SocketAddr) -> Self {
        UserState(Arc::new(RwLock::new(Client::new(addr))))
    }

    pub async fn with_nick(&self, nick: String) {
        let mut client = self.0.write().await;
        client.nick = Some(nick);
        _ = self.is_registered();
    }

    pub async fn with_user(&self, user: String) {
        let mut user_data = self.0.write().await;
        user_data.user = Some(user);
        _ = self.is_registered();
    }

    pub async fn is_registered(&self) -> bool {
        let mut user_data = self.0.write().await;
        if user_data.registered {
            true
        } else if user_data.nick.is_none() || user_data.user.is_none() {
            false
        } else {
            user_data.registered = true;
            true
        }
    }
}
