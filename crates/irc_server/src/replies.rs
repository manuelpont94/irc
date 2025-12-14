use crate::constants::*;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum IrcReply<'a> {
    // Connection registration
    Welcome {
        nick: &'a str,
        user: &'a str,
        host: &'a str,
    },
    YourHost {
        servername: &'a str,
        version: &'a str,
    },
    Created {
        date: &'a str,
    },
    MyInfo {
        servername: &'a str,
        version: &'a str,
        modes: &'a str,
    },

    // User modes
    UModeIs {
        nick: &'a str,
        modes: &'a str,
    },
    ErrUModeUnknownFlag {
        nick: &'a str,
    },
    ErrUsersDontMatch {
        nick: &'a str,
    },

    // Channel operations
    Topic {
        channel: &'a str,
        topic: &'a str,
    },
    NoTopic {
        channel: &'a str,
    },
    Names {
        channel: &'a str,
        names: Vec<&'a str>,
    },
    List {
        channel: &'a str,
        visible: u32,
        topic: &'a str,
    },
    ListEnd,

    // Errors
    ErrNeedMoreParams {
        command: &'a str,
    },
    ErrUnknownCommand {
        nick: &'a str,
        command: &'a str,
    },
    ErrNoSuchNick {
        nick: &'a str,
    },
    ErrNoSuchChannel {
        channel: &'a str,
    },
    ErrNotRegistered,
}

impl<'a> IrcReply<'a> {
    pub fn format(&self) -> String {
        match self {
            IrcReply::Welcome {
                nick, user, host, ..
            } => format!(
                ":{SERVER_NAME} {RPL_WELCOME_NB:03} {nick} :{RPL_WELCOME_STR} {nick}!{user}@{host}"
            ),
            IrcReply::UModeIs { nick, modes } => {
                format!(":{SERVER_NAME} {RPL_UMODEIS_NB:03} {nick} :{modes}")
            }
            IrcReply::ErrUModeUnknownFlag { nick } => format!(
                ":{SERVER_NAME} {ERR_UMODEUNKNOWNFLAG_NB:03} {nick} :{ERR_UMODEUNKNOWNFLAG_STR}"
            ),
            IrcReply::ErrUsersDontMatch { nick } => format!(
                ":{SERVER_NAME} {ERR_USERSDONTMATCH_NB:03} {nick} :{ERR_USERSDONTMATCH_STR}"
            ),
            IrcReply::ErrUnknownCommand { nick, command } => format!(
                ":{SERVER_NAME} {ERR_UNKNOWNCOMMAND_NB:03} {nick} {command} :{ERR_UNKNOWNCOMMAND_STR}"
            ),
            _ => todo!("Implement remaining reply variants"),
        }
    }
}

#[inline]
fn handle_sasl(sasl_status: bool) -> &'static str {
    let mut sasl = "";
    if sasl_status {
        sasl = "sasl";
    }
    sasl
}

#[inline]
pub fn handle_multi_prefix(handle_multi_prefix_status: bool) -> &'static str {
    let mut multi_prefix = "";
    if handle_multi_prefix_status {
        multi_prefix = "multi-prefix";
    }
    multi_prefix
}

pub fn handle_echo_message(echo_message_status: bool) -> &'static str {
    let mut echo_message = "";

    if echo_message_status {
        echo_message = "echo-message";
    }
    echo_message
}

pub fn get_server_cap_reply(
    nick: &str,
    command: &str,
    sasl_status: bool,
    echo_message_status: bool,
    handle_multi_prefix_status: bool,
) -> String {
    format!(
        "CAP {nick} {command} :{}{}{}",
        handle_sasl(sasl_status),
        handle_echo_message(echo_message_status),
        handle_multi_prefix(handle_multi_prefix_status)
    )
}
