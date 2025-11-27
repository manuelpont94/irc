use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_till, take_while1},
    character::complete::{char, space1},
    combinator::{opt, recognize, verify},
    multi::{many1, separated_list1},
    sequence::{pair, preceded},
};

use crate::parsers::{
    channel_parser, host_parser, key_parser, nickname_parser, trailing_parser, user_parser,
};

#[derive(Debug, PartialEq)]
pub enum IrcConnectionRegistration {
    PASS(String),              // with few tests
    NICK(String),              // with few tests
    USER(String, u32, String), // with few tests
    OPER(String, String),      // with few tests
    MODE(String, Vec<(char, Vec<char>)>),
    SERVICE(String, String, String, String),
    QUIT(Option<String>),
    SQUIT(String, String),
}

impl IrcConnectionRegistration {
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
fn valid_password_message_parser(input: &str) -> IResult<&str, IrcConnectionRegistration> {
    let mut parser = verify(
        preceded(tag_no_case("PASS "), take_till(|c| c == '\n' || c == '\r')),
        |s: &str| !s.trim().is_empty(),
    );
    let (rem, parsed) = parser.parse(input)?;
    Ok((rem, IrcConnectionRegistration::PASS(parsed.to_string())))
}

//     3.1.2 Nick message

//       Command: NICK
//    Parameters: <nickname>

//    NICK command is used to give user a nickname or change the existing
//    one.
fn valid_nick_message_parser(input: &str) -> IResult<&str, IrcConnectionRegistration> {
    let mut parser = preceded(tag_no_case("NICK "), nickname_parser);
    let (rem, parsed) = parser.parse(input)?;
    Ok((rem, IrcConnectionRegistration::NICK(parsed.to_string())))
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

fn valid_user_message_parser(input: &str) -> IResult<&str, IrcConnectionRegistration> {
    let (rem, (username, mode, _unused, realname)) = ((
        preceded(tag_no_case("USER "), user_parser),
        preceded(tag(" "), user_mode_parser),
        preceded(tag(" "), take_while1(|c: char| !c.is_whitespace())), // <unused> (single token)
        preceded(tag(" :"), trailing_parser),                          // realname until end
    ))
        .parse(input)?;

    Ok((
        rem,
        IrcConnectionRegistration::USER(username.to_string(), mode, realname.to_string()),
    ))
}

// 3.1.4 Oper message

//       Command: OPER
//    Parameters: <name> <password>

//    A normal user uses the OPER command to obtain operator privileges.
//    The combination of <name> and <password> are REQUIRED to gain
//    Operator privileges.  Upon success, the user will receive a MODE
//    message (see section 3.1.5) indicating the new user modes.

fn valid_oper_message_parser(input: &str) -> IResult<&str, IrcConnectionRegistration> {
    let (rem, (name, password)) = ((
        preceded(
            tag_no_case("OPER "),
            take_while1(|c: char| !c.is_whitespace()),
        ),
        verify(
            preceded(space1, take_till(|c| c == '\n' || c == '\r')),
            |s: &str| !s.is_empty(),
        ),
    ))
        .parse(input)?;

    Ok((
        rem,
        IrcConnectionRegistration::OPER(name.to_string(), password.to_string()),
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
fn valid_mode_message_parser(input: &str) -> IResult<&str, IrcConnectionRegistration> {
    let (rem, (nickname, modes)) = (
        preceded(tag_no_case("MODE "), nickname_parser),
        many1(pair(
            alt((char('+'), char('-'))),
            many1(alt((char('i'), char('w'), char('o'), char('O'), char('r')))),
        )),
    )
        .parse(input)?;
    Ok((
        rem,
        IrcConnectionRegistration::MODE(nickname.to_string(), modes),
    ))
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

fn valid_service_message_parser(input: &str) -> IResult<&str, IrcConnectionRegistration> {
    let (rem, (nickname, _reserved, distribution, service_type, _reserved_2, info)) = (
        preceded(tag_no_case("SERVICE "), nickname_parser),
        preceded(tag(" "), take_while1(|c: char| !c.is_whitespace())), // reserved
        preceded(tag(" "), take_while1(|c: char| !c.is_whitespace())), // distribution
        preceded(tag(" "), take_while1(|c: char| !c.is_whitespace())), // type
        preceded(tag(" "), take_while1(|c: char| !c.is_whitespace())), // reserved
        preceded(tag(" :"), trailing_parser),
    )
        .parse(input)?;
    Ok((
        rem,
        IrcConnectionRegistration::SERVICE(
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
fn valid_quit_message_parser(input: &str) -> IResult<&str, IrcConnectionRegistration> {
    let (rem, parsed) = (preceded(
        tag_no_case("QUIT"),
        opt(preceded(tag(" :"), take_till(|c| c == '\n' || c == '\r'))),
    ))
    .parse(input)?;
    let parsed = parsed.map(str::to_string);
    Ok((rem, IrcConnectionRegistration::QUIT(parsed)))
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

fn valid_squit_message_parser(input: &str) -> IResult<&str, IrcConnectionRegistration> {
    let (rem, (server, comment)) = (
        preceded(tag_no_case("SQUIT "), host_parser),
        preceded(tag(" :"), take_till(|c| c == '\n' || c == '\r')),
    )
        .parse(input)?;
    // todo!()
    Ok((
        rem,
        IrcConnectionRegistration::SQUIT(server.to_string(), comment.to_string()),
    ))
}

pub enum IrcChannelOperation {
    LEAVE, // JOIN 0 - should be tested befoire JOIN Channel
    JOIN(Vec<String>, Option<Vec<String>>),
    PART(Vec<String>, Option<String>),
    MODE,
    TOPIC,
    NAMES,
    LIST,
    INVITE,
    KICK,
}
impl IrcChannelOperation {
    pub fn irc_command_parser(input: &str) -> IResult<&str, Self> {
        let mut parser = alt((
            valid_join_channel_parser,
            valid_leave_channel_parser,
            valid_part_channel_parser,
        ));
        parser.parse(input)
    }
}

// 3.2.1 Join message

//       Command: JOIN
//    Parameters: ( <channel> *( "," <channel> ) [ <key> *( "," <key> ) ] )
//                / "0"

//    The JOIN command is used by a user to request to start listening to
//    the specific channel.  Servers MUST be able to parse arguments in the
//    form of a list of target, but SHOULD NOT use lists when sending JOIN
//    messages to clients.

//    Once a user has joined a channel, he receives information about
//    all commands his server receives affecting the channel.  This
//    includes JOIN, MODE, KICK, PART, QUIT and of course PRIVMSG/NOTICE.
//    This allows channel members to keep track of the other channel
//    members, as well as channel modes.

//    If a JOIN is successful, the user receives a JOIN message as
//    confirmation and is then sent the channel's topic (using RPL_TOPIC) and
//    the list of users who are on the channel (using RPL_NAMREPLY), which
//    MUST include the user joining.

//    Note that this message accepts a special argument ("0"), which is
//    a special request to leave all channels the user is currently a member
//    of.  The server will process this message as if the user had sent
//    a PART command (See Section 3.2.2) for each channel he is a member
//    of.

pub fn valid_join_channel_parser(input: &str) -> IResult<&str, IrcChannelOperation> {
    let (rem, (channels, keys)) = preceded(
        tag_no_case("JOIN "),
        (
            (separated_list1(char(','), channel_parser)),
            opt(preceded(tag(" "), separated_list1(char(','), key_parser))),
        ),
    )
    .parse(input)?;
    let channels = channels
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<String>>();
    let keys = keys.map(|v| v.into_iter().map(str::to_string).collect::<Vec<String>>());
    Ok((rem, IrcChannelOperation::JOIN(channels, keys)))
}

// LEAVE Message / JOIN 0
pub fn valid_leave_channel_parser(input: &str) -> IResult<&str, IrcChannelOperation> {
    let (rem, _join0) = recognize(tag_no_case("JOIN 0")).parse(input)?;
    Ok((rem, IrcChannelOperation::LEAVE))
}

// 3.2.2 Part message

//       Command: PART
//    Parameters: <channel> *( "," <channel> ) [ <Part Message> ]

//    The PART command causes the user sending the message to be removed
//    from the list of active members for all given channels listed in the
//    parameter string.  If a "Part Message" is given, this will be sent
//    instead of the default message, the nickname.  This request is always
//    granted by the server.

//    Servers MUST be able to parse arguments in the form of a list of
//    target, but SHOULD NOT use lists when sending PART messages to
//    clients.

pub fn valid_part_channel_parser(input: &str) -> IResult<&str, IrcChannelOperation> {
    let (rem, (channels, optional_message)) = preceded(
        tag_no_case("PART "),
        (
            separated_list1(tag(","), channel_parser),
            opt(preceded(tag(":"), trailing_parser)),
        ),
    )
    .parse(input)?;
    let channels = channels
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<String>>();
    let optional_message = optional_message.map(str::to_string);
    Ok((rem, IrcChannelOperation::PART(channels, optional_message)))
}

pub enum IrcMessageSending {
    PRIVMSG,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password_message_parser() {
        // Example:
        //    PASS secretpasswordhere
        let input = "PASS secretpasswordhere";
        let (rem, password) = valid_password_message_parser(input).unwrap();
        assert!(rem == "");
        assert_eq!(
            password,
            IrcConnectionRegistration::PASS("secretpasswordhere".to_string())
        );
        let input = "PASS  ";
        assert!(valid_password_message_parser(input).is_err(), "no password");
        let input = "PASS";
        assert!(valid_password_message_parser(input).is_err(), "no password");
    }

    #[test]
    fn test_valid_nick_message_parser() {
        // Example:
        // NICK Wiz ; Introducing new nick "Wiz" if session is
        //  still unregistered, or user changing his
        //  nickname to "Wiz"

        let input = "NICK Wiz";
        let (rem, nickname) = valid_nick_message_parser(input).unwrap();
        assert!(rem == "");
        assert_eq!(nickname, IrcConnectionRegistration::NICK("Wiz".to_string()));
        let input = "NICK  ";
        assert!(valid_nick_message_parser(input).is_err(), "no nickname");
        let input = "NICK";
        assert!(valid_nick_message_parser(input).is_err(), "no nickname");
    }

    #[test]
    fn test_valid_user_message_parser() {
        // Example:
        // USER guest 0 * :Ronnie Reagan ; User registering themselves with a
        // username of "guest" and real name
        // "Ronnie Reagan".

        // USER guest 8 * :Ronnie Reagan ; User registering themselves with a
        // username of "guest" and real name
        // "Ronnie Reagan", and asking to be set
        // invisible.

        let input = "USER guest 0 * :Ronnie Reagan";
        let (rem, nickname) = valid_user_message_parser(input).unwrap();
        assert!(rem == "");
        assert_eq!(
            nickname,
            IrcConnectionRegistration::USER(
                "guest".to_string(),
                0_u32,
                "Ronnie Reagan".to_string()
            )
        );
        let input = "USER guest 8 * :Ronnie Reagan";
        let (rem, nickname) = valid_user_message_parser(input).unwrap();
        assert!(rem == "");
        assert_eq!(
            nickname,
            IrcConnectionRegistration::USER(
                "guest".to_string(),
                8_u32,
                "Ronnie Reagan".to_string()
            )
        );
        let input = "USER guest * :Ronnie Reagan";
        assert!(valid_user_message_parser(input).is_err(), "missing mode");
    }

    #[test]
    fn test_valid_oper_message_parser() {
        // Example:
        //    OPER foo bar ; Attempt to register as an operator
        //    using a username of "foo" and "bar"
        //    as the password.
        let input = "OPER foo bar";
        let (rem, nickname) = valid_oper_message_parser(input).unwrap();
        assert!(rem == "");
        assert_eq!(
            nickname,
            IrcConnectionRegistration::OPER("foo".to_string(), "bar".to_string())
        );
        let input = "OPER foo ";
        // dbg!(valid_oper_message_parser(input));
        assert!(valid_oper_message_parser(input).is_err(), "no password");
        let input = "OPER";
        assert!(
            valid_oper_message_parser(input).is_err(),
            "no user / no password"
        );
    }

    #[test]
    fn test_valid_mode_message_parser() {
        // Example:

        let input = "NICK Wiz";
        let (rem, nickname) = valid_nick_message_parser(input).unwrap();
        assert!(rem == "");
        assert_eq!(nickname, IrcConnectionRegistration::NICK("Wiz".to_string()));
        let input = "NICK  ";
        assert!(valid_nick_message_parser(input).is_err(), "no nickname");
        let input = "NICK";
        assert!(valid_nick_message_parser(input).is_err(), "no nickname");
    }
}

// ## Valid Examples
// ```
// irc.example.com          ✓ Three shortnames separated by dots
// tolsun.oulu.fi           ✓ Valid Finnish server
// cm22.eng.umd.edu         ✓ Starts with digits, has hyphens not at end
// localhost                ✓ Single shortname
// 127.0.0.1                ✓ IP address format (shortnames starting with digits)
// irc-server.net           ✓ Hyphen in middle
// a                        ✓ Single character
// 123.456.789.0            ✓ All digits (numeric format)
// test-123.example.org     ✓ Mix of letters, digits, hyphens
// ```

// ## Invalid Examples
// ```
// -irc.example.com         ✗ Shortname starts with hyphen
// irc-.example.com         ✗ Shortname ends with hyphen
// irc..example.com         ✗ Empty shortname (double dot)
// irc_server.com           ✗ Underscore not allowed
// irc server.com           ✗ Space not allowed
// irc.example.com.         ✗ Trailing dot (empty final shortname)
// .irc.example.com         ✗ Leading dot (empty first shortname)
// irc@example.com          ✗ '@' not allowed
// irc.exam ple.com         ✗ Space in shortname
// [irc].example.com        ✗ Brackets not allowed
// this-is-a-very-long-server-name-that-exceeds-sixty-three-characters-limit.com
//                          ✗ Exceeds 63 character maximum
