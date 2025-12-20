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
    ErrNicknameInUse {
        nick: &'a Nickname,
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
        nick: &'a Nickname,
        channel: &'a ChannelName,
    },
    ErrNotOnChannel {
        nick: &'a Nickname,
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
        let server_name = crate::constants::SERVER_NAME
            .get()
            .map(|s| s.as_str())
            .unwrap_or("unknown.server");
        match self {
            // misceallanneous
            IrcReply::Pong { destination } => {
                format!(":{server_name} PONG {destination}")
            }
            // Capabilities
            IrcReply::CapList { nick, capabilities } => {
                format!(":{server_name} CAP {nick} LIST :{capabilities}")
            }
            IrcReply::CapLs { nick, capabilities } => {
                format!(":{server_name} CAP {nick} LS :{capabilities}")
            }
            // registration replies & errors
            IrcReply::Welcome { nick, user, host } => format!(
                ":{server_name} {RPL_WELCOME_NB:03} {nick} :{RPL_WELCOME_STR} {nick}!{user}@{host}"
            ),

            IrcReply::UModeIs { nick, modes } => {
                format!(":{server_name} {RPL_UMODEIS_NB:03} {nick} :{modes}")
            }
            IrcReply::ErrUModeUnknownFlag { nick } => format!(
                ":{server_name} {ERR_UMODEUNKNOWNFLAG_NB:03} {nick} :{ERR_UMODEUNKNOWNFLAG_STR}"
            ),
            IrcReply::ErrUsersDontMatch { nick } => format!(
                ":{server_name} {ERR_USERSDONTMATCH_NB:03} {nick} :{ERR_USERSDONTMATCH_STR}"
            ),
            IrcReply::ErrNotRegistered { nick } => {
                format!(":{server_name} {ERR_NOTREGISTERED_NB:03} {nick} :{ERR_NOTREGISTERED_STR}")
            }
            IrcReply::ErrUnknownCommand { nick, command } => format!(
                ":{server_name} {ERR_UNKNOWNCOMMAND_NB:03} {nick} {command} :{ERR_UNKNOWNCOMMAND_STR}"
            ),
            //Channels replies & errors
            IrcReply::NoTopic { nick, channel } => {
                format!(":{server_name} {RPL_NOTOPIC_NB:03} {nick} {channel} :{RPL_NOTOPIC_STR}")
            }
            IrcReply::Topic {
                nick,
                channel,
                topic,
            } => format!(":{server_name} {RPL_TOPIC_NB:03} {nick}  {channel} :{topic}"),
            IrcReply::Names {
                nick,
                channel,
                visibility,
                names,
            } => format!(
                ":{server_name} {RPL_NAMREPLY_NB:03} {nick} {visibility} {channel} :{names}"
            ),
            IrcReply::EndOfName { nick, channel } => {
                format!(
                    ":{server_name} {RPL_ENDOFNAMES_NB:03} {nick} {channel} :{RPL_ENDOFNAMES_STR}"
                )
            }
            IrcReply::ErrBannedFromChan { channel } => format!(
                ":{server_name} {ERR_BANNEDFROMCHAN_NB:03} {channel} :{ERR_BANNEDFROMCHAN_STR}"
            ),
            IrcReply::ErrInviteOnlyChan { channel } => format!(
                ":{server_name} {ERR_INVITEONLYCHAN_NB:03} {channel} :{ERR_INVITEONLYCHAN_STR}"
            ),
            IrcReply::ErrBadChannelKey { channel } => format!(
                ":{server_name} {ERR_BADCHANNELKEY_NB:03} {channel} :{ERR_BADCHANNELKEY_STR}"
            ),
            IrcReply::ErrChannelIsFull { channel } => {
                format!(
                    ":{server_name} {ERR_CHANNELISFULL_NB:03} {channel} :{ERR_INVITEONLYCHAN_STR}"
                )
            }
            IrcReply::ErrNoSuchChannel { nick, channel } => {
                format!(
                    ":{server_name} {ERR_NOSUCHCHANNEL_NB:03} {nick} {channel} :{ERR_NOSUCHCHANNEL_STR}"
                )
            }
            IrcReply::ErrNotOnChannel { nick, channel } => {
                format!(
                    ":{server_name} {ERR_NOTONCHANNEL_NB:03} {nick} {channel} :{ERR_NOTONCHANNEL_STR}"
                )
            }

            // Generic
            IrcReply::ErrNeedMoreParams { nick, command } => {
                format!(
                    ":{server_name} {ERR_NEEDMOREPARAMS_NB:03} {nick } {command} :{ERR_NEEDMOREPARAMS_STR}"
                )
            }
            // Registration
            IrcReply::ErrNicknameInUse { nick } => {
                format!(":{server_name} {ERR_NICKNAMEINUSE_NB:03} {nick } :{ERR_NICKNAMEINUSE_STR}")
            }

            _ => todo!("Implement remaining reply variants"),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum MessageReply<'a> {
    UpdateNick {
        old_nick: &'a Nickname,
        new_nick: &'a Nickname,
        user: &'a Username,
        host: &'a str,
    },
    BroadcastJoinMsg {
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
    PartMsg {
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
            MessageReply::BroadcastJoinMsg {
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
            MessageReply::PartMsg {
                nick_from,
                user_from,
                host_from,
                channel,
                message,
            } => format!(":{nick_from}!{user_from}@{host_from} PART {channel} {message}"),
            MessageReply::UpdateNick {
                old_nick,
                new_nick,
                user,
                host,
            } => format!(":{old_nick}!{user}@{host} NICK :{new_nick}"),
        }
    }
}
