use crate::{errors::InternalIrcError, server_state::ServerState, user_state::UserState};

pub async fn handle_join_channel(
    channels: Vec<String>,
    keys: Option<Vec<String>>,
    server_state: &ServerState,
    user_state: &UserState,
) -> Result<Option<String>, InternalIrcError> {
    if !user_state.is_registered().await {
        todo!()
    }
    for channel_name in channels {
        if server_state.channels_exists(channel_name) {
            todo!()
            // add user to channel members
        } else {
            todo!()
            // create the channel
        }

        // broadcast
    }

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
    todo!()
}
