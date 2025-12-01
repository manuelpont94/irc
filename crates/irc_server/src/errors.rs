use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum IrcError {
    /// Erreur générique de parsing.
    #[error("Parsing error: '{0}'")]
    ParsingError(String),

    #[error("CAP Pre-Registration error: '{0}'")]
    IrcCapPreRegistration(String),
}
