use dashmap::DashSet;
use log::{error, info};
use tokio::sync::{RwLock, broadcast};

use crate::{
    message_models::BroadcastIrcMessage,
    types::{ChannelName, ClientId, Topic},
};

/// Control message sent from Server Broker to a Client Writer Task
pub enum SubscriptionControl {
    Subscribe {
        channel_name: ChannelName,
        receiver: broadcast::Receiver<BroadcastIrcMessage>,
    },
    Unsubscribe(ChannelName),
}

#[derive(Debug, Clone)]
pub enum ChannelType {
    Network,  // '#'
    Local,    // '&'
    Modeless, // '+'
    Safe,     // '!'
}

// Use Tokio's RwLock for async/await support
#[derive(Debug)]
pub struct IrcChannel {
    pub name: ChannelName,
    // Immutable
    pub kind: ChannelType,
    pub topic: RwLock<Option<Topic>>,
    pub topic_set_by: RwLock<Option<usize>>,
    pub topic_set_at: RwLock<Option<u64>>,
    pub members: DashSet<ClientId>,
    pub operators: DashSet<ClientId>,
    pub voiced: DashSet<ClientId>,
    pub modes: RwLock<ChannelModes>,
    pub tx: broadcast::Sender<BroadcastIrcMessage>,
}

impl IrcChannel {
    pub fn new(name: ChannelName) -> Self {
        let tx = broadcast::channel(5000).0;

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

    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastIrcMessage> {
        self.tx.subscribe()
    }

    pub fn broadcast_message(&self, message: BroadcastIrcMessage) {
        // works perfectly with &self
        info!(
            "Broadcasting to {}: {} receivers",
            self.name,
            self.tx.receiver_count()
        );
        match self.tx.send(message) {
            Ok(n) => info!("Sent to {} receivers", n),
            Err(e) => error!("Broadcast failed: {:?}", e),
        }
    }

    pub fn add_member(&self, client_id: ClientId) -> bool {
        self.members.insert(client_id)
    }

    pub fn remove_member(&self, client_id: ClientId) {
        let _ = self.members.remove(&client_id);
    }

    pub fn add_operator(&self, client_id: ClientId) -> bool {
        self.operators.insert(client_id)
    }

    pub async fn is_banned(&self, client_id: ClientId) -> bool {
        let modes = self.modes.read().await;
        modes.ban_list.contains(&client_id)
    }

    pub async fn add_ban_user(&self, client_id: ClientId) -> bool {
        let modes = self.modes.write().await;
        modes.ban_list.insert(client_id)
    }
}

pub enum IrcChannelOperationStatus {
    NewJoin,
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

// n RFC 2812, which defines the Internet Relay Chat (IRC) protocol, channel modes are settings that dictate how a channel operates. Each mode can control various aspects of channel access and interaction. Here's a breakdown of each mode you mentioned, including its implications:

// Invite Only (+i):

// Description: When a channel is set to invite-only mode (+i), users cannot join the channel unless they are invited by a channel operator (operator).
// Implications: This mode is useful for maintaining a controlled environment where only selected users can participate. It prevents unauthorized users from joining the discussion.
// Moderated (+m):

// Description: In moderated channels (+m), only channel operators and users with voice (+v) can send messages to the channel.
// Implications: This mode helps to prevent spam and keeps the discussion focused. It ensures that the channel is moderated, which is particularly useful in larger channels.
// No External Messages (+n):

// Description: The no external messages mode (+n) prevents messages from users outside the channel from being sent to that channel.
// Implications: This ensures that only users who are currently in the channel can participate in conversations, reducing unwanted noise from other channels.
// Private (+p):

// Description: A private channel (+p) does not appear in the channel list when users query for channels. Only users who know the channel name can join.
// Implications: This mode enhances privacy and security by preventing the channel from being listed publicly, which can be important for confidential discussions.
// Secret (+s):

// Description: A secret channel (+s) is similar to a private channel, but it also prevents outsiders from knowing the channel's existence. This means it will not even show up in a list of channels.
// Implications: This mode is used for discussions that require a high level of confidentiality. Only users who are already in the channel can invite new users.
// Topic Lock (+t):

// Description: When the topic lock mode (+t) is enabled, only channel operators can change the channel topic.
// Implications: Enforcing a topic lock is useful in maintaining order and ensuring that important channel information (the topic) is not changed arbitrarily.
// Key (+k ):

// Description: A channel can require a key (password) for users to join it, indicated by the +k mode. The key is set by the channel operator.
// Implications: This increases security by controlling access based on a shared secret. Users must provide the correct key to gain entry to the channel.
// User Limit (+l ):

// Description: This mode sets a limit on the number of users that can join a channel. If the limit is reached, new users are not able to join until others leave.
// Implications: This is useful for managing the size of the channel, especially in roles where discussions may become unmanageable if too many participants are involved.
// Ban List (+b):

// Description: The ban list mode (+b) allows channel operators to specify users who are banned from entering the channel. Bans can be applied to users based on their hostmask or nicknames.
// Implications: This feature helps maintain a safe and respectful environment by excluding users who engage in disruptive behavior.
// Except List (+e):

// Description: The except list (+e) allows specific users to bypass the ban list. This means that although a user may be banned, those on the except list can still enter.
// Implications: This mode gives operators flexibility to manage bans and allows them to provide exceptions for trusted users.
// Invite Exceptions (+I):

// Description: The invite exceptions (+I) mode allows specific users to join an invite-only channel without needing an invitation from a channel operator.
// Implications: This mode is useful for allowing trusted users, such as co-moderators or guests, to join easily while maintaining the exclusivity of the invite-only status.
// These modes collectively provide a robust mechanism for IRC channel management, allowing operators to customize the interaction and accessibility of channels to fit their needs and maintain a desired environment.

#[derive(Debug, Clone)]
pub struct ChannelModes {
    pub invite_only: bool,                    // +i
    pub moderated: bool,                      // +m
    pub no_external_msgs: bool,               // +n
    pub private: bool,                        // +p
    pub secret: bool,                         // +s
    pub topic_lock: bool,                     // +t
    pub key: Option<String>,                  // +k <key>
    pub user_limit: Option<usize>,            // +l <count>
    pub ban_list: DashSet<ClientId>,          // +b
    pub except_list: DashSet<ClientId>,       // +e
    pub invite_exceptions: DashSet<ClientId>, // +I
}
//TODO invite exceptions
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
