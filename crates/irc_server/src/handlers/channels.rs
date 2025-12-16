use log::{error, info};
use nom::HexDisplay;

use crate::{
    channels_models::{ChannelMessage, IrcChannel, SubscriptionControl},
    errors::InternalIrcError,
    replies::IrcReply,
    server_state::ServerState,
    user_state::UserState,
};

pub async fn handle_join_channel(
    channels: Vec<String>,
    keys: Option<Vec<String>>,
    client_id: usize,
    server_state: &ServerState,
    user_state: &UserState,
) -> Result<Option<String>, InternalIrcError> {
    // 3.2.1 Join message

    //       Command: JOIN
    //    Parameters: ( <channel> *( "," <channel> ) [ <key> *( "," <key> ) ] )
    //                / "0"

    //    The JOIN command is used by a user to request to start listening to
    //    the specific channel.  Servers MUST be able to parse arguments in the
    //    form of a list of target, but SHOULD NOT use lists when sending JOIN
    //    messages to clients.

    //    Once a user has joined a channel, he receives information about
    //    all commands his server receives affecting the channel.  This
    //    includes JOIN, MODE, KICK, PART, QUIT and of course PRIVMSG/NOTICE.
    //    This allows channel members to keep track of the other channel
    //    members, as well as channel modes.

    //    If a JOIN is successful, the user receives a JOIN message as
    //    confirmation and is then sent the channel's topic (using RPL_TOPIC) and
    //    the list of users who are on the channel (using RPL_NAMREPLY), which
    //    MUST include the user joining.

    //    Note that this message accepts a special argument ("0"), which is
    //    a special request to leave all channels the user is currently a member
    //    of.  The server will process this message as if the user had sent
    //    a PART command (See Section 3.2.2) for each channel he is a member
    //    of.

    // User sends JOIN #test
    // │
    // ├─ check: user is registered?
    // │    └─ no → ERR_NOTREGISTERED (451)
    // │
    // ├─ check: channel exists?
    // │    └─ no → create channel
    // │           └─ give +o to user
    // │
    // ├─ add user to channel members
    // │
    // ├─ broadcast:
    // │    :nick!user@host JOIN #test
    // │
    // ├─ send topic (if any)
    // │    RPL_TOPIC (332) or RPL_NOTOPIC (331)
    // │
    // ├─ send names list
    // │    RPL_NAMREPLY (353)
    // │    RPL_ENDOFNAMES (366)
    //
    // Channels are created implicitly on first JOIN
    // First JOINer gets +o

    let caracs = user_state.get_caracs().await;
    if !caracs.registered {
        let nick = match caracs.nick {
            Some(nick) => nick.clone(),
            None => "*".to_owned(),
        };
        return Ok(Some(
            crate::replies::IrcReply::ErrNotRegistered { nick: &nick }.format(),
        ));
    }
    for channel_name in channels {
        match server_state.handle_join(channel_name, client_id).await {
            Ok(channel_tx) => {
                let irc_reply = IrcReply::Join {
                    nick: &caracs.clone().nick.unwrap_or("".to_owned()),
                    user: &caracs.clone().user.unwrap_or("".to_owned()),
                    host: &format!("{}", caracs.addr),
                    channel: &channel_name,
                };
                let welcome_channel_message = ChannelMessage::new(irc_reply.format());
                channel_tx
                    .broadcast_message(welcome_channel_message)
                    .unwrap();

                // in progress
                // RPL_TOPIC (Numeric 332): Sent only to the joining user.
                // RPL_NAMREPLY (Numeric 353) & RPL_ENDOFNAMES (Numeric 366): Sent only to the joining user.
                //
            }
            Err(e) => (),
        }
        //
        // broadcast
        //
    }
    Ok(None)
}
