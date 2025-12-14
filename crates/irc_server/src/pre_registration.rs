use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag_no_case, take_till},
    combinator::recognize,
};

use crate::{errors::InternalIrcError, handlers::registration::*};
// CAP            = "CAP" SP cap-subcmd [SP cap-params]
// cap-subcmd     = "LS" / "LIST" / "REQ" / "ACK" / "NAK" / "CLEAR" / "END"
// cap-params     = 1*(cap-token / cap-version / cap-list)
// cap-list       = cap-token *(" " cap-token)
// cap-token      = 1*(ALPHA / DIGIT / "-" / "_")
// cap-version    = cap-token "=" 1*(ALPHA / DIGIT / "." / "-" / "_")

// FULL CAPABILITY LIST EXPLAINED

// account-notify: announce account login/logout
// account-tag: adds account name to message tags
// away-notify: notify channel when users set or remove away status
// batch: required for chathistory and playback
// cap-notify: inform clients when capabilities change dynamically
// chghost: notify clients about host changes
// echo-message: sender sees their own messages
// extended-join: JOIN message includes account + realname
// invite-notify: notify ops when invites are sent
// message-tags: IRCv3 message tagging system
// multi-prefix: send all user mode prefixes
// sasl: authentication before registration
// server-time: adds precise timestamps
// setname: clients can change realname via SETNAME
// userhost-in-names: include full user@host in NAMES list

#[derive(Debug, PartialEq)]
pub enum IrcCapPreRegistration {
    LS,
    LIST,
    REQ(String),
    ACK(String),
    NACK(String),
    CLEAR(String),
    END,
}

impl IrcCapPreRegistration {
    pub fn irc_cap_parser(input: &str) -> IResult<&str, Self> {
        let mut parser = alt((valid_cap_ls, valid_cap_list, valid_cap_end));
        parser.parse(input)
    }

    pub fn handle_command(command: &str, user: &str) -> Result<Option<String>, InternalIrcError> {
        match IrcCapPreRegistration::irc_cap_parser(command) {
            Ok((_, valid_cap)) => match valid_cap {
                IrcCapPreRegistration::LS => Ok(handle_cap_ls_response(user)),
                IrcCapPreRegistration::LIST => Ok(handle_cap_list_response(user)),
                IrcCapPreRegistration::END => Ok(handle_cap_end_response()),
                _ => todo!(),
            },
            Err(_e) => Err(InternalIrcError::InvalidCommand),
        }
    }
}

// 3.1 CAP LS [version]

// Client → server OR server → client.
// Requests server capability listing.
// Server replies with its full set.
// C: CAP LS 302
// S: CAP * LS :sasl multi-prefix echo-message

fn valid_cap_ls(input: &str) -> IResult<&str, IrcCapPreRegistration> {
    let (rem, _parsed) =
        recognize((tag_no_case("CAP LS"), take_till(|c| c == '\r' || c == '\n'))).parse(input)?;
    Ok((rem, IrcCapPreRegistration::LS))
}

// 3.2 CAP LIST
// Client → server.
// Server returns the list of capabilities currently active for this client.

fn valid_cap_list(input: &str) -> IResult<&str, IrcCapPreRegistration> {
    let (rem, _parsed) = recognize((
        tag_no_case("CAP LIST"),
        take_till(|c| c == '\r' || c == '\n'),
    ))
    .parse(input)?;
    Ok((rem, IrcCapPreRegistration::LIST))
}

// 3.3 CAP REQ <capabilities>
// Client → server.
// Asks the server to enable specific capabilities.
// Example:
// CAP REQ :sasl echo-message

// 3.4 CAP ACK <capabilities>
// Server → client.
// Server accepted the request.

// 3.5 CAP NAK <capabilities>
// Server → client.
// Server rejected some or all requested capabilities.
// Reasons include:
// Nonexistent capability
// Version mismatch
// Not allowed for this client
// Requires authentication

// 3.6 CAP CLEAR
// Client → server.
// Client requests disabling all active capabilities.
// Server responds with:
// CAP ACK :

// 3.7 CAP END
// Client → server.
// Ends negotiation.
// After this, client typically expects start of normal IRC registration.

fn valid_cap_end(input: &str) -> IResult<&str, IrcCapPreRegistration> {
    let (rem, _parsed) = recognize((
        tag_no_case("CAP END"),
        take_till(|c| c == '\r' || c == '\n'),
    ))
    .parse(input)?;
    Ok((rem, IrcCapPreRegistration::END))
}

//     +-------------------------+
//     |       Disconnected      |
//     +------------+------------+
//                  |
//                  v
//       +----------+----------+
//       |  Pre-registration   |
//       | (before 001 welcome)|
//       +----------------------+
//          |   ∧
//          |   |
//  CAP LS  |   | CAP LS response
//  CAP REQ |   | CAP ACK/NAK
//  CAP LIST|   |
//          v   |
//  +------------+---------+
//  |  CAP negotiation     |
//  +-----------------------+
//          |
//      CAP END
//          |
//          v
//    Normal IRC state

// IRCv3 CAPABILITIES DESCRIPTION

////////////////////////////
// Capability: SASL
////////////////////////////
// Purpose:
// SASL allows a client to authenticate securely before IRC registration finishes.
// Server announces:
// CAP * LS :sasl
// Client flow:
// CAP REQ :sasl
// CAP ACK :sasl
// AUTHENTICATE PLAIN
// AUTHENTICATE <base64 user/pass>
// 903 <nick> :SASL authentication successful
// or
// 904 <nick> :SASL authentication failed
// Client ends with:
// CAP END
// Server obligations:

// Accept AUTHENTICATE command during pre-registration.

// Send "AUTHENTICATE +" to signal readiness.

// Decode base64 credentials.

// Send numeric 903 on success, 904 on failure.

// Delay sending welcome (001) until SASL completes.
// Minimum server logic:
// AUTHENTICATE PLAIN
// AUTHENTICATE +
// 903 <nick> :OK
// Notes:
// Irssi does not require SASL.

////////////////////////////
// Capability: echo-message
////////////////////////////
// Purpose:
// Server echoes back the users own PRIVMSG and NOTICE messages.
// Server announces:
// CAP * LS :echo-message
// Client flow:
// CAP REQ :echo-message
// CAP ACK :echo-message
// Behavior:
// If client sends:
// PRIVMSG #chan :hello
// Server must send back to the same client:
// :nick!user@host PRIVMSG #chan :hello
// Minimum server logic:
// When broadcasting a PRIVMSG, also send the same message back to the sender.
// Notes:
// Optional but easy to implement.

////////////////////////////
// Capability: multi-prefix
////////////////////////////
// Purpose:
// Allows server to send all prefix modes in NAMES replies.
// Without multi-prefix:
// :server 353 nick = #chan :@Alice Bob +Charlie
// With multi-prefix:
// :server 353 nick = #chan :@+Alice Bob +Charlie
// Server announces:
// CAP * LS :multi-prefix
// Client flow:
// CAP REQ :multi-prefix
// CAP ACK :multi-prefix
// Server obligations:
// Send full list of user mode prefixes in numeric 353 replies.
// Minimum server logic:
// For each user in a channel, build a prefix string containing all their modes.
// Notes:
// Very easy. Irssi fully compatible.

// Summary Table:
// Capability What it does Required by Irssi? Difficulty
// sasl Secure authentication before registration No Hard
// echo-message Echo users own PRIVMSG/NOTICE No Easy
// multi-prefix Send all user mode prefixes in NAMES No Very easy

// Recommended minimal capabilities for a homemade IRC server:
// Only announce:
// multi-prefix
// Do not announce sasl unless implemented.
// echo-message optional.
