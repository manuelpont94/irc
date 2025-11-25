use std::path::PrefixComponent;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_till, take_while1},
    character::complete::{char, line_ending, space1},
    combinator::{opt, recognize, verify},
    multi::many1,
    sequence::{pair, preceded, terminated},
};

use crate::parsers::{host_parser, nickname_parser, user_parser};

pub enum IrcCommand {
    PASS(String),
    NICK(String),
    USER(String, u32, String),
    OPER(String, String),
    MODE(String, Vec<(char, Vec<char>)>),
    SERVICE(String, String, String, String),
    QUIT(Option<String>),
    SQUIT(String, String),
}

impl IrcCommand {
    pub fn irc_command_parser(input: &str) -> IResult<&str, Self> {
        let mut parser = alt((
            valid_password_message_parser,
            valid_nick_message_parser,
            valid_user_message_parser,
            valid_oper_message_parser,
            valid_mode_message_parser,
            valid_service_message_parser,
            valid_quit_message_parser,
            valid_squit_message_parser,
        ));
        parser.parse(input)
    }
}

//     3.1.1 Password message

//       Command: PASS
//    Parameters: <password>

//    The PASS command is used to set a 'connection password'.  The
//    optional password can and MUST be set before any attempt to register
//    the connection is made.  Currently this requires that user send a
//    PASS command before sending the NICK/USER combination.
fn valid_password_message_parser(input: &str) -> IResult<&str, IrcCommand> {
    let mut parser = verify(
        preceded(tag_no_case("PASS "), take_till(|c| c == '\n' || c == '\r')),
        |s: &str| !s.trim().is_empty(),
    );
    let (rem, parsed) = parser.parse(input)?;
    Ok((rem, IrcCommand::PASS(parsed.to_string())))
}

//     3.1.2 Nick message

//       Command: NICK
//    Parameters: <nickname>

//    NICK command is used to give user a nickname or change the existing
//    one.
fn valid_nick_message_parser(input: &str) -> IResult<&str, IrcCommand> {
    let mut parser = (preceded(
        tag_no_case("NICK "),
        terminated(nickname_parser, line_ending),
    ));
    let (rem, parsed) = parser.parse(input)?;
    Ok((rem, IrcCommand::NICK(parsed.to_string())))
}

// 3.1.3 User message

//       Command: USER
//    Parameters: <user> <mode> <unused> <realname>

//    The USER command is used at the beginning of connection to specify
//    the username, hostname and realname of a new user.

//    The <mode> parameter should be a numeric, and can be used to
//    automatically set user modes when registering with the server.  This
//    parameter is a bitmask, with only 2 bits having any signification: if
//    the bit 2 is set, the user mode 'w' will be set and if the bit 3 is
//    set, the user mode 'i' will be set.  (See Section 3.1.5 "User
//    Modes").

//    The <realname> may contain space characters.
fn user_mode_parser(input: &str) -> IResult<&str, u32> {
    // take digits
    let (rem, digits) = recognize(take_while1(|c: char| c.is_ascii_digit())).parse(input)?;

    // convert to integer
    let mode = digits.parse().map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(digits, nom::error::ErrorKind::Digit))
    })?;

    Ok((rem, mode))
}

fn valid_user_message_parser(input: &str) -> IResult<&str, IrcCommand> {
    let (rem, (username, mode, _unused, realname)) = ((
        preceded(tag_no_case("USER "), user_parser),
        preceded(space1, user_mode_parser),
        preceded(space1, take_while1(|c: char| !c.is_whitespace())), // <unused> (single token)
        preceded(space1, preceded(tag(":"), take_till(|_| false))),  // realname until end
    ))
        .parse(input)?;

    Ok((
        rem,
        IrcCommand::USER(username.to_string(), mode, realname.to_string()),
    ))
}

// 3.1.4 Oper message

//       Command: OPER
//    Parameters: <name> <password>

//    A normal user uses the OPER command to obtain operator privileges.
//    The combination of <name> and <password> are REQUIRED to gain
//    Operator privileges.  Upon success, the user will receive a MODE
//    message (see section 3.1.5) indicating the new user modes.

fn valid_oper_message_parser(input: &str) -> IResult<&str, IrcCommand> {
    let (rem, (name, password)) = ((
        preceded(
            tag_no_case("OPER "),
            take_while1(|c: char| !c.is_whitespace()),
        ),
        preceded(space1, take_till(|c| c == '\n' || c == '\r')),
    ))
        .parse(input)?;

    Ok((
        rem,
        IrcCommand::OPER(name.to_string(), password.to_string()),
    ))
}

// 3.1.5 User mode message

//       Command: MODE
//    Parameters: <nickname>
//                *( ( "+" / "-" ) *( "i" / "w" / "o" / "O" / "r" ) )

//    The user MODE's are typically changes which affect either how the
//    client is seen by others or what 'extra' messages the client is sent.

//    A user MODE command MUST only be accepted if both the sender of the
//    message and the nickname given as a parameter are both the same.  If
//    no other parameter is given, then the server will return the current
//    settings for the nick.

//       The available modes are as follows:

//            a - user is flagged as away;
//            i - marks a users as invisible;
//            w - user receives wallops;
//            r - restricted user connection;
//            o - operator flag;
//            O - local operator flag;
//            s - marks a user for receipt of server notices.

//    Additional modes may be available later on.

//    The flag 'a' SHALL NOT be toggled by the user using the MODE command,
//    instead use of the AWAY command is REQUIRED.

//    If a user attempts to make themselves an operator using the "+o" or
//    "+O" flag, the attempt SHOULD be ignored as users could bypass the
//    authentication mechanisms of the OPER command.  There is no
//    restriction, however, on anyone `deopping' themselves (using "-o" or
//    "-O").

//    On the other hand, if a user attempts to make themselves unrestricted
//    using the "-r" flag, the attempt SHOULD be ignored.  There is no
//    restriction, however, on anyone `deopping' themselves (using "+r").
//    This flag is typically set by the server upon connection for
//    administrative reasons.  While the restrictions imposed are left up
//    to the implementation, it is typical that a restricted user not be
//    allowed to change nicknames, nor make use of the channel operator
//    status on channels.

//    The flag 's' is obsolete but MAY still be used.
fn valid_mode_message_parser(input: &str) -> IResult<&str, IrcCommand> {
    let (rem, (nickname, modes)) = (
        preceded(tag_no_case("MODE "), nickname_parser),
        many1(pair(
            alt((char('+'), char('-'))),
            many1(alt((char('i'), char('w'), char('o'), char('O'), char('r')))),
        )),
    )
        .parse(input)?;
    Ok((rem, IrcCommand::MODE(nickname.to_string(), modes)))
}

// 3.1.6 Service message

//       Command: SERVICE
//    Parameters: <nickname> <reserved> <distribution> <type>
//                <reserved> <info>

//    The SERVICE command to register a new service.  Command parameters
//    specify the service nickname, distribution, type and info of a new
//    service.

// Kalt                         Informational                     [Page 13]

// RFC 2812          Internet Relay Chat: Client Protocol        April 2000

//    The <distribution> parameter is used to specify the visibility of a
//    service.  The service may only be known to servers which have a name
//    matching the distribution.  For a matching server to have knowledge
//    of the service, the network path between that server and the server
//    on which the service is connected MUST be composed of servers which
//    names all match the mask.

//    The <type> parameter is currently reserved for future usage.

fn valid_service_message_parser(input: &str) -> IResult<&str, IrcCommand> {
    let (rem, (nickname, _reserved, distribution, service_type, _reserved_2, info)) = (
        preceded(tag_no_case("SERVICE "), nickname_parser),
        preceded(tag(" "), take_while1(|c: char| !c.is_whitespace())), // reserved
        preceded(tag(" "), take_while1(|c: char| !c.is_whitespace())), // distribution
        preceded(tag(" "), take_while1(|c: char| !c.is_whitespace())), // type
        preceded(tag(" "), take_while1(|c: char| !c.is_whitespace())), // reserved
        preceded(tag(" :"), take_till(|c: char| c == '\n' || c == '\r')),
    )
        .parse(input)?;
    Ok((
        rem,
        IrcCommand::SERVICE(
            nickname.to_string(),
            distribution.to_string(),
            service_type.to_string(),
            info.to_string(),
        ),
    ))
}
// 3.1.7 Quit

//       Command: QUIT
//    Parameters: [ <Quit Message> ]

//    A client session is terminated with a quit message.  The server
//    acknowledges this by sending an ERROR message to the client.
// TODO TEST avec recognize et None
fn valid_quit_message_parser(input: &str) -> IResult<&str, IrcCommand> {
    let (rem, parsed) = (preceded(
        tag_no_case("QUIT"),
        opt(preceded(tag(" :"), take_till(|c| c == '\n' || c == '\r'))),
    ))
    .parse(input)?;
    let parsed = parsed.map(str::to_string);
    Ok((rem, IrcCommand::QUIT(parsed)))
}

// 3.1.8 Squit

//       Command: SQUIT
//    Parameters: <server> <comment>

//    The SQUIT command is available only to operators.  It is used to
//    disconnect server links.  Also servers can generate SQUIT messages on
//    error conditions.  A SQUIT message may also target a remote server
//    connection.  In this case, the SQUIT message will simply be sent to
//    the remote server without affecting the servers in between the
//    operator and the remote server.

//    The <comment> SHOULD be supplied by all operators who execute a SQUIT
//    for a remote server.  The server ordered to disconnect its peer
//    generates a WALLOPS message with <comment> included, so that other
//    users may be aware of the reason of this action.

fn valid_squit_message_parser(input: &str) -> IResult<&str, IrcCommand> {
    let (rem, (server, comment)) = (
        preceded(tag_no_case("SQUIT "), host_parser),
        preceded(tag(" :"), take_till(|c| c == '\n' || c == '\r')),
    )
        .parse(input)?;
    // todo!()
    Ok((
        rem,
        IrcCommand::SQUIT(server.to_string(), comment.to_string()),
    ))
}
