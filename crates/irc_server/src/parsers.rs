use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while, take_while_m_n, take_while1},
    character::complete::{char, satisfy},
    combinator::{opt, recognize, verify},
    multi::{count, many0, separated_list1},
    sequence::{pair, preceded},
};

// 2.3.1 Message format in Augmented BNF

//    The protocol messages must be extracted from the contiguous stream of
//    octets.  The current solution is to designate two characters, CR and
//    LF, as message separators.  Empty messages are silently ignored,
//    which permits use of the sequence CR-LF between messages without
//    extra problems.

//    The extracted message is parsed into the components <prefix>,
//    <command> and list of parameters (<params>).

//     The Augmented BNF representation for this is:

//  a.   message    =  [ ":" prefix SPACE ] command [ params ] crlf
//  b.   prefix     =  servername / ( nickname [ [ "!" user ] "@" host ] )
//  C.   command    =  1*letter / 3digit
//  d.   params     =  *14( SPACE middle ) [ SPACE ":" trailing ]
//                =/ 14( SPACE middle ) [ SPACE [ ":" ] trailing ]

//  e.   nospcrlfcl =  %x01-09 / %x0B-0C / %x0E-1F / %x21-39 / %x3B-FF
//                     ; any octet except NUL, CR, LF, " " and ":"
//  f.   middle     =  nospcrlfcl *( ":" / nospcrlfcl )
//  g.   trailing   =  *( ":" / " " / nospcrlfcl )

//  h.   SPACE      =  %x20        ; space character
//  i.   crlf       =  %x0D %x0A   ; "carriage return" "linefeed"

//  g.   trailing   =  *( ":" / " " / nospcrlfcl )

//  h.   wildcards = 3.3.1 Private messages [...] Wildcards are the  '*' and '?'  characters.

//  i.   masks
fn is_nospcrlfcl(c: u8) -> bool {
    match c {
        0x01..=0x09 | 0x0B..=0x0C | 0x0E..=0x1F | 0x21..=0x39 | 0x3B..=0xFF => true,
        _ => false,
    }
}

//  f.   middle     =  nospcrlfcl *( ":" / nospcrlfcl )
pub fn middle_parser(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        take_while1(|c: char| is_nospcrlfcl(c as u8)),
        many0(alt((
            tag(":"), // literal colon allowed after first char
            take_while1(|c: char| is_nospcrlfcl(c as u8)),
        ))),
    ))
    .parse(input)
}

//  g.   trailing   =  *( ":" / " " / nospcrlfcl )
pub fn trailing_parser(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c == ':' || c == ' ' || is_nospcrlfcl(c as u8)).parse(input)
}

//  h.   wildcards = 3.3.1 Private messages [...] Wildcards are the  '*' and '?'  characters.
pub fn wildcards_parser(input: &str) -> IResult<&str, &str> {
    alt((tag("#"), tag("?"))).parse(input)
}

// 00.  target     =  nickname / server
// 01.  msgtarget  =  msgto *( "," msgto )
// 02.  msgto      =  channel / ( user [ "%" host ] "@" servername )
//      msgto      =/ ( user "%" host ) / targetmask
//      msgto      =/ nickname / ( nickname "!" user "@" host )
// 03.  channel    =  ( "#" / "+" / ( "!" channelid ) / "&" ) chanstring
//                 [ ":" chanstring ]
// 04.  servername =  hostname
// 05.  host       =  hostname / hostaddr
// 06.  hostname   =  shortname *( "." shortname )
// 07.  shortname  =  ( letter / digit ) *( letter / digit / "-" )
//                 *( letter / digit )
//                   ; as specified in RFC 1123 [HNAME]
// 08.  hostaddr   =  ip4addr / ip6addr
// 09.  ip4addr    =  1*3digit "." 1*3digit "." 1*3digit "." 1*3digit
// 10.  ip6addr    =  1*hexdigit 7( ":" 1*hexdigit )
//      ip6addr    =/ "0:0:0:0:0:" ( "0" / "FFFF" ) ":" ip4addr
// 11.  nickname   =  ( letter / special ) *8( letter / digit / special / "-" )
// 12.  targetmask =  ( "$" / "#" ) mask
//                   ; see details on allowed masks in section 3.3.1
// 13.  chanstring =  %x01-07 / %x08-09 / %x0B-0C / %x0E-1F / %x21-2B
//      chanstring =/ %x2D-39 / %x3B-FF
//                   ; any octet except NUL, BELL, CR, LF, " ", "," and ":"
// 14.  channelid  = 5( %x41-5A / digit )   ; 5( A-Z / 0-9 )

//   Other parameter syntaxes are:

// 15.  user       =  1*( %x01-09 / %x0B-0C / %x0E-1F / %x21-3F / %x41-FF )
//                   ; any octet except NUL, CR, LF, " " and "@"
// 16.  key        =  1*23( %x01-05 / %x07-08 / %x0C / %x0E-1F / %x21-7F )
//                   ; any 7-bit US_ASCII character,
//                   ; except NUL, CR, LF, FF, h/v TABs, and " "
//   letter     =  %x41-5A / %x61-7A       ; A-Z / a-z
//   digit      =  %x30-39                 ; 0-9
//   hexdigit   =  digit / "A" / "B" / "C" / "D" / "E" / "F"
//   special    =  %x5B-60 / %x7B-7D
//                    ; "[", "]", "\", "`", "_", "^", "{", "|", "}"

fn hexdigit(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_hexdigit())(input)
}

// 00.  target     =  nickname / server
pub fn target_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = alt((nickname_parser, servername_parser));
    parser.parse(input)
}

// 01.  msgtarget  =  msgto *( "," msgto )
pub fn msgtarget_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = recognize(pair(msgto_parser, many0(preceded(tag(","), msgto_parser))));
    parser.parse(input)
}

// 02.  msgto      =  channel / ( user [ "%" host ] "@" servername )
//      msgto      =/ ( user "%" host ) / targetmask
//      msgto      =/ nickname / ( nickname "!" user "@" host )
fn msgto_user_host_server_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = recognize((
        user_parser,
        opt(preceded(tag("%"), host_parser)),
        tag("@"),
        servername_parser,
    ));
    parser.parse(input)
}

fn msgto_user_host_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = recognize((user_parser, tag("%"), host_parser));
    parser.parse(input)
}

fn msgto_nick_user_host_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = recognize((
        nickname_parser,
        tag("!"),
        user_parser,
        tag("@"),
        host_parser,
    ));
    parser.parse(input)
}

pub fn msgto_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = alt((
        channel_parser,
        msgto_user_host_server_parser,
        msgto_user_host_parser,
        targetmask_parser,
        msgto_nick_user_host_parser,
        nickname_parser,
    ));
    parser.parse(input)
}

// 03.  channel    =  ( "#" / "+" / ( "!" channelid ) / "&" ) chanstring
//                 [ ":" chanstring ]
fn channel_prefix_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = alt((
        tag("#"),
        tag("+"),
        recognize(pair(tag("!"), channelid_parser)),
        tag("&"),
    ));
    parser.parse(input)
}

// channel = ( "#" / "+" / ( "!" channelid ) / "&" ) chanstring [ ":" chanstring ]
pub fn channel_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = recognize((
        channel_prefix_parser,
        chanstring_parser,
        opt(preceded(tag(":"), chanstring_parser)),
    ));
    parser.parse(input)
}

// 04.  servername =  hostname
pub fn servername_parser(input: &str) -> IResult<&str, &str> {
    hostname_parser(input) // earlier definition
}

// 05.  host       =  hostname / hostaddr
// host = hostname / hostaddr
pub fn host_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = alt((hostname_parser, hostaddr_parser));
    parser.parse(input)
}

// 06.  hostname   =  shortname *( "." shortname )
// hostname = shortname *( "." shortname )
pub fn hostname_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = verify(
        recognize((
            shortname_parser,
            many0(preceded(tag("."), shortname_parser)),
        )),
        |s: &str| s.len() <= 63,
    );
    parser.parse(input)
}

// 07.  shortname  =  ( letter / digit ) *( letter / digit / "-" )
//                 *( letter / digit )
//                   ; as specified in RFC 1123 [HNAME]
pub fn shortname_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = recognize((
        satisfy(|c| c.is_ascii_alphanumeric()), // first char
        many0(satisfy(|c| c.is_ascii_alphanumeric() || c == '-')),
        satisfy(|c| c.is_ascii_alphanumeric()), // last char
    ));
    parser.parse(input)
}

// 08.  hostaddr   =  ip4addr / ip6addr
// hostaddr = ip4addr / ip6addr
pub fn hostaddr_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = alt((ip4addr_parser, ip6addr_parser));
    parser.parse(input)
}

// 09.  ip4addr    =  1*3digit "." 1*3digit "." 1*3digit "." 1*3digit
// ip4addr = 1*3digit "." 1*3digit "." 1*3digit "." 1*3digit
fn ip4_octet_parser(input: &str) -> IResult<&str, &str> {
    take_while_m_n(1, 3, |c: char| c.is_ascii_digit())(input)
}

fn ip4addr_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = recognize((
        ip4_octet_parser,
        tag("."),
        ip4_octet_parser,
        tag("."),
        ip4_octet_parser,
        tag("."),
        ip4_octet_parser,
    ));
    parser.parse(input)
}

// 10.  ip6addr    =  1*hexdigit 7( ":" 1*hexdigit )
//      ip6addr    =/ "0:0:0:0:0:" ( "0" / "FFFF" ) ":" ip4addr
// ip6addr = 1*hexdigit 7( ":" 1*hexdigit )
fn ip6_block_parser(input: &str) -> IResult<&str, &str> {
    hexdigit(input) // already allows 1+
}

fn ip6addr_normal_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = recognize((
        ip6_block_parser,
        count(preceded(tag(":"), ip6_block_parser), 7),
    ));
    parser.parse(input)
}

// ip6addr =/ "0:0:0:0:0:" ( "0" / "FFFF" ) ":" ip4addr
fn ip6addr_ipv4_compat_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = recognize((
        tag("0:0:0:0:0:"),
        alt((tag("0"), tag("FFFF"))),
        tag(":"),
        ip4addr_parser,
    ));
    parser.parse(input)
}

fn ip6addr_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = alt((ip6addr_ipv4_compat_parser, ip6addr_normal_parser));
    parser.parse(input)
}

// 11.  nickname   =  ( letter / special ) *8( letter / digit / special / "-" )
// nickname = ( letter / special ) *8( letter / digit / special / "-" )
fn is_nickname_tail_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "-[]\\`^{}".contains(c)
}

fn is_nickname_first_char(c: char) -> bool {
    c.is_ascii_alphabetic() || "-[]\\`^{}".contains(c)
}

pub fn nickname_parser(input: &str) -> IResult<&str, &str> {
    // First char: letter OR special
    let first = satisfy(is_nickname_first_char);

    // Up to 8 tail characters
    // tail_char = letter / digit / special / "-"
    let tail = take_while(is_nickname_tail_char);

    let parser = recognize(pair(first, tail));

    // Enforce max length = 9
    verify(parser, |s: &str| s.len() <= 9).parse(input) // first char control ensure that no empty string can be valid
}

// 12.  targetmask =  ( "$" / "#" ) mask
//                   ; see details on allowed masks in section 3.3.1
// fn mask_parser(input: &str) -> IResult<&str, &str> {
//     // Placeholder — ask me if you need full mask rules!
//     take_while1(|c: char| c != ' ' && c != ',')(input)
// }
/// Checks if a character is a valid mask character according to RFC 2812.
/// Must be:
/// 1. Not NUL, CR, LF (standard line endings)
/// 2. Not Space, Comma, or Colon (standard IRC parameter delimiters)
/// 3. Not a Dot (since this function defines the *segments* between dots)
fn is_valid_mask_segment_char(c: char) -> bool {
    c != '\0'
        && c != '\r'
        && c != '\n'
        && c != ' '
        && c != ','
        && c != ':'
        && c != '.'
        && c.is_ascii()
}

/// Checks if a character is a valid wildcard: '*' or '?'
fn is_wildcard(c: char) -> bool {
    c == '*' || c == '?'
}

/// Parses a single, structurally valid segment of the mask (sequence of characters not including dots).
/// It ensures that all characters comply with general IRC parameter rules.
fn mask_segment(input: &str) -> IResult<&str, &str> {
    // Matches one or more characters that are valid for an IRC mask segment.
    take_while1(is_valid_mask_segment_char)(input)
}

/// **Constraints:**
/// 1. Must contain at least one "." (period).
/// 2. Must not contain any wildcards ('*' or '?') following the last ".".
pub fn targetmask_parser(input: &str) -> IResult<&str, &str> {
    // 1. Structure Check: Parse the mask as segments separated by dots.
    // We use recognize to get the full matched string slice, which is guaranteed
    // to be structurally correct (segments separated by dots) and free of disallowed IRC chars.
    let (rem, full_mask) =         // 2. Semantic Check: Apply the two RFC constraints using `verify`.
        verify(
            recognize(separated_list1(char('.'), mask_segment)),
            |mask_str: &str| {
                // Constraint 1: Must contain at least one dot.
                // `separated_list1` already enforces this, but a direct check is fine.
                let has_dot = mask_str.contains('.');

                // // Constraint 2: No wildcards after the last dot.
                let is_last_segment_valid = match mask_str.rfind('.') {
                    Some(index) => {
                        let post_dot_segment = &mask_str[index + 1..];
                        // Check if *any* character in the segment is a wildcard.
                        !post_dot_segment.chars().any(is_wildcard)
                    }
                    None => false, // Should be unreachable if has_dot is true, but safe.
                };

                // Both constraints must be true for a valid RFC 2812 host/server mask.
                has_dot && is_last_segment_valid
                // false
            },
        )
        .parse(input)?;
    Ok((rem, full_mask))
}

// pub fn targetmask_parser(input: &str) -> IResult<&str, &str> {
//     let mut parser = recognize(pair(alt((tag("$"), tag("#"))), mask_parser));
//     parser.parse(input)
// }

// 13.  chanstring =  %x01-07 / %x08-09 / %x0B-0C / %x0E-1F / %x21-2B
//      chanstring =/ %x2D-39 / %x3B-FF
//                   ; any octet except NUL, BELL, CR, LF, " ", "," and ":"
fn is_chan_char(c: char) -> bool {
    match c {
        '\u{0000}' | '\u{0007}' | '\r' | '\n' | ' ' | ',' | ':' => false,
        _ => c as u32 <= 0xFF,
    }
}

fn chanstring_parser(input: &str) -> IResult<&str, &str> {
    take_while1(is_chan_char)(input)
}

// 14.  channelid  = 5( %x41-5A / digit )   ; 5( A-Z / 0-9 )
// channelid = 5( %x41-5A / digit ) ; A–Z or 0–9
fn channelid_parser(input: &str) -> IResult<&str, &str> {
    let mut parser = recognize(count(
        satisfy(|c: char| c.is_ascii_uppercase() || c.is_ascii_digit()),
        5,
    ));
    parser.parse(input)
}

// 15.  user       =  1*( %x01-09 / %x0B-0C / %x0E-1F / %x21-3F / %x41-FF )
//                   ; any octet except NUL, CR, LF, " " and "@"
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
pub fn user_parser(input: &str) -> IResult<&str, &str> {
    take_while1(is_user_char).parse(input)
}

// 16.  key        =  1*23( %x01-05 / %x07-08 / %x0C / %x0E-1F / %x21-7F )
//                   ; any 7-bit US_ASCII character,
//                   ; except NUL, CR, LF, FF, h/v TABs, and " "
fn is_key_char(c: char) -> bool {
    // Reject any non-ASCII byte (multi-byte UTF-8)
    if !c.is_ascii() {
        return false;
    }
    let b = c as u8;
    matches!(b,
        0x01..=0x05 |  // exclude NUL, ACK
        0x07..=0x08 |  // exclude ACK, include BEL and BS
        0x0C |         // FF
        0x0E..=0x1F |  // exclude CR (0x0D), include SO through US
        0x21..=0x7F    // excludes SPACE (0x20), includes ! through DEL
    )
}

/// Parses "key" according to RFC2812 ABNF rule.
/// Maximum length is 23 characters.
pub fn key_parser(input: &str) -> IResult<&str, &str> {
    verify(take_while1(is_key_char), |s: &str| s.len() <= 23).parse(input)
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
            let (rest, out) = nickname_parser(case).expect(&format!("Should parse: {case}"));
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
            assert!(nickname_parser(case).is_err(), "Should fail: {case}");
        }
    }

    #[test]
    fn test_partial_parse() {
        // valid prefix, then an invalid char later
        let (rest, out) = nickname_parser("abc!def").unwrap();
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
                user_parser(case).unwrap_or_else(|_| panic!("should parse: {case:?}"));
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
            assert!(user_parser(case).is_err(), "should fail at start: {case:?}");
        }
    }

    #[test]
    fn stops_on_invalid_middle() {
        let (rest, out) = user_parser("foo bar").unwrap();
        assert_eq!(out, "foo");
        assert_eq!(rest, " bar");
    }

    #[test]
    fn rejects_utf8_multibyte() {
        // snowman = 0xE2 98 83 (multi-byte UTF-8)
        assert!(user_parser("☃test").is_err());

        // multi-byte anywhere stops parsing
        let (rest, out) = user_parser("abc☃def")
            .unwrap_or_else(|_| panic!("should partially parse ASCII prefix"));
        assert_eq!(out, "abc");
        assert_eq!(rest, "☃def");
    }

    #[test]
    fn control_character_edge_cases() {
        // Check boundaries explicitly
        assert!(user_parser("\x01").is_ok());
        assert!(user_parser("\x09").is_ok());
        assert!(user_parser("\x0A").is_err()); // LF
        assert!(user_parser("\x0B").is_ok());
        assert!(user_parser("\x0C").is_ok());
        assert!(user_parser("\x0D").is_err()); // CR
        assert!(user_parser("\x0E").is_ok());
        assert!(user_parser("\x1F").is_ok());
        assert!(user_parser("\x20").is_err()); // space
        assert!(user_parser("\x40").is_err()); // '@'
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_valid_masks_final() {
//         assert_eq!(rfc2812_mask_final("*.foo.com"), Ok(("", "*.foo.com")));
//         assert_eq!(rfc2812_mask_final("a-b.c@d"), Ok(("", "a-b.c@d")));
//         assert_eq!(rfc2812_mask_final("?user@host.domain"), Ok(("", "?user@host.domain")));
//     }

//     #[test]
//     fn test_invalid_masks_no_dot_final() {
//         // Fails: no dot present.
//         assert!(rfc2812_mask_final("abc").is_err());
//     }

//     #[test]
//     fn test_invalid_masks_wildcard_after_last_dot_final() {
//         // Fails: wildcard after the last dot.
//         assert!(rfc2812_mask_final("a.b*").is_err());
//     }

//     #[test]
//     fn test_invalid_masks_disallowed_chars() {
//         // Fails: contains a space (disallowed by is_valid_mask_segment_char).
//         assert!(rfc2812_mask_final("a.b c").is_err());
//         // Fails: contains a comma (disallowed by is_valid_mask_segment_char).
//         assert!(rfc2812_mask_final("a,b.c").is_err());
//         // Fails: contains a colon (disallowed by is_valid_mask_segment_char).
//         assert!(rfc2812_mask_final("a.b:c").is_err());
//     }
// }
