use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug, Clone)]
pub enum InternalIrcError {
    /// Erreur générique de parsing.
    #[error("Parsing error: '{0}'")]
    ParsingError(String),

    #[error("CAP Pre-Registration error: '{0}'")]
    CapPreRegistration(String),

    #[error("Connection Registration error: '{0}'")]
    ConnectionRegistrationError(String),

    #[error("Channel Operations error: '{0}'")]
    ChannelOperations(String),

    #[error("Invalid Command")]
    InvalidCommand,

    #[error("User State error: '{0}'")]
    UserStateError(&'static str),

    #[error("Server State error: '{0}'")]
    ServerStateError(&'static str),
}
