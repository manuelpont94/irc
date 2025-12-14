use core::net::SocketAddr;
use std::{
    collections::HashSet,
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::RwLock;

use crate::errors::InternalIrcError;
use crate::replies::IrcReply;

const MODE_WALLOPS: u8 = 0b0000_0100; // Bit 2 = mode 'w' (wallops)
const MODE_INVISIBLE: u8 = 0b0000_1000; // Bit 3 = mode 'i' (invisible)

use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

fn get_next_user_id() -> usize {
    NEXT_USER_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug)]
pub struct User {
    pub user_id: Option<usize>,
    pub nick: Option<String>,
    pub user: Option<String>,
    pub modes: HashSet<char>,
    pub full_user_name: Option<String>,
    pub registered: AtomicBool,
    pub addr: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct UserSnapshot {
    pub user_id: Option<usize>,
    pub nick: Option<String>,
    pub user: Option<String>,
    pub modes: HashSet<char>,
    pub full_user_name: Option<String>,
    pub registered: bool,
    pub addr: SocketAddr,
}

impl User {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            user_id: None,
            nick: None,
            user: None,
            modes: HashSet::new(),
            full_user_name: None,
            registered: AtomicBool::new(false),
            addr,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserState(Arc<RwLock<User>>);
impl UserState {
    pub fn new(addr: SocketAddr) -> Self {
        UserState(Arc::new(RwLock::new(User::new(addr))))
    }

    pub async fn with_nick(&self, nick: String) {
        let mut client = self.0.write().await;
        client.nick = Some(nick);
    }

    pub async fn with_user(&self, user: String, full_user_name: String, mode: u8) {
        let mut user_data = self.0.write().await;
        user_data.user = Some(user);
        user_data.full_user_name = Some(full_user_name);
        user_data.modes = UserState::parse_basic_user_mode(mode);
    }

    pub async fn is_registered(&self) -> bool {
        // first check under read lock
        // 🚀 fast path: atomic read
        if self.0.read().await.registered.load(Ordering::Acquire) {
            return true;
        }

        // slow path: need to check nick/user and maybe register
        let mut user_data = self.0.write().await;

        // double-check under write lock
        if user_data.registered.load(Ordering::Relaxed) {
            return true;
        }

        if user_data.nick.is_none() || user_data.user.is_none() {
            return false;
        }

        // 👇 first and only registration
        user_data.user_id = Some(get_next_user_id());
        user_data.registered.store(true, Ordering::Release);

        true
    }

    pub async fn get_user_id(&self) -> Option<usize> {
        if self.is_registered().await {
            let user_data = self.0.read().await;
            user_data.user_id
        } else {
            None
        }
    }

    fn parse_basic_user_mode(mode: u8) -> HashSet<char> {
        let mut res: HashSet<char> = HashSet::new();
        if (mode & MODE_WALLOPS) != 0 {
            res.insert('w');
        }
        if (mode & MODE_INVISIBLE) != 0 {
            res.insert('i');
        }
        res
    }

    pub async fn get_caracs(&self) -> UserSnapshot {
        let user_data = self.0.read().await;
        UserSnapshot {
            user_id: user_data.user_id,
            nick: user_data.nick.clone(),
            user: user_data.user.clone(),
            modes: user_data.modes.clone(),
            full_user_name: user_data.full_user_name.clone(),
            registered: user_data.registered.load(Ordering::Acquire),
            addr: user_data.addr,
        }
    }

    pub async fn with_modes<'a>(
        &self,
        nick: &'a str,
        modes: Vec<(char, Vec<char>)>,
    ) -> Result<Option<IrcReply<'a>>, InternalIrcError> {
        // known_modes :
        // a - user is flagged as away;
        // i - marks a users as invisible;
        // w - user receives wallops;
        // r - restricted user connection;
        // o - operator flag;
        // O - local operator flag;
        // s - marks a user for receipt of server notices.
        pub const KNOWN_MODES: [char; 7] = ['a', 'i', 'w', 'r', 'o', 'O', 's'];
        let modes_are_valid = modes
            .iter()
            .all(|(f, ms)| (*f == '-' || *f == '+') && ms.iter().all(|m| KNOWN_MODES.contains(m)));
        if !modes_are_valid {
            return Ok(Some(IrcReply::ErrUModeUnknownFlag { nick }));
        }
        let mut user_data = self.0.write().await;
        if !user_data.registered.load(Ordering::Acquire) {
            Err(InternalIrcError::UserStateError(
                "Cannot change of an unregistered user",
            ))
        } else if user_data.nick != Some(nick.to_owned()) {
            Ok(Some(IrcReply::ErrUsersDontMatch { nick }))
        } else {
            let current_flags = user_data.modes.clone();
            let mut new_user_mode_flags: HashSet<char> = current_flags.clone();
            for (flag, inner_modes) in modes {
                for mode in inner_modes {
                    match flag {
                        '+' => {
                            if !current_flags.contains(&mode) {
                                new_user_mode_flags.insert(mode);
                            }
                        }
                        '-' => {
                            if current_flags.contains(&mode) {
                                new_user_mode_flags.remove(&mode);
                            }
                        }
                        _ => panic!("cannot happened, filtered before"),
                    }
                }
            }
            user_data.modes = new_user_mode_flags;
            Ok(None)
        }
    }
}
