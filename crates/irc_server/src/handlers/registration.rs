use crate::{
    errors::InternalIrcError, replies::IrcReply, server_state::ServerState, user_state::UserState,
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

pub fn handle_cap_ls_response(nick: &str) -> Option<String> {
    Some(crate::replies::get_server_cap_reply(
        nick,
        "LS",
        IRC_SERVER_CAP_SASL,
        IRC_SERVER_CAP_ECHO_MESSAGE,
        IRC_SERVER_CAP_MULTI_PREFIX,
    ))
    // :server CAP * LS :chghost echo-message extended-join invite-notify
    // :server CAP * LS :message-tags multi-prefix sasl
}
// 3.2 CAP LIST
// Client → server.
// Server returns the list of capabilities currently active for this client.

pub fn handle_cap_list_response(nick: &str) -> Option<String> {
    Some(crate::replies::get_server_cap_reply(
        nick,
        "LIST",
        IRC_SERVER_CAP_SASL,
        IRC_SERVER_CAP_ECHO_MESSAGE,
        IRC_SERVER_CAP_MULTI_PREFIX,
    ))
    // :server CAP * LS :chghost echo-message extended-join invite-notify
    // :server CAP * LS :message-tags multi-prefix sasl
}

// 3.7 CAP END
// Client → server.
// Ends negotiation.
// After this, client typically expects start of normal IRC registration.

pub fn handle_cap_end_response() -> Option<String> {
    None
}

//     3.1.2 Nick message
//       Command: NICK
//    Parameters: <nickname>
//    NICK command is used to give user a nickname or change the existing
//    one.
pub async fn handle_nick_registration(
    nick: String,
    user_state: &UserState,
    server_state: &ServerState,
) -> Result<Option<String>, InternalIrcError> {
    user_state.with_nick(nick).await;
    when_registered(user_state, server_state).await
}

pub async fn handle_user_registration(
    user_name: String,
    mode: u8,
    full_user_name: String,
    user_state: &UserState,
    server_state: &ServerState,
) -> Result<Option<String>, InternalIrcError> {
    user_state.with_user(user_name, full_user_name, mode).await;
    when_registered(user_state, server_state).await
}

pub async fn when_registered(
    user_state: &UserState,
    server_state: &ServerState,
) -> Result<Option<String>, InternalIrcError> {
    if user_state.is_registered().await {
        let user_data = user_state.get_caracs().await;
        let nick = user_data.nick.unwrap();
        let user = user_data.user.unwrap();
        let host = user_data.addr;
        let host_str = format!("{host:?}");
        server_state.add_connecting_user(user_state).await?;
        Ok(Some(
            IrcReply::Welcome {
                nick: &nick,
                user: &user,
                host: &host_str,
            }
            .format(),
        ))
    } else {
        Ok(None)
    }
}

pub async fn handle_mode_registration(
    nick: String,
    modes: Vec<(char, Vec<char>)>,
    user_state: &UserState,
) -> Result<Option<String>, InternalIrcError> {
    return match user_state.with_modes(&nick, modes).await {
        Ok(Some(status)) => Ok(Some(status.format())),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    };
}
