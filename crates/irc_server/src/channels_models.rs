use crate::users::UserId;
use dashmap::{DashMap, DashSet};

#[derive(Debug, Clone)]
pub enum ChannelType {
    Network,  // '#'
    Local,    // '&'
    Modeless, // '+'
    Safe,     // '!'
}

#[derive(Debug, Clone)]
pub struct ChannelModes {
    pub invite_only: bool,      // +i
    pub moderated: bool,        // +m
    pub no_external_msgs: bool, // +n
    pub private: bool,          // +p
    pub secret: bool,           // +s
    pub topic_lock: bool,       // +t

    pub key: Option<String>,                // +k <key>
    pub user_limit: Option<u32>,            // +l <count>
    pub ban_list: DashSet<UserId>,          // +b
    pub except_list: DashSet<UserId>,       // +e
    pub invite_exceptions: DashSet<UserId>, // +I
}

impl Default for ChannelModes {
    fn default() -> Self {
        Self {
            invite_only: false,
            moderated: false,
            no_external_msgs: false,
            private: false,
            secret: false,
            topic_lock: false,

            key: None,
            user_limit: None,
            ban_list: DashSet::new(),
            except_list: DashSet::new(),
            invite_exceptions: DashSet::new(),
        }
    }
}

pub type ChannelName = String;

#[derive(Debug, Clone)]
pub struct IrcChannel {
    pub name: ChannelName,
    pub kind: ChannelType,

    pub topic: Option<String>,
    pub topic_set_by: Option<UserId>,
    pub topic_set_at: Option<u64>, // timestamp

    pub members: DashSet<UserId>,   // list of nicknames
    pub operators: DashSet<String>, // '@'
    pub voiced: DashSet<String>,    // '+'

    pub modes: ChannelModes,
}
