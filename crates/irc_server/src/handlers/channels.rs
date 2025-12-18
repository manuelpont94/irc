use std::sync::Arc;

use crate::types::*;
use crate::{
    channels_models::{ChannelMessage, IrcChannel, IrcChannelOperationStatus, SubscriptionControl},
    errors::InternalIrcError,
    message_models::IrcMessage,
    replies::IrcReply,
    server_state::ServerState,
    user_state::{UserState, UserStatus},
};

pub async fn handle_join_channel(
    channels_keys: Vec<(Channel, Option<String>)>,
    client_id: usize,
    server_state: &ServerState,
    user_state: &UserState,
) -> Result<UserStatus, InternalIrcError> {
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
    let nick = caracs.clone().nick.unwrap_or(Nickname("*".to_owned()));
    let user = caracs.clone().user.unwrap_or(Username("*".to_owned()));
    let host = &format!("{}", caracs.addr);
    if !caracs.registered {
        let nick = match caracs.nick {
            Some(nick) => nick,
            None => Nickname("*".to_owned()),
        };
        let irc_reply = IrcReply::ErrNotRegistered { nick: &nick };
        let not_registered_message = IrcMessage::new(irc_reply.format());
        let _ = user_state.tx_outbound.send(not_registered_message).await;
        return Ok(UserStatus::Active);
    }
    for (channel_name, key) in channels_keys {
        match server_state
            .handle_join(channel_name.0.clone(), client_id, key, false)
            .await
        {
            Ok((IrcChannelOperationStatus::NewJoin, Some(channel))) => {
                let irc_reply = IrcReply::Join {
                    nick: &nick,
                    user: &user.0,
                    host,
                    channel: &channel_name.0,
                };
                let rx = channel.subscribe();
                let _ = user_state
                    .tx_control
                    .send(SubscriptionControl::Subscribe {
                        channel_name: channel_name.0.clone(),
                        receiver: rx,
                    })
                    .await;
                let welcome_channel_message = ChannelMessage::new(irc_reply.format());
                channel.broadcast_message(welcome_channel_message);
                let potential_topic = channel.topic.read().await;
                if let Some(topic) = potential_topic.as_deref() {
                    let irc_reply = IrcReply::Topic {
                        nick: &nick,
                        channel: &channel_name.0,
                        topic: topic,
                    };
                    let topic_message = IrcMessage::new(irc_reply.format());
                    let _ = user_state.tx_outbound.send(topic_message).await;
                } else {
                    let irc_reply = IrcReply::NoTopic {
                        nick: &nick,
                        channel: &channel_name.0,
                    };
                    let no_topic_message = IrcMessage::new(irc_reply.format());
                    let _ = user_state.tx_outbound.send(no_topic_message).await;
                }

                let (visibility, member_list) = handle_names_reply(&channel, server_state).await;
                // ├─ send names list
                // │    RPL_NAMREPLY (353)
                // │    RPL_ENDOFNAMES (366)
                let irc_reply = IrcReply::Names {
                    nick: &nick,
                    channel: &channel_name.0,
                    visibility: &visibility,
                    names: &member_list,
                };
                let channel_names = IrcMessage::new(irc_reply.format());
                let _ = user_state.tx_outbound.send(channel_names).await;
                let irc_reply = IrcReply::EndOfName {
                    nick: &nick,
                    channel: &channel_name.0,
                };
                let channel_end_of_names = IrcMessage::new(irc_reply.format());
                let _ = user_state.tx_outbound.send(channel_end_of_names).await;
                user_state.join_channel(&channel_name.0).await
            }
            Ok((IrcChannelOperationStatus::ChannelIsFull, None)) => {
                let irc_reply = IrcReply::ErrChannelIsFull {
                    channel: &channel_name.0,
                };
                let err_channel_is_full = IrcMessage::new(irc_reply.format());
                let _ = user_state.tx_outbound.send(err_channel_is_full).await;
            }
            Ok((IrcChannelOperationStatus::BannedFromChan, None)) => {
                let irc_reply = IrcReply::ErrBannedFromChan {
                    channel: &channel_name.0,
                };
                let err_banned_from_chan = IrcMessage::new(irc_reply.format());
                let _ = user_state.tx_outbound.send(err_banned_from_chan).await;
            }
            Ok((IrcChannelOperationStatus::InviteOnlyChan, None)) => {
                let irc_reply = IrcReply::ErrInviteOnlyChan {
                    channel: &channel_name.0,
                };
                let err_invite_only_chan = IrcMessage::new(irc_reply.format());
                let _ = user_state.tx_outbound.send(err_invite_only_chan).await;
            }
            Ok((IrcChannelOperationStatus::BadChannelKey, None)) => {
                let irc_reply = IrcReply::ErrBadChannelKey {
                    channel: &channel_name.0,
                };
                let err_bad_channel_key = IrcMessage::new(irc_reply.format());
                let _ = user_state.tx_outbound.send(err_bad_channel_key).await;
            }
            Ok((IrcChannelOperationStatus::AlreadyMember, None)) => (),
            Ok(_) => (),
            Err(_e) => (),
        }
        //
        // broadcast
        //
    }
    Ok(UserStatus::Active)
}

async fn handle_names_reply(
    channel: &Arc<IrcChannel>,
    server_state: &ServerState,
) -> (String, String) {
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
    //let is_public = modes.

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
    let modes = channel.modes.read().await;
    let is_secret_channel = modes.secret;
    let is_private_channel = modes.private;

    let visibility_symbol = {
        if is_secret_channel {
            "@"
        } else if is_private_channel {
            "*"
        } else {
            "="
        }
    };

    let mut member_list = String::new();
    let channel_members = channel
        .members
        .iter()
        .map(|m| m.clone())
        .collect::<Vec<usize>>();

    for client_id in channel_members {
        if let Some(user) = server_state.users.get(&client_id) {
            let prefix = if channel.operators.contains(&client_id) {
                "@"
            } else if channel.voiced.contains(&client_id) {
                "+"
            } else {
                ""
            };
            let user_caracs = user.user.read().await;
            let nick = user_caracs.nick.as_ref().unwrap().clone();
            member_list.push_str(&format!("{prefix}{nick} "));
        }
    }
    (visibility_symbol.to_owned(), member_list.trim().to_string())
}

pub async fn handle_invalid_join_channel(
    command: String,
    user_state: &UserState,
) -> Result<UserStatus, InternalIrcError> {
    let user_caracs = user_state.get_caracs().await;
    let nick = if user_caracs.registered {
        user_caracs.nick.unwrap().clone()
    } else {
        Nickname("*".to_string())
    };
    let irc_reply = IrcReply::ErrNeedMoreParams {
        nick: &nick,
        command: &command,
    };
    let invalid_join_message = IrcMessage::new(irc_reply.format());
    let _ = user_state.tx_outbound.send(invalid_join_message).await;
    Ok(UserStatus::Active)
}
