use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while, take_while_m_n, take_while1},
    character::complete::{alpha1, alphanumeric0, digit0, digit1, one_of, satisfy},
    combinator::{map, recognize, verify},
    multi::{count, many0},
    sequence::{pair, preceded, tuple},
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
    pub fn parse(input: &str) -> IResult<&str, &str> {
        todo!()
    }
}
pub struct Command {}

pub struct Params {}

pub struct Message {
    prefix: Option<Prefix>,
    command: Command,
    params: Option<Params>,
}
impl FromStr for Message {
    type Err = MessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
