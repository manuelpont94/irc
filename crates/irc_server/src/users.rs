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

pub struct Nickname(String);
// nickname = ( letter / special ) *8( letter / digit / special / "-" )
impl Nickname {
    fn is_tail_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || "-[]\\`^{}".contains(c)
    }

    fn is_first_char(c: char) -> bool {
        c.is_ascii_alphabetic() || "-[]\\`^{}".contains(c)
    }

    pub fn parse(input: &str) -> IResult<&str, &str> {
        // First char: letter OR special
        let first = satisfy(Nickname::is_first_char);

        // Up to 8 tail characters
        // tail_char = letter / digit / special / "-"
        let tail = take_while(Nickname::is_tail_char);

        let parser = recognize(pair(first, tail));

        // Enforce max length = 9
        verify(parser, |s: &str| s.len() <= 9).parse(input)
    }

    pub fn is_nickname_valid(input: &str) -> bool {
        Nickname::parse(input).is_ok_and(|(rem, _)| rem == "")
    }
}

impl FromStr for Nickname {
    type Err = MessageError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match Nickname::parse(input) {
            Ok((_rem, nickname)) => Ok(Nickname(nickname.to_owned())),
            Err(_) => Err(MessageError::ParseError("error with the nickname")),
        }
    }
}

// 1.2.1 Users

//    Each user is distinguished from other users by a unique nickname
//    having a maximum length of nine (9) characters.  See the protocol
//    grammar rules (section 2.3.1) for what may and may not be used in a
//    nickname.

//   user       =  1*( %x01-09 / %x0B-0C / %x0E-1F / %x21-3F / %x41-FF )
//                   ; any octet except NUL, CR, LF, " " and "@"

//    While the maximum length is limited to nine characters, clients
//    SHOULD accept longer strings as they may become used in future
//    evolutions of the protocol.
pub struct User(String);
impl User {
    fn is_user_char(c: char) -> bool {
        // Reject any non-ASCII byte (multi-byte UTF-8)
        if !c.is_ascii() {
            return false;
        }

        let b = c as u8;

        matches!(b,
            0x01..=0x09 |  // exclude NUL and LF
            0x0B..=0x0C |
            0x0E..=0x1F |
            0x21..=0x3F |  // excludes SPACE (0x20) and '@' (0x40)
            0x41..=0x7F    // ASCII 0x41+ (but UTF-8 never produces >0x7F as 1 byte)
        )
    }

    /// Parses "user" according to the ABNF rule.
    pub fn parse(input: &str) -> IResult<&str, &str> {
        take_while1(User::is_user_char).parse(input)
    }

    pub fn is_user_valid(input: &str) -> bool {
        User::parse(input).is_ok_and(|(rem, _)| rem == "")
    }
}

impl FromStr for User {
    type Err = MessageError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match User::parse(input) {
            Ok((_rem, user)) => Ok(User(user.to_owned())),
            Err(_) => Err(MessageError::ParseError("error with the user")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_identifiers() {
        let cases = [
            "a",
            "a1",
            "abc123",
            "Z9",
            "x-y",
            "t[est]",
            "g{ood}",
            "h\\i", // backslash
            "j`k",  // backtick
            "m^n",
            "Qwert\\`^",
            "{wert\\`^",
        ];

        for &case in &cases {
            let (rest, out) = Nickname::parse(case).expect(&format!("Should parse: {case}"));
            assert_eq!(rest, "");
            assert_eq!(out, case);
        }
    }

    #[test]
    fn test_invalid_identifiers() {
        let cases = [
            "1abc", // cannot start with digit
            "",     // empty
        ];

        for &case in &cases {
            assert!(Nickname::parse(case).is_err(), "Should fail: {case}");
        }
    }

    #[test]
    fn test_partial_parse() {
        // valid prefix, then an invalid char later
        let (rest, out) = Nickname::parse("abc!def").unwrap();
        assert_eq!(out, "abc");
        assert_eq!(rest, "!def");
    }

    #[test]
    fn valid_users() {
        let cases = [
            "a",
            "abc123",
            "hello.world",
            "user-name",
            "test!#$%&'()*+,-./0123",
            "AZaz09",     // plain alnum
            "\x01abc",    // lowest allowed control
            "\x1Ftest",   // high control range
            "\x21hello",  // ASCII printable except space/@
            "foo\x7Fbar", // DEL is allowed (%x41-FF but ASCII only goes to 0x7F)
        ];

        for &case in &cases {
            let (rest, out) =
                User::parse(case).unwrap_or_else(|_| panic!("should parse: {case:?}"));
            assert_eq!(rest, "");
            assert_eq!(out, case);
        }
    }

    #[test]
    fn invalid_starting_character() {
        let cases = [
            "",      // empty is invalid (needs 1+)
            "\0abc", // NUL
            " abc",  // space
            "@name", // '@'
            "\nabc", // LF
            "\rabc", // CR
        ];

        for &case in &cases {
            assert!(User::parse(case).is_err(), "should fail at start: {case:?}");
        }
    }

    #[test]
    fn stops_on_invalid_middle() {
        let (rest, out) = User::parse("foo bar").unwrap();
        assert_eq!(out, "foo");
        assert_eq!(rest, " bar");
    }

    #[test]
    fn rejects_utf8_multibyte() {
        // snowman = 0xE2 98 83 (multi-byte UTF-8)
        assert!(User::parse("☃test").is_err());

        // multi-byte anywhere stops parsing
        let (rest, out) = User::parse("abc☃def")
            .unwrap_or_else(|_| panic!("should partially parse ASCII prefix"));
        assert_eq!(out, "abc");
        assert_eq!(rest, "☃def");
    }

    #[test]
    fn control_character_edge_cases() {
        // Check boundaries explicitly
        assert!(User::parse("\x01").is_ok());
        assert!(User::parse("\x09").is_ok());
        assert!(User::parse("\x0A").is_err()); // LF
        assert!(User::parse("\x0B").is_ok());
        assert!(User::parse("\x0C").is_ok());
        assert!(User::parse("\x0D").is_err()); // CR
        assert!(User::parse("\x0E").is_ok());
        assert!(User::parse("\x1F").is_ok());
        assert!(User::parse("\x20").is_err()); // space
        assert!(User::parse("\x40").is_err()); // '@'
    }
}
