use crate::types::ClientId;

#[derive(Debug, Clone)]
pub struct DirectIrcMessage {
    pub sender: Option<ClientId>,
    pub raw_line: String,
}
impl DirectIrcMessage {
    pub fn new(line: String) -> Self {
        let final_line = if line.ends_with("\r\n") {
            line
        } else {
            format!("{line}\r\n")
        };
        DirectIrcMessage {
            sender: None,
            raw_line: final_line,
        }
    }

    pub fn new_with_sender(line: String, sender: ClientId) -> Self {
        let final_line = if line.ends_with("\r\n") {
            line
        } else {
            format!("{line}\r\n")
        };
        DirectIrcMessage {
            sender: Some(sender),
            raw_line: final_line,
        }
    }
}
