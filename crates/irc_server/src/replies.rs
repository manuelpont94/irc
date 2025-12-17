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
    Join {
        nick: &'a str,
        user: &'a str,
        host: &'a str,
        channel: &'a str,
    },
    Topic {
        nick: &'a str,
        channel: &'a str,
        topic: &'a str,
    },
    NoTopic {
        nick: &'a str,
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
    ErrNotRegistered {
        nick: &'a str,
    },
    ErrBannedFromChan {
        channel: &'a str,
    },
    ErrInviteOnlyChan {
        channel: &'a str,
    },
    ErrBadChannelKey {
        channel: &'a str,
    },
    ErrChannelIsFull {
        channel: &'a str,
    },
}

//

impl<'a> IrcReply<'a> {
    pub fn format(&self) -> String {
        match self {
            // registration replies & errors
            IrcReply::Welcome {
                nick, user, host, ..
            } => format!(
                ":{SERVER_NAME} {RPL_WELCOME_NB:03} {nick} :{RPL_WELCOME_STR} {nick}!{user}@{host}"
            ),
            IrcReply::Join {
                nick,
                user,
                host,
                channel,
            } => format!(":{nick}!{user}@{host} JOIN {channel}"),
            IrcReply::UModeIs { nick, modes } => {
                format!(":{SERVER_NAME} {RPL_UMODEIS_NB:03} {nick} :{modes}")
            }
            IrcReply::ErrUModeUnknownFlag { nick } => format!(
                ":{SERVER_NAME} {ERR_UMODEUNKNOWNFLAG_NB:03} {nick} :{ERR_UMODEUNKNOWNFLAG_STR}"
            ),
            IrcReply::ErrUsersDontMatch { nick } => format!(
                ":{SERVER_NAME} {ERR_USERSDONTMATCH_NB:03} {nick} :{ERR_USERSDONTMATCH_STR}"
            ),
            IrcReply::ErrNotRegistered { nick } => {
                format!(":{SERVER_NAME} {ERR_NOTREGISTERED_NB:03} {nick} :{ERR_NOTREGISTERED_STR}")
            }
            IrcReply::ErrUnknownCommand { nick, command } => format!(
                ":{SERVER_NAME} {ERR_UNKNOWNCOMMAND_NB:03} {nick} {command} :{ERR_UNKNOWNCOMMAND_STR}"
            ),
            //Channels replies & errors
            IrcReply::NoTopic { nick, channel } => {
                format!(":{SERVER_NAME} {RPL_NOTOPIC_NB:03} {nick} {channel} :{RPL_NOTOPIC_STR}")
            }
            IrcReply::Topic {
                nick,
                channel,
                topic,
            } => format!(":{SERVER_NAME} {RPL_TOPIC_NB:03} {nick}  {channel} :{topic}"),
            IrcReply::ErrBannedFromChan { channel } => format!(
                ":{SERVER_NAME} {ERR_BANNEDFROMCHAN_NB:03} {channel} :{ERR_BANNEDFROMCHAN_STR}"
            ),
            IrcReply::ErrInviteOnlyChan { channel } => format!(
                ":{SERVER_NAME} {ERR_INVITEONLYCHAN_NB:03} {channel} :{ERR_INVITEONLYCHAN_STR}"
            ),
            IrcReply::ErrBadChannelKey { channel } => format!(
                ":{SERVER_NAME} {ERR_BADCHANNELKEY_NB:03} {channel} :{ERR_BADCHANNELKEY_STR}"
            ),
            IrcReply::ErrChannelIsFull { channel } => {
                format!(
                    ":{SERVER_NAME} {ERR_CHANNELISFULL_NB:03} {channel} :{ERR_INVITEONLYCHAN_STR}"
                )
            }
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
