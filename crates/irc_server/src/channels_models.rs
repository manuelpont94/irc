use dashmap::DashSet;
use tokio::sync::{RwLock, broadcast};

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

// Use Tokio's RwLock for async/await support
#[derive(Debug)]
pub struct IrcChannel {
    pub name: String,
    // Immutable
    pub kind: ChannelType,
    pub topic: RwLock<Option<String>>,
    pub topic_set_by: RwLock<Option<usize>>,
    pub topic_set_at: RwLock<Option<u64>>,
    pub members: DashSet<usize>,
    pub operators: DashSet<String>,
    pub voiced: DashSet<String>,
    pub modes: RwLock<ChannelModes>,
    pub tx: broadcast::Sender<ChannelMessage>,
}

impl IrcChannel {
    pub fn new(name: String) -> Self {
        let (tx, _) = broadcast::channel(100); // Capacity of 1 is too small! Use ~100.

        IrcChannel {
            name,
            kind: ChannelType::Network,

            topic: RwLock::new(None),
            topic_set_by: RwLock::new(None),
            topic_set_at: RwLock::new(None),
            members: DashSet::new(),
            operators: DashSet::new(),
            voiced: DashSet::new(),
            modes: RwLock::new(ChannelModes::default()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ChannelMessage> {
        self.tx.subscribe()
    }

    pub fn broadcast_message(
        &self,
        message: ChannelMessage,
    ) -> Result<usize, broadcast::error::SendError<ChannelMessage>> {
        // works perfectly with &self
        self.tx.send(message)
    }

    pub fn add_member(&self, user_id: usize) -> bool {
        self.members.insert(user_id)
    }
}

pub enum IrcChannelOperationStatus {
    Ok,
    AlreadyMember,
    InviteOnlyChan,
    ChannelIsFull,
    NoSuchChannel,
    TooManyTargets,
    BannedFromChan,
    BadChannelKey,
    BadChanMask,
    TooManyChannels,
    UnavailableResource,
}
