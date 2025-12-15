use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct IrcMessage {
    pub raw_line: String,
}
impl IrcMessage {
    pub fn new(line: String) -> Self {
        let final_line = if line.ends_with("\r\n") {
            line
        } else {
            format!("{line}\r\n")
        };
        IrcMessage {
            raw_line: final_line,
        }
    }
}

/// Control message sent from Server Broker to a Client Writer Task
pub enum SubscriptionControl {
    Subscribe {
        channel_name: String,
        receiver: broadcast::Receiver<IrcMessage>,
    },
    Unsubscribe(String),
}
