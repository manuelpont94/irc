use crate::{
    errors::InternalIrcError,
    server_state::ServerState,
    types::MessageTo,
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
    msgtarget: Vec<MessageTo>,
    message: String,
    server_state: &ServerState,
    user_state: &UserState,
) -> Result<UserStatus, InternalIrcError> {
    // on reparse msgtarger ...

    for target in msgtarget {
        match target {
            MessageTo::ChannelName(c) => todo!(),
            MessageTo::NickUserHost(_nuh) => error!("PRIVMSG to NickUserHost not implemented yet"),
            MessageTo::Nickname(n) => todo!(),
            MessageTo::UserHostServer(_uhs) => todo!(),
            MessageTo::UserHost(_uh) => error!("PRIVMSG to UserHost not implemented yet"),
            MessageTo::TargetMask(_tm) => error!("PRIVMSG to TargetMask not implemented yet"),
        }
    }
    Ok(UserStatus::Active)
}
