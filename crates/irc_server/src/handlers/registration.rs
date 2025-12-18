use crate::{
    errors::InternalIrcError,
    message_models::IrcMessage,
    replies::IrcReply,
    server_state::ServerState,
    types::{Nickname, Realname, Username},
    user_state::{UserState, UserStatus},
};

pub const IRC_SERVER_CAP_MULTI_PREFIX: bool = false;
pub const IRC_SERVER_CAP_SASL: bool = false;
pub const IRC_SERVER_CAP_ECHO_MESSAGE: bool = false;

// 3.1 CAP LS [version]

// Client → server OR server → client.
// Requests server capability listing.
// Server replies with its full set.
// C: CAP LS 302
// S: CAP * LS :sasl multi-prefix echo-message

pub async fn handle_cap_ls_response(
    _client_id: usize,
    _server: &ServerState,
    user_state: &UserState,
) -> Result<UserStatus, InternalIrcError> {
    let user_caracs = user_state.get_caracs().await;
    let nick = if user_caracs.registered {
        user_caracs.nick.unwrap().clone()
    } else {
        Nickname("*".to_string())
    };
    let irc_reply = IrcReply::CapLs {
        nick: &nick,
        capabilities: &get_capabilities(),
    };
    let cap_list_message = IrcMessage::new(irc_reply.format());
    let _ = user_state.tx_outbound.send(cap_list_message).await;
    // :server CAP * LS :chghost echo-message extended-join invite-notify
    // :server CAP * LS :message-tags multi-prefix sasl
    if &nick != &Nickname("*".to_owned()) {
        Ok(UserStatus::Handshaking)
    } else {
        Ok(UserStatus::Active)
    }
    // :server CAP * LS :chghost echo-message extended-join invite-notify
    // :server CAP * LS :message-tags multi-prefix sasl
}
// 3.2 CAP LIST
// Client → server.
// Server returns the list of capabilities currently active for this client.

pub async fn handle_cap_list_response(
    _client_id: usize,
    _server: &ServerState,
    user_state: &UserState,
) -> Result<UserStatus, InternalIrcError> {
    let user_caracs = user_state.get_caracs().await;
    let nick = if user_caracs.registered {
        user_caracs.nick.unwrap().clone()
    } else {
        Nickname("*".to_string())
    };
    let irc_reply = IrcReply::CapList {
        nick: &nick,
        capabilities: &get_capabilities(),
    };
    let cap_list_message = IrcMessage::new(irc_reply.format());
    let _ = user_state.tx_outbound.send(cap_list_message).await;
    // :server CAP * LS :chghost echo-message extended-join invite-notify
    // :server CAP * LS :message-tags multi-prefix sasl
    if &nick != &Nickname("*".to_owned()) {
        Ok(UserStatus::Handshaking)
    } else {
        Ok(UserStatus::Active)
    }
}

fn get_capabilities() -> String {
    let mut capabilities_string = String::new();
    if IRC_SERVER_CAP_SASL {
        capabilities_string.push_str("sasl ");
    };
    if IRC_SERVER_CAP_ECHO_MESSAGE {
        capabilities_string.push_str("echo-message ");
    }
    if IRC_SERVER_CAP_MULTI_PREFIX {
        capabilities_string.push_str("multi-prefix ");
    }
    capabilities_string.trim().to_string()
}

// 3.7 CAP END
// Client → server.
// Ends negotiation.
// After this, client typically expects start of normal IRC registration.

pub fn handle_cap_end_response() -> Result<UserStatus, InternalIrcError> {
    Ok(UserStatus::Handshaking)
}

//     3.1.2 Nick message
//       Command: NICK
//    Parameters: <nickname>
//    NICK command is used to give user a nickname or change the existing
//    one.
pub async fn handle_nick_registration(
    nick: Nickname,
    _client_id: usize,
    user_state: &UserState,
    server_state: &ServerState,
) -> Result<UserStatus, InternalIrcError> {
    user_state.with_nick(nick).await;
    when_registered(user_state, server_state).await
}

pub async fn handle_user_registration(
    user_name: Username,
    mode: u8,
    real_name: Realname,
    _client_id: usize,
    user_state: &UserState,
    server_state: &ServerState,
) -> Result<UserStatus, InternalIrcError> {
    user_state.with_user(user_name, real_name, mode).await;
    when_registered(user_state, server_state).await
}

pub async fn when_registered(
    user_state: &UserState,
    server_state: &ServerState,
) -> Result<UserStatus, InternalIrcError> {
    if user_state.is_registered().await {
        let user_data = user_state.get_caracs().await;
        let nick = user_data.nick.unwrap();
        let user = user_data.user.unwrap();
        let host = user_data.addr;
        server_state.add_connecting_user(user_state).await?;
        let welcome_message = IrcMessage::new(
            IrcReply::Welcome {
                nick: &nick,
                user: &format!("{user}"),
                host: &format!("{host:?}"),
            }
            .format(),
        );
        let _ = user_state.tx_outbound.send(welcome_message).await;
        Ok(UserStatus::Active)
    } else {
        Ok(UserStatus::Handshaking)
    }
}

pub async fn handle_mode_registration(
    nick: Nickname,
    modes: Vec<(char, Vec<char>)>,
    user_state: &UserState,
) -> Result<UserStatus, InternalIrcError> {
    match user_state.with_modes(&nick, modes).await {
        Ok(Some(status)) => {
            let status_message = IrcMessage::new(status.format());
            let _ = user_state.tx_outbound.send(status_message);
        }
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    Ok(UserStatus::Active)
}

pub async fn handle_quit_registration(
    reason: Option<String>,
    client_id: usize,
    _user_state: &UserState,
    server_state: &ServerState,
) -> Result<UserStatus, InternalIrcError> {
    server_state.handle_quit(client_id, reason.clone()).await;
    Ok(UserStatus::Leaving(reason))
}
