use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    multi::many1,
    sequence::preceded,
};

use crate::{
    errors::InternalIrcError,
    handlers::miscellanneous::handle_ping,
    ops::parsers::host_parser,
    user_state::{UserState, UserStatus},
};
pub enum IrcMiscellaneousMessages {
    KILL,
    PING(Vec<String>),
    PONG,
    ERROR,
}
impl IrcMiscellaneousMessages {
    pub fn irc_command_parser(input: &str) -> IResult<&str, Self> {
        let mut parser = alt((valid_ping_parser,));
        parser.parse(input)
    }

    pub async fn handle_command(
        command: &str,
        _client_id: usize,
        user_state: &UserState,
    ) -> Result<UserStatus, InternalIrcError> {
        match IrcMiscellaneousMessages::irc_command_parser(command) {
            Ok((_rem, valid_commmand)) => match valid_commmand {
                IrcMiscellaneousMessages::PING(server) => handle_ping(server, user_state).await,
                _ => todo!(),
            },
            Err(_e) => Err(InternalIrcError::InvalidCommand),
        }
    }
}

pub fn valid_ping_parser(input: &str) -> IResult<&str, IrcMiscellaneousMessages> {
    let (rem, servers) =
        preceded(tag_no_case("PING"), many1(preceded(tag(" "), host_parser))).parse(input)?;
    let servers = servers
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    Ok((rem, IrcMiscellaneousMessages::PING(servers)))
}
