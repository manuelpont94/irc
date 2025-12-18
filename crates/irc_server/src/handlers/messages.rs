use crate::{
    errors::InternalIrcError,
    ops::parsers::{
        channel_parser, 
        msgto_user_host_server_splitted_parser, msgto_user_host_splitted_parser, nickname_parser,
        targetmask_parser,
    },
    server_state::ServerState,
    user_state::{UserState, UserStatus},
};
use log::error;
// 3.3.1 Private messages

//       Command: PRIVMSG
//    Parameters: <msgtarget> <text to be sent>

//    PRIVMSG is used to send private messages between users, as well as to
//    send messages to channels.  <msgtarget> is usually the nickname of
//    the recipient of the message, or a channel name.

//    The <msgtarget> parameter may also be a host mask (#<mask>) or server
//    mask ($<mask>).  In both cases the server will only send the PRIVMSG
//    to those who have a server or host matching the mask.  The mask MUST
//    have at least 1 (one) "." in it and no wildcards following the last
//    ".".  This requirement exists to prevent people sending messages to
//    "#*" or "$*", which would broadcast to all users.  Wildcards are the
//    '*' and '?'  characters.  This extension to the PRIVMSG command is
//    only available to operators.

//    Numeric Replies:

//            ERR_NORECIPIENT                 ERR_NOTEXTTOSEND
//            ERR_CANNOTSENDTOCHAN            ERR_NOTOPLEVEL
//            ERR_WILDTOPLEVEL                ERR_TOOMANYTARGETS
//            ERR_NOSUCHNICK
//            RPL_AWAY

pub async fn handle_privmsg(
    msgtarget: Vec<String>,
    message: String,
    server_state: &ServerState,
    user_state: &UserState,
) -> Result<UserStatus, InternalIrcError> {
    // on reparse msgtarger ...

    for target in msgtarget {
        match nickname_parser(&target) {
            Ok((_rem, nick)) => {
                // private message to nick
                // if let Some(_target_user_state) = server_state.get_user_by_nick(nick).await {
                //     // send message to target_user_state
                // } else {
                //     // ERR_NOSUCHNICK
                //     error!("No such nick: {}", nick);
                // }
                continue;
            }
            Err(_) => {}
        }
        match channel_parser(&target) {
            Ok((_rem, channel_name)) => {
                // message to channel
                continue;
            }
            Err(_) => {}
        }
        match msgto_user_host_splitted_parser(&target) {
            Ok((_rem, (user, host))) => {
                // private message to user@host
                continue;
            }
            Err(_) => {}
        }
        // match msgto_nick_user_host_splitted_parser(&target) {
        //     Ok((_rem, (nick, user, host))) => {
        //         continue;
        //     }
        //     Err(_) => {}
        // }
        // match msgto_user_host_server_splitted_parser(&target) {
        //     Ok((_rem, (user, opt_host, server))) => {
        //         // private message to user@host/server
        //         continue;
        //     }
        //     Err(_) => {}
        // }
        // match targetmask_parser(&target) {
        //     Ok((_rem, _mask)) => {
        //         // message to hostmask or servermask
        //         continue;
        //     }
        //     Err(_) => {}
        // }
        error!("Invalid msgtarget: {target}");
    }
    Ok(UserStatus::Active)
}
