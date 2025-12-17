use log::{error, info};

use crate::{
    channels_models::{ChannelMessage, IrcChannelOperationStatus, SubscriptionControl},
    errors::InternalIrcError,
    message_models::IrcMessage,
    replies::IrcReply,
    server_state::ServerState,
    user_state::UserState,
};

pub async fn handle_join_channel(
    channels_keys: Vec<(String, Option<String>)>,
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
    for (channel_name, key) in channels_keys {
        match server_state
            .handle_join(channel_name.clone(), client_id, key, false)
            .await
        {
            Ok((IrcChannelOperationStatus::NewJoin, Some(channel))) => {
                let irc_reply = IrcReply::Join {
                    nick: &caracs.clone().nick.unwrap_or("*".to_owned()),
                    user: &caracs.clone().user.unwrap_or("*".to_owned()),
                    host: &format!("{}", caracs.addr),
                    channel: &channel_name,
                };
                let rx = channel.subscribe();
                let _ = user_state
                    .tx_control
                    .send(SubscriptionControl::Subscribe {
                        channel_name: channel_name.clone(),
                        receiver: rx,
                    })
                    .await;
                let welcome_channel_message = ChannelMessage::new(irc_reply.format());
                channel.broadcast_message(welcome_channel_message).unwrap();
                let potential_topic = channel.topic.read().await;
                if let Some(topic) = potential_topic.as_deref() {
                    let irc_reply = IrcReply::Topic {
                        nick: &caracs.clone().nick.unwrap_or("*".to_owned()),
                        channel: &channel_name,
                        topic: topic,
                    };
                    let topic_message = IrcMessage::new(irc_reply.format());
                    let _ = user_state.tx_outbound.send(topic_message).await;
                } else {
                    let irc_reply = IrcReply::NoTopic {
                        nick: &caracs.clone().nick.unwrap_or("*".to_owned()),
                        channel: &channel_name,
                    };
                    let no_topic_message = IrcMessage::new(irc_reply.format());
                    let _ = user_state.tx_outbound.send(no_topic_message).await;
                }
                // ├─ send names list
                // │    RPL_NAMREPLY (353)
                // │    RPL_ENDOFNAMES (366)
            }
            Ok((IrcChannelOperationStatus::ChannelIsFull, None)) => {
                let irc_reply = IrcReply::ErrChannelIsFull {
                    channel: &channel_name,
                };
                let err_channel_is_full = IrcMessage::new(irc_reply.format());
                let _ = user_state.tx_outbound.send(err_channel_is_full).await;
            }
            Ok((IrcChannelOperationStatus::BannedFromChan, None)) => {
                let irc_reply = IrcReply::ErrBannedFromChan {
                    channel: &channel_name,
                };
                let err_banned_from_chan = IrcMessage::new(irc_reply.format());
                let _ = user_state.tx_outbound.send(err_banned_from_chan).await;
            }
            Ok((IrcChannelOperationStatus::InviteOnlyChan, None)) => {
                let irc_reply = IrcReply::ErrInviteOnlyChan {
                    channel: &channel_name,
                };
                let err_invite_only_chan = IrcMessage::new(irc_reply.format());
                let _ = user_state.tx_outbound.send(err_invite_only_chan).await;
            }
            Ok((IrcChannelOperationStatus::BadChannelKey, None)) => {
                let irc_reply = IrcReply::ErrBadChannelKey {
                    channel: &channel_name,
                };
                let err_bad_channel_key = IrcMessage::new(irc_reply.format());
                let _ = user_state.tx_outbound.send(err_bad_channel_key).await;
            }
            Ok((IrcChannelOperationStatus::AlreadyMember, None)) => todo!(),
            Ok(_) => (),
            Err(_e) => (),
        }
        //
        // broadcast
        //
    }
    Ok(None)
}

fn handle_names_reply(server_state: &ServerState, channel_name: &str) -> String {
    // The RPL_NAMREPLY (353) is one of the most important numeric replies in IRC. It tells the client exactly who is currently in a channel and what their "status" is.
    // Here is a breakdown of the syntax and the specific cases mentioned in RFC 2812.

    // 1. The Syntax Breakdown
    // The format is: 353 <target_nick> <symbol> <channel> :[prefix]<nick> ...
    //     <target_nick>: The nickname of the person who just joined or requested the list.
    //     <symbol>: One of three characters (=, *, or @) indicating the visibility of the channel.
    //     <channel>: The name of the channel.
    //     :[prefix]<nick>: A space-separated list of users. Prefixes (like @ for Ops or + for Voice) indicate their permissions within that channel.

    // 2. The Three Visibility Cases
    // The RFC defines the symbols based on the channel modes (specifically +s for Secret and +p for Private).

    // Case A: Public Channels (=)
    // This is the default for standard channels. Any channel that is not set to Secret (+s) or Private (+p) uses the = symbol.

    //     Logic: channel_modes does not contain s or p.
    //     Example:
    //         :server 353 Alice = #tokio :Alice @Bob +Charlie
    //     Translation: Alice is looking at #tokio. It is a public channel. Members are Alice, Bob (an Operator), and Charlie (has Voice).

    // Case B: Private Channels (*)
    // Used when a channel has the Private mode (+p) set. In older IRC deamons, this meant the channel wouldn't show up in a global /LIST unless you knew the name.
    //     Logic: channel_modes contains p.
    //     Example:

    //         :server 353 Alice * #secret_project :Alice @Manager
    //     Translation: Alice is in #secret_project. The * tells her client the channel is marked Private.

    // Case C: Secret Channels (@)
    // Used when a channel has the Secret mode (+s) set. Secret channels are completely hidden; if you aren't in them, they don't exist as far as the server is concerned.
    //     Logic: channel_modes contains s.
    //     Example:
    //         :server 353 Alice @ #admins :Alice @SuperUser
    //     Translation: Alice is in the secret #admins channel. The @ symbol confirms the channel is in Secret mode.
    todo!()
}
