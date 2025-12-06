use std::fmt::Display;

use crate::{
    constants::{ERR_NEEDMOREPARAMS_NB, ERR_NEEDMOREPARAMS_STR},
    errors::IrcError,
    parsers::{
        channel_parser, key_parser, nickname_parser, target_parser, trailing_parser, user_parser,
        wildcards_parser,
    },
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{char, satisfy},
    combinator::{opt, recognize},
    multi::{many1, separated_list1},
    sequence::{pair, preceded},
};

pub enum IrcChannelOperation {
    LEAVE, // JOIN 0 - should be tested befoire JOIN Channel
    JOIN(Vec<String>, Option<Vec<String>>),
    PART(Vec<String>, Option<String>),
    MODE(String, Vec<(char, Vec<char>)>),
    TOPIC(String, Option<String>),
    NAMES(Option<Vec<String>>, Option<String>),
    LIST(Option<Vec<String>>, Option<String>),
    INVITE(String, String),
    KICK(Vec<String>, Vec<String>, Option<String>),
}
impl IrcChannelOperation {
    pub fn irc_command_parser(input: &str) -> IResult<&str, Self> {
        let mut parser = alt((
            valid_join_channel_parser,
            valid_leave_channel_parser,
            valid_part_channel_parser,
            valid_mode_channel_parser,
            valid_topic_channel_parser,
            valid_names_channel_parser,
            valid_list_channel_parser,
            valid_invite_channel_parser,
            valid_kick_channel_parser,
        ));
        parser.parse(input)
    }

    pub fn handle_command(command: &str) -> Result<Option<String>, IrcError> {
        match IrcChannelOperation::irc_command_parser(command) {
            Ok(valid_commmand) => todo!(),
            Err(e) => Err(IrcError::IrcChannelOperations(format!("{}", e.to_owned()))),
        }
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

// 3.2.3 Channel mode message

//       Command: MODE
//    Parameters: <channel> *( ( "-" / "+" ) *<modes> *<modeparams> )

//    The MODE command is provided so that users may query and change the
//    characteristics of a channel.  For more details on available modes
//    and their uses, see "Internet Relay Chat: Channel Management" [IRC-
//    CHAN].  Note that there is a maximum limit of three (3) changes per
//    command for modes that take a parameter.
//    https://www.rfc-editor.org/rfc/rfc2811
//    The various modes available for channels are as follows:

//         O - give "channel creator" status;
//         o - give/take channel operator privilege;
//         v - give/take the voice privilege;

//         a - toggle the anonymous channel flag;
//         i - toggle the invite-only channel flag;
//         m - toggle the moderated channel;
//         n - toggle the no messages to channel from clients on the
//             outside;
//         q - toggle the quiet channel flag;
//         p - toggle the private channel flag;
//         s - toggle the secret channel flag;
//         r - toggle the server reop channel flag;
//         t - toggle the topic settable by channel operator only flag;

//         k - set/remove the channel key (password);
//         l - set/remove the user limit to channel;

//         b - set/remove ban mask to keep users out;
//         e - set/remove an exception mask to override a ban mask;
//         I - set/remove an invitation mask to automatically override
//             the invite-only flag;

fn is_channel_mode(c: char) -> bool {
    matches!(
        c,
        'O' | 'o'
            | 'v'
            | 'a'
            | 'i'
            | 'm'
            | 'n'
            | 'q'
            | 'p'
            | 's'
            | 'r'
            | 't'
            | 'k'
            | 'l'
            | 'b'
            | 'e'
            | 'I'
    )
}

fn valid_mode_channel_parser(input: &str) -> IResult<&str, IrcChannelOperation> {
    let (rem, (channel, modes)) = (
        preceded(tag_no_case("MODE "), channel_parser),
        preceded(
            tag(" "),
            many1(pair(
                alt((char('+'), char('-'))),
                many1(satisfy(is_channel_mode)),
            )),
        ),
    )
        .parse(input)?;
    Ok((rem, IrcChannelOperation::MODE(channel.to_owned(), modes)))
}

// 3.2.4 Topic message

//       Command: TOPIC
//    Parameters: <channel> [ <topic> ]

//    The TOPIC command is used to change or view the topic of a channel.
//    The topic for channel <channel> is returned if there is no <topic>
//    given.  If the <topic> parameter is present, the topic for that
//    channel will be changed, if this action is allowed for the user
//    requesting it.  If the <topic> parameter is an empty string, the
//    topic for that channel will be removed.

fn valid_topic_channel_parser(input: &str) -> IResult<&str, IrcChannelOperation> {
    let (rem, (channel, topic)) = (
        preceded(tag_no_case("TOPIC "), channel_parser),
        opt(preceded(tag(" "), trailing_parser)),
    )
        .parse(input)?;
    let topic = topic.map(str::to_owned);
    Ok((rem, IrcChannelOperation::TOPIC(channel.to_owned(), topic)))
}

// 3.2.5 Names message

//       Command: NAMES
//    Parameters: [ <channel> *( "," <channel> ) [ <target> ] ]

//    By using the NAMES command, a user can list all nicknames that are
//    visible to him. For more details on what is visible and what is not,
//    see "Internet Relay Chat: Channel Management" [IRC-CHAN].  The
//    <channel> parameter specifies which channel(s) to return information
//    about.  There is no error reply for bad channel names.

//    If no <channel> parameter is given, a list of all channels and their
//    occupants is returned.  At the end of this list, a list of users who
//    are visible but either not on any channel or not on a visible channel
//    are listed as being on `channel' "*".

//    If the <target> parameter is specified, the request is forwarded to
//    that server which will generate the reply.

//    Wildcards are allowed in the <target> parameter.

fn valid_names_channel_parser(input: &str) -> IResult<&str, IrcChannelOperation> {
    let (rem, (_names, params)) = ((
        tag_no_case("NAMES"),
        opt(preceded(
            tag(" "),
            (
                separated_list1(tag(","), channel_parser),
                opt(preceded(tag(" "), alt((target_parser, wildcards_parser)))),
            ),
        )),
    ))
        .parse(input)?;
    let channels = params
        .clone()
        .map(|(ch, _)| ch.into_iter().map(str::to_owned).collect::<Vec<String>>());
    let target = params.map(|(_, targ)| targ.map(str::to_owned)).flatten();
    // let topic = topic.map(str::to_owned);
    Ok((rem, IrcChannelOperation::NAMES(channels, target)))
}

// 3.2.6 List message

//       Command: LIST
//    Parameters: [ <channel> *( "," <channel> ) [ <target> ] ]

//    The list command is used to list channels and their topics.  If the
//    <channel> parameter is used, only the status of that channel is
//    displayed.

//    If the <target> parameter is specified, the request is forwarded to
//    that server which will generate the reply.

//    Wildcards are allowed in the <target> parameter.

fn valid_list_channel_parser(input: &str) -> IResult<&str, IrcChannelOperation> {
    let (rem, (_list, params)) = ((
        tag_no_case("LIST"),
        opt(preceded(
            tag(" "),
            (
                separated_list1(tag(","), channel_parser),
                opt(preceded(tag(" "), alt((target_parser, wildcards_parser)))),
            ),
        )),
    ))
        .parse(input)?;
    let channels = params
        .clone()
        .map(|(ch, _)| ch.into_iter().map(str::to_owned).collect::<Vec<String>>());
    let target = params.map(|(_, targ)| targ.map(str::to_owned)).flatten();
    Ok((rem, IrcChannelOperation::LIST(channels, target)))
}

// 3.2.7 Invite message

//       Command: INVITE
//    Parameters: <nickname> <channel>

//    The INVITE command is used to invite a user to a channel.  The
//    parameter <nickname> is the nickname of the person to be invited to
//    the target channel <channel>.  There is no requirement that the
//    channel the target user is being invited to must exist or be a valid
//    channel.  However, if the channel exists, only members of the channel
//    are allowed to invite other users.  When the channel has invite-only
//    flag set, only channel operators may issue INVITE command.

//    Only the user inviting and the user being invited will receive
//    notification of the invitation.  Other channel members are not
//    notified.  (This is unlike the MODE changes, and is occasionally the
//    source of trouble for users.)

fn valid_invite_channel_parser(input: &str) -> IResult<&str, IrcChannelOperation> {
    let (rem, (nickname, channel)) =
        (preceded(tag_no_case("INVITE "), (nickname_parser, channel_parser))).parse(input)?;
    Ok((
        rem,
        IrcChannelOperation::INVITE(nickname.to_owned(), channel.to_owned()),
    ))
}

// 3.2.8 Kick command

//       Command: KICK
//    Parameters: <channel> *( "," <channel> ) <user> *( "," <user> )
//                [<comment>]

//    The KICK command can be used to request the forced removal of a user
//    from a channel.  It causes the <user> to PART from the <channel> by
//    force.  For the message to be syntactically correct, there MUST be
//    either one channel parameter and multiple user parameter, or as many
//    channel parameters as there are user parameters.  If a "comment" is
//    given, this will be sent instead of the default message, the nickname
//    of the user issuing the KICK.

//    The server MUST NOT send KICK messages with multiple channels or
//    users to clients.  This is necessarily to maintain backward
//    compatibility with old client software.
fn valid_kick_channel_parser(input: &str) -> IResult<&str, IrcChannelOperation> {
    let (rem, (channels, users, comment)) = (preceded(
        tag_no_case("KICK "),
        (
            separated_list1(tag(","), channel_parser),
            (preceded(tag(" "), separated_list1(tag(","), user_parser))),
            opt(preceded(tag(" :"), trailing_parser)),
        ),
    ))
    .parse(input)?;
    let channels = channels
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<String>>();
    let users = users
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<String>>();
    let comment = comment.map(str::to_owned);
    Ok((rem, IrcChannelOperation::KICK(channels, users, comment)))
}

#[derive(Debug)]
pub struct IrcInvalidChannelOperation(String);
impl IrcInvalidChannelOperation {
    pub fn irc_command_parser(input: &str) -> IResult<&str, Self> {
        let mut parser = alt((
            invalid_join_channel_parser,
            invalid_join_channel_parser, // valid_leave_channel_parser,
        ));
        parser.parse(input)
    }
    pub fn handle_command(command: &str) -> Result<Option<String>, IrcError> {
        match IrcInvalidChannelOperation::irc_command_parser(command) {
            Ok((_rem, valid_commmand)) => Ok(Some(format!("{}", valid_commmand))),
            Err(e) => Err(IrcError::InvalidCommand),
        }
    }
}
impl Display for IrcInvalidChannelOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn invalid_join_channel_parser(input: &str) -> IResult<&str, IrcInvalidChannelOperation> {
    let (rem, _) = tag_no_case("JOIN").parse(input)?;
    Ok((
        rem,
        IrcInvalidChannelOperation(format!(
            "{} JOIN :{}",
            ERR_NEEDMOREPARAMS_NB, ERR_NEEDMOREPARAMS_STR
        )),
    ))
}
