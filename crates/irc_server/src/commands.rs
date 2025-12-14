use std::fmt::Display;

use nom::{IResult, Parser, bytes::complete::take_till};

use crate::{
    constants::{ERR_UNKNOWNCOMMAND_NB, ERR_UNKNOWNCOMMAND_STR},
    errors::InternalIrcError,
};

pub enum IrcServiceQueryCommands {
    SERVLIST,
    SQUERY,
    WHO,
    WHOIS,
    WHOWAS,
}

pub enum IrcMiscellaneousMessages {
    KILL,
    PING,
    PONG,
    ERROR,
}

pub enum IrcOptionalFeatures {
    AWAY,
    REHASH,
    DIE,
    RESTART,
    SUMMON,
    USERS,
    WALLOPS,
    USERHOST,
    ISON,
}

pub struct IrcUnknownCommand(String);
impl IrcUnknownCommand {
    pub fn irc_command_parser(input: &str) -> IResult<&str, Self> {
        unknwon_command_parser(input)
    }
    pub fn handle_command(command: &str) -> Result<Option<String>, InternalIrcError> {
        match IrcUnknownCommand::irc_command_parser(command) {
            Ok((_rem, valid_commmand)) => Ok(Some(format!("{valid_commmand}"))),
            Err(_) => Err(InternalIrcError::ParsingError(format!(
                "error during parsing unknown command: '{command}'"
            ))),
        }
    }
}
impl Display for IrcUnknownCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn unknwon_command_parser(input: &str) -> IResult<&str, IrcUnknownCommand> {
    let (rem, command) = take_till(|c| c == ' ').parse(input)?;
    Ok((
        rem,
        IrcUnknownCommand(format!(
            "{ERR_UNKNOWNCOMMAND_NB} * {command} :{ERR_UNKNOWNCOMMAND_STR}",
        )),
    ))
}
