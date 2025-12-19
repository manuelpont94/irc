use crate::{
    errors::InternalIrcError,
    handlers::miscellanneous::IrcUnknownCommand,
    ops::{
        channel::{IrcChannelOperation, IrcInvalidChannelOperation},
        message::IrcMessageSending,
        miscellanneous::IrcMiscellaneousMessages,
        pre_registration::IrcCapPreRegistration,
        registration::IrcConnectionRegistration,
    },
    server_state::ServerState,
    types::ClientId,
    user_state::{UserState, UserStatus},
};

pub async fn handle_request(
    request: &str,
    client_id: ClientId,
    server_state: &ServerState,
    user_state: &UserState,
) -> Result<UserStatus, InternalIrcError> {
    log::info!("{request:?}");

    // -1. Try Message-sending
    match IrcMessageSending::handle_command(request, client_id, server_state, user_state).await {
        Ok(status) => return Ok(status),
        Err(InternalIrcError::InvalidCommand) => {}
        Err(err) => return Err(err),
    }

    // 0. Try pre-registration
    match IrcMiscellaneousMessages::handle_command(request, client_id, user_state).await {
        Ok(status) => return Ok(status),
        Err(InternalIrcError::InvalidCommand) => {}
        Err(err) => return Err(err),
    }

    // 1. Try pre-registration
    match IrcCapPreRegistration::handle_command(request, client_id, server_state, user_state).await
    {
        Ok(status) => return Ok(status),
        Err(InternalIrcError::InvalidCommand) => {}
        Err(err) => return Err(err),
    }

    // 2. Try registration
    match IrcConnectionRegistration::handle_command(request, client_id, server_state, user_state)
        .await
    {
        Ok(status) => return Ok(status),
        Err(InternalIrcError::InvalidCommand) => {}
        Err(err) => return Err(err),
    }

    // 3. Try normal channel operations
    match IrcChannelOperation::handle_command(request, client_id, server_state, user_state).await {
        Ok(status) => return Ok(status),
        Err(InternalIrcError::InvalidCommand) => {}
        Err(err) => return Err(err),
    }

    // 4. Try invalid-channel ops
    match IrcInvalidChannelOperation::handle_command(request, user_state).await {
        Ok(status) => return Ok(status),
        Err(InternalIrcError::InvalidCommand) => {}
        Err(err) => return Err(err),
    }

    // 5. Fallback to "unknown command"
    IrcUnknownCommand::handle_command(request, user_state).await
}
