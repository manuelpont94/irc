use crate::{
    errors::InternalIrcError,
    message_models::IrcMessage,
    replies::IrcReply,
    user_state::{UserState, UserStatus},
};
use nom::{IResult, Parser, bytes::complete::take_till};

// 3.7.2 Ping message

//       Command: PING
//    Parameters: <server1> [ <server2> ]

//    The PING command is used to test the presence of an active client or
//    server at the other end of the connection.  Servers send a PING
//    message at regular intervals if no other activity detected coming
//    from a connection.  If a connection fails to respond to a PING
//    message within a set amount of time, that connection is closed.  A
//    PING message MAY be sent even if the connection is active.

//    When a PING message is received, the appropriate PONG message MUST be
//    sent as reply to <server1> (server which sent the PING message out)
//    as soon as possible.  If the <server2> parameter is specified, it
//    represents the target of the ping, and the message gets forwarded
//    there.

//    Numeric Replies:

//            ERR_NOORIGIN                  ERR_NOSUCHSERVER

//    Examples:

//    PING tolsun.oulu.fi             ; Command to send a PING message to
//                                    server

//    PING WiZ tolsun.oulu.fi         ; Command from WiZ to send a PING
//                                    message to server "tolsun.oulu.fi"

//    PING :irc.funet.fi              ; Ping message sent by server
//                                    "irc.funet.fi"

pub async fn handle_ping(
    server: Vec<String>,
    user_state: &UserState,
) -> Result<UserStatus, InternalIrcError> {
    let irc_reply = IrcReply::Pong {
        destination: &server[0],
    };
    let pong_message = IrcMessage::new(irc_reply.format());
    let _ = user_state.tx_outbound.send(pong_message).await;
    Ok(UserStatus::Active)
}

pub struct IrcUnknownCommand(String);
impl IrcUnknownCommand {
    pub fn irc_command_parser(input: &str) -> IResult<&str, Self> {
        unknwon_command_parser(input)
    }
    pub async fn handle_command(
        command: &str,
        user_state: &UserState,
    ) -> Result<UserStatus, InternalIrcError> {
        match IrcUnknownCommand::irc_command_parser(command) {
            Ok((_rem, IrcUnknownCommand(parsed_command))) => {
                let user_caracs = user_state.get_caracs().await;
                let nick = if user_caracs.registered {
                    user_caracs.nick.unwrap().clone()
                } else {
                    "*".to_string()
                };
                let irc_reply = IrcReply::ErrUnknownCommand {
                    nick: &nick,
                    command: &parsed_command,
                };
                let unknown_command_message = IrcMessage::new(irc_reply.format());
                let _ = user_state.tx_outbound.send(unknown_command_message).await;
                if &nick != "*" {
                    Ok(UserStatus::Handshaking)
                } else {
                    Ok(UserStatus::Active)
                }
            }
            Err(_) => Err(InternalIrcError::ParsingError(format!(
                "error during parsing unknown command: '{command}'"
            ))),
        }
    }
}

pub fn unknwon_command_parser(input: &str) -> IResult<&str, IrcUnknownCommand> {
    let (rem, command) = take_till(|c| c == ' ').parse(input)?;
    Ok((rem, IrcUnknownCommand(command.to_string())))
}
