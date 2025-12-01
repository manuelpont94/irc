use std::{fmt::Display, sync::Arc};

use dashmap::DashMap;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_till, take_till1, take_while1},
    character::complete::{char, satisfy, space1},
    combinator::{Opt, opt, recognize, verify},
    multi::{many1, separated_list1},
    sequence::{pair, preceded},
};

use crate::{
    channels_models::{ChannelName, IrcChannel},
    constants::{
        ERR_NEEDMOREPARAMS_NB, ERR_NEEDMOREPARAMS_STR, ERR_UNKNOWNCOMMAND_NB,
        ERR_UNKNOWNCOMMAND_STR,
    },
    errors::IrcError,
    parsers::{
        channel_parser, host_parser, key_parser, msgtarget_parser, nickname_parser, target_parser,
        targetmask_parser, trailing_parser, user_parser, wildcards_parser,
    },
    pre_registration::IRC_SERVER_CAP_ECHO_MESSAGE,
    users::{User, UserId, UserState},
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
    pub fn handle_command(command: &str) -> Result<String, &str> {
        match IrcUnknownCommand::irc_command_parser(command) {
            Ok((_rem, valid_commmand)) => Ok(format!("{}", valid_commmand)),
            Err(e) => Err("{e}"),
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
            "{} * {} :{}",
            ERR_UNKNOWNCOMMAND_NB, command, ERR_UNKNOWNCOMMAND_STR
        )),
    ))
}
