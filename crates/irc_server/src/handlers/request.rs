use crate::{
    channel_ops::{IrcChannelOperation, IrcInvalidChannelOperation},
    commands::IrcUnknownCommand,
    errors::InternalIrcError,
    pre_registration::IrcCapPreRegistration,
    registration::IrcConnectionRegistration,
    state::ServerState,
    users::UserState,
};

pub async fn handle_request(
    request: &str,
    server: &ServerState,
    user: &UserState,
) -> Result<Option<String>, InternalIrcError> {
    log::info!("{request:?}");

    // 1. Try pre-registration
    match IrcCapPreRegistration::handle_command(request, "*") {
        Ok(ok) => return Ok(ok),
        Err(InternalIrcError::InvalidCommand) => {}
        Err(err) => return Err(err),
    }

    // 2. Try registration
    match IrcConnectionRegistration::handle_command(request, server, user).await {
        Ok(ok) => return Ok(ok),
        Err(InternalIrcError::InvalidCommand) => {}
        Err(err) => return Err(err),
    }

    // 3. Try normal channel operations
    match IrcChannelOperation::handle_command(request) {
        Ok(ok) => return Ok(ok),
        Err(InternalIrcError::InvalidCommand) => {}
        Err(err) => return Err(err),
    }

    // 4. Try invalid-channel ops
    match IrcInvalidChannelOperation::handle_command(request) {
        Ok(ok) => return Ok(ok),
        Err(InternalIrcError::InvalidCommand) => {}
        Err(err) => return Err(err),
    }

    // 5. Fallback to "unknown command"
    IrcUnknownCommand::handle_command(request)
}
