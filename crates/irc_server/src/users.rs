use core::net::SocketAddr;
use log::error;
use nom::combinator::Opt;
use std::sync::Arc;
use tokio::sync::RwLock;

use dashmap::{DashMap, DashSet};
use tokio::sync::Mutex;

const MODE_WALLOPS: u8 = 0b0000_0100; // Bit 2 = mode 'w' (wallops)
const MODE_INVISIBLE: u8 = 0b0000_1000; // Bit 3 = mode 'i' (invisible)

pub type UserId = u64;
type Nick = String;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct User {
    id: UserId,
    nick: Nick,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub nick: Option<String>,
    pub user: Option<String>,
    pub mode: Vec<char>,
    pub full_user_name: Option<String>,
    pub registered: bool,
    pub addr: SocketAddr,
}

impl Client {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            nick: None,
            user: None,
            mode: Vec::new(),
            full_user_name: None,
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
    }

    pub async fn with_user(&self, user: String, full_user_name: String, mode: u8) {
        let mut user_data = self.0.write().await;
        user_data.user = Some(user);
        user_data.full_user_name = Some(full_user_name);
        user_data.mode = UserState::parse_basic_user_mode(mode);
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

    fn parse_basic_user_mode(mode: u8) -> Vec<char> {
        let mut res: Vec<char> = Vec::new();
        if (mode & MODE_WALLOPS) != 0 {
            res.push('w');
        }
        if (mode & MODE_INVISIBLE) != 0 {
            res.push('i');
        }
        res
    }

    pub async fn get_caracs(&self) -> Client {
        let user_data = self.0.read().await;
        user_data.clone()
    }
}
