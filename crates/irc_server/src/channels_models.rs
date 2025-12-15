use std::sync::Arc;

use dashmap::DashSet;
use tokio::sync::broadcast;

/// Control message sent from Server Broker to a Client Writer Task
pub enum SubscriptionControl {
    Subscribe {
        channel_name: String,
        receiver: broadcast::Receiver<ChannelMessage>,
    },
    Unsubscribe(String),
}

#[derive(Debug, Clone)]
pub struct ChannelMessage {
    pub raw_line: String,
}
impl ChannelMessage {
    pub fn new(line: String) -> Self {
        let final_line = if line.ends_with("\r\n") {
            line
        } else {
            format!("{line}\r\n")
        };
        ChannelMessage {
            raw_line: final_line,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChannelType {
    Network,  // '#'
    Local,    // '&'
    Modeless, // '+'
    Safe,     // '!'
}

#[derive(Debug, Clone)]
pub struct ChannelModes {
    pub invite_only: bool,                 // +i
    pub moderated: bool,                   // +m
    pub no_external_msgs: bool,            // +n
    pub private: bool,                     // +p
    pub secret: bool,                      // +s
    pub topic_lock: bool,                  // +t
    pub key: Option<String>,               // +k <key>
    pub user_limit: Option<u32>,           // +l <count>
    pub ban_list: DashSet<usize>,          // +b
    pub except_list: DashSet<usize>,       // +e
    pub invite_exceptions: DashSet<usize>, // +I
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
    pub topic_set_by: Option<usize>,
    pub topic_set_at: Option<u64>, // timestamp

    pub members: DashSet<usize>,    // list of nicknames
    pub operators: DashSet<String>, // '@'
    pub voiced: DashSet<String>,    // '+'

    pub modes: ChannelModes,

    pub tx: Arc<broadcast::Sender<ChannelMessage>>,
}
impl IrcChannel {
    pub fn new(name: String) -> Self {
        // no need to store receiver, will be created on fly by the subscribe function
        let (tx, _) = broadcast::channel(1);

        IrcChannel {
            name,
            kind: ChannelType::Network,
            topic: None,
            topic_set_by: None,
            topic_set_at: None,
            members: DashSet::new(),
            operators: DashSet::new(),
            voiced: DashSet::new(),
            modes: ChannelModes::default(),
            tx: Arc::new(tx),
        }
    }

    /// Create a receiver for a new channel member
    pub fn subscribe(&self) -> broadcast::Receiver<ChannelMessage> {
        self.tx.subscribe() // Fresh receiver each time
    }

    pub fn broadcast_message(
        &self,
        message: ChannelMessage,
    ) -> Result<usize, broadcast::error::SendError<ChannelMessage>> {
        self.tx.send(message)
    }
}
