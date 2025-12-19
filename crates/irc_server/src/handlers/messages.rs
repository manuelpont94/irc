use crate::{
    errors::InternalIrcError,
    message_models::{BroadcastIrcMessage, DirectIrcMessage},
    replies::MessageReply,
    server_state::ServerState,
    types::{ClientId, MessageTo},
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
    client_id: ClientId,
    server_state: &ServerState,
    user_state: &UserState,
) -> Result<UserStatus, InternalIrcError> {
    let caracs = user_state.get_caracs().await;
    let nick_from = caracs.nick.unwrap();
    let user_from = caracs.user.unwrap();
    let host_from = format!("{}", caracs.addr);

    for target in msgtarget {
        match target {
            MessageTo::ChannelName(channel) => {
                let irc_channel_opt = server_state.get_channel(&channel).map(|r| r.clone());
                if let Some(irc_channel) = irc_channel_opt {
                    let mrep = MessageReply::ChannelPrivMsg {
                        nick_from: &nick_from,
                        user_from: &user_from,
                        host_from: &host_from,
                        channel: &channel,
                        message: &message,
                    };
                    let broadcast_irc_message =
                        BroadcastIrcMessage::new_with_sender(mrep.format(), client_id);
                    let _ = irc_channel.broadcast_message(broadcast_irc_message);
                }
                //todo faire le else :)
            }
            MessageTo::NickUserHost(_nuh) => error!("PRIVMSG to NickUserHost not implemented yet"),
            MessageTo::Nickname(nick_to) => {
                if let Some(user_state_dest) = server_state.get_user_state_from_nick(&nick_to) {
                    let mrep = MessageReply::NicknamePrivMsg {
                        nick_from: &nick_from,
                        user_from: &user_from,
                        host_from: &host_from,
                        nick_to: &nick_to,
                        message: &message,
                    };
                    let direct_irc_message = DirectIrcMessage::new(mrep.format());
                    let _ = user_state_dest.tx_outbound.send(direct_irc_message).await;
                }
                //todo faire le else :)
            }
            MessageTo::UserHostServer(_uhs) => {
                error!("PRIVMSG to UserHostServer not implemented yet")
            }
            MessageTo::UserHost(_uh) => error!("PRIVMSG to UserHost not implemented yet"),
            MessageTo::TargetMask(_tm) => error!("PRIVMSG to TargetMask not implemented yet"),
        }
    }
    Ok(UserStatus::Active)
}
