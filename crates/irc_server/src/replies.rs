use crate::{constants::*, types::Nickname};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum IrcReply<'a> {
    Pong {
        destination: &'a str,
    },
    // Capabilities
    CapLs {
        nick: &'a Nickname,
        capabilities: &'a str,
    },
    CapList {
        nick: &'a Nickname,
        capabilities: &'a str,
    },
    // Connection registration
    Welcome {
        nick: &'a Nickname,
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
        nick: &'a Nickname,
        modes: &'a str,
    },
    ErrUModeUnknownFlag {
        nick: &'a Nickname,
    },
    ErrUsersDontMatch {
        nick: &'a Nickname,
    },

    // Channel operations
    Join {
        nick: &'a Nickname,
        user: &'a str,
        host: &'a str,
        channel: &'a str,
    },
    Topic {
        nick: &'a Nickname,
        channel: &'a str,
        topic: &'a str,
    },
    NoTopic {
        nick: &'a Nickname,
        channel: &'a str,
    },
    Names {
        nick: &'a Nickname,
        channel: &'a str,
        visibility: &'a str,
        names: &'a str,
    },
    EndOfName {
        nick: &'a Nickname,
        channel: &'a str,
    },
    List {
        channel: &'a str,
        visible: u32,
        topic: &'a str,
    },
    ListEnd,

    // Errors
    ErrNeedMoreParams {
        nick: &'a Nickname,
        command: &'a str,
    },
    ErrUnknownCommand {
        nick: &'a Nickname,
        command: &'a str,
    },
    ErrNoSuchNick {
        nick: &'a Nickname,
    },
    ErrNoSuchChannel {
        channel: &'a str,
    },
    ErrNotRegistered {
        nick: &'a Nickname,
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
            // misceallanneous
            IrcReply::Pong { destination } => {
                format!(":{SERVER_NAME} PONG {destination}")
            }
            // Capabilities
            IrcReply::CapList { nick, capabilities } => {
                format!(":{SERVER_NAME} CAP {nick} LIST :{capabilities}")
            }
            IrcReply::CapLs { nick, capabilities } => {
                format!(":{SERVER_NAME} CAP {nick} LS :{capabilities}")
            }
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
            } => format!(":{nick}!{user}@{host} JOIN :{channel}"),
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
            IrcReply::Names {
                nick,
                channel,
                visibility,
                names,
            } => format!(
                ":{SERVER_NAME} {RPL_NAMREPLY_NB:03} {nick} {visibility} {channel} :{names}"
            ),
            IrcReply::EndOfName { nick, channel } => {
                format!(
                    ":{SERVER_NAME} {RPL_ENDOFNAMES_NB:03} {nick} {channel} :{RPL_ENDOFNAMES_STR}"
                )
            }
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
            IrcReply::ErrNeedMoreParams { nick, command } => {
                format!(
                    ":{SERVER_NAME} {ERR_NEEDMOREPARAMS_NB:03} {nick } {command} :{ERR_NEEDMOREPARAMS_STR}"
                )
            }
            _ => todo!("Implement remaining reply variants"),
        }
    }
}
