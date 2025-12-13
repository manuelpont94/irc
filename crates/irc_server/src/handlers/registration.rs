use crate::{
    constants::{RPL_WELCOME_NB, RPL_WELCOME_STR},
    errors::IrcError,
    users::UserState,
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

#[inline]
fn handle_sasl() -> &'static str {
    let mut sasl = "";
    if IRC_SERVER_CAP_SASL {
        sasl = "sasl";
    }
    sasl
}

#[inline]
pub fn handle_multi_prefix() -> &'static str {
    let mut multi_prefix = "";
    if IRC_SERVER_CAP_MULTI_PREFIX {
        multi_prefix = "multi-prefix";
    }
    multi_prefix
}

pub fn handle_echo_message() -> &'static str {
    let mut echo_message = "";

    if IRC_SERVER_CAP_ECHO_MESSAGE {
        echo_message = "echo-message";
    }
    echo_message
}

pub fn handle_cap_ls_response(user: &str) -> Option<String> {
    Some(format!(
        "CAP {} LS :{}{}{}",
        user,
        handle_sasl(),
        handle_echo_message(),
        handle_multi_prefix()
    ))
    // :server CAP * LS :chghost echo-message extended-join invite-notify
    // :server CAP * LS :message-tags multi-prefix sasl
}

// 3.2 CAP LIST
// Client → server.
// Server returns the list of capabilities currently active for this client.

pub fn handle_cap_list_response(user: &str) -> Option<String> {
    Some(format!(
        "CAP {} LIST :{}{}{}",
        user,
        handle_sasl(),
        handle_echo_message(),
        handle_multi_prefix()
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
) -> Result<Option<String>, IrcError> {
    user_state.with_nick(nick).await;
    when_registered(user_state).await
}

pub async fn handle_user_registration(
    user_name: String,
    mode: u8,
    full_user_name: String,
    user_state: &UserState,
) -> Result<Option<String>, IrcError> {
    user_state.with_user(user_name, full_user_name, mode).await;
    when_registered(user_state).await
}

pub async fn when_registered(user_state: &UserState) -> Result<Option<String>, IrcError> {
    if user_state.is_registered().await {
        let user_data = user_state.get_caracs().await;
        let nick = user_data.nick.unwrap();
        let user = user_data.user.unwrap();
        let host = user_data.addr;
        Ok(Some(format!(
            ":localhost {RPL_WELCOME_NB:03} {nick} {RPL_WELCOME_STR} {nick}!{user}@{host}",
        )))
    } else {
        Ok(None)
    }
}
