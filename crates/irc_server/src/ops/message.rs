use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    sequence::preceded,
};

use crate::{
    errors::InternalIrcError,
    handlers::messages::handle_privmsg,
    ops::parsers::{msgtarget_parser, targetmask_parser, trailing_parser},
    server_state::ServerState,
    user_state::{UserState, UserStatus},
};
use std::str::FromStr;
use thiserror::Error;

// https://www.rfc-editor.org/rfc/rfc2812
// 2.3.1 Message format in Augmented BNF

//    The protocol messages must be extracted from the contiguous stream of
//    octets.  The current solution is to designate two characters, CR and
//    LF, as message separators.  Empty messages are silently ignored,
//    which permits use of the sequence CR-LF between messages without
//    extra problems.

//    The extracted message is parsed into the components <prefix>,
//    <command> and list of parameters (<params>).

//     The Augmented BNF representation for this is:

//     message    =  [ ":" prefix SPACE ] command [ params ] crlf
//     prefix     =  servername / ( nickname [ [ "!" user ] "@" host ] )
//     command    =  1*letter / 3digit
//     params     =  *14( SPACE middle ) [ SPACE ":" trailing ]
//                =/ 14( SPACE middle ) [ SPACE [ ":" ] trailing ]

//     nospcrlfcl =  %x01-09 / %x0B-0C / %x0E-1F / %x21-39 / %x3B-FF
//                     ; any octet except NUL, CR, LF, " " and ":"
//     middle     =  nospcrlfcl *( ":" / nospcrlfcl )
//     trailing   =  *( ":" / " " / nospcrlfcl )

//     SPACE      =  %x20        ; space character
//     crlf       =  %x0D %x0A   ; "carriage return" "linefeed"

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("parsing error {0}")]
    ParseError(&'static str),
}

pub struct Prefix {}
impl Prefix {
    //     prefix = servername / ( nickname [ [ "!" user ] "@" host ] )
    // ```

    // **Signification :** Le préfixe peut être :
    // - Soit un nom de serveur : `irc.server.com`
    // - Soit un utilisateur avec différents formats :
    //   - `nickname` seul : `alice`
    //   - `nickname@host` : `alice@192.168.1.1`
    //   - `nickname!user@host` : `alice!alice@host.com`
    pub fn parse(_input: &str) -> IResult<&str, &str> {
        todo!()
    }
}
pub struct Command {}

pub struct Params {}

pub struct Message {
    _prefix: Option<Prefix>,
    _command: Command,
    _params: Option<Params>,
}
impl FromStr for Message {
    type Err = MessageError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

pub enum IrcMessageSending {
    PRIVMSG(String, String),
    NOTICE,
    MOTD,
    VERSION,
    STATS,
    LINKS,
    TIME,
    CONNECT,
    TRACE,
    ADMIN,
    INFO,
}

impl IrcMessageSending {
    pub fn irc_command_parser(input: &str) -> IResult<&str, Self> {
        let mut parser = alt((valid_privmsg_message_parser,));
        parser.parse(input)
    }

    pub async fn handle_command(
        command: &str,
        _client_id: usize,
        server_state: &ServerState,
        user_state: &UserState,
    ) -> Result<UserStatus, InternalIrcError> {
        match IrcMessageSending::irc_command_parser(command) {
            Ok((_rem, valid_commmand)) => match valid_commmand {
                IrcMessageSending::PRIVMSG(msgtarget, msg) => {
                    handle_privmsg(msgtarget, msg, server_state, user_state).await
                }
                _ => todo!(),
            },
            Err(_e) => Err(InternalIrcError::InvalidCommand),
        }
    }
}

// 3.3.1 Private messages

//       Command: PRIVMSG
//    Parameters: <msgtarget> <text to be sent>

//    PRIVMSG is used to send private messages between users, as well as to
//    send messages to channels.  <msgtarget> is usually the nickname of
//    the recipient of the message, or a channel name.

//    The <msgtarget> parameter may also be a host mask (#<mask>) or server
//    mask ($<mask>).  In both cases the server will only send the PRIVMSG
//    to those who have a server or host matching the mask.  The mask MUST
//    have at least 1 (one) "." in it and no wildcards following the last
//    ".".  This requirement exists to prevent people sending messages to
//    "#*" or "$*", which would broadcast to all users.  Wildcards are the
//    '*' and '?'  characters.  This extension to the PRIVMSG command is
//    only available to operators.

fn valid_privmsg_message_parser(input: &str) -> IResult<&str, IrcMessageSending> {
    let (rem, (mstarget, text_to_be_sent)) = (preceded(
        tag_no_case("PRIVMSG "),
        (
            alt((msgtarget_parser, targetmask_parser)),
            preceded(tag(" :"), trailing_parser),
        ),
    ))
    .parse(input)?;
    let mstarget = mstarget.to_owned();
    let text_to_be_sent = text_to_be_sent.to_owned();
    Ok((rem, IrcMessageSending::PRIVMSG(mstarget, text_to_be_sent)))
}
