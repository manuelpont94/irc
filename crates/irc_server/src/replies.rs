use crate::{
    constants::*,
    types::{ChannelName, Nickname, Topic, Username},
};

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
        user: &'a Username,
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
    Topic {
        nick: &'a Nickname,
        channel: &'a ChannelName,
        topic: &'a Topic,
    },
    NoTopic {
        nick: &'a Nickname,
        channel: &'a ChannelName,
    },
    Names {
        nick: &'a Nickname,
        channel: &'a ChannelName,
        visibility: &'a str,
        names: &'a str,
    },
    EndOfName {
        nick: &'a Nickname,
        channel: &'a ChannelName,
    },
    List {
        channel: &'a ChannelName,
        visible: u32,
        topic: &'a Topic,
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
        channel: &'a ChannelName,
    },
    ErrNotRegistered {
        nick: &'a Nickname,
    },
    ErrBannedFromChan {
        channel: &'a ChannelName,
    },
    ErrInviteOnlyChan {
        channel: &'a ChannelName,
    },
    ErrBadChannelKey {
        channel: &'a ChannelName,
    },
    ErrChannelIsFull {
        channel: &'a ChannelName,
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

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum MessageReply<'a> {
    BroadcastJoin {
        nick: &'a Nickname,
        user: &'a Username,
        host: &'a str,
        channel: &'a ChannelName,
    },
    NicknamePrivMsg {
        nick_from: &'a Nickname,
        user_from: &'a Username,
        host_from: &'a str,
        nick_to: &'a Nickname,
        message: &'a str,
    },
    ChannelPrivMsg {
        nick_from: &'a Nickname,
        user_from: &'a Username,
        host_from: &'a str,
        channel: &'a ChannelName,
        message: &'a str,
    },
}
impl<'a> MessageReply<'a> {
    pub fn format(&self) -> String {
        match self {
            MessageReply::BroadcastJoin {
                nick,
                user,
                host,
                channel,
            } => format!(":{nick}!{user}@{host} JOIN :{channel}"),
            MessageReply::NicknamePrivMsg {
                nick_from,
                user_from,
                host_from,
                nick_to,
                message,
            } => format!(":{nick_from}!{user_from}@{host_from} PRIVMSG {nick_to} :{message}"),
            MessageReply::ChannelPrivMsg {
                nick_from,
                user_from,
                host_from,
                channel,
                message,
            } => format!(":{nick_from}!{user_from}@{host_from} PRIVMSG {channel} :{message}"),
        }
    }
}
