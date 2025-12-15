pub const SERVER_NAME: &str = "192.168.1.34";

// 001    RPL_WELCOME
//               "Welcome to the Internet Relay Network
//                <nick>!<user>@<host>"
pub const RPL_WELCOME_NB: u16 = 1;
pub const RPL_WELCOME_STR: &str = "Welcome to the Internet Relay Network";

// 421    ERR_UNKNOWNCOMMAND
//           "<command> :Unknown command"
pub const ERR_UNKNOWNCOMMAND_NB: u16 = 421;
pub const ERR_UNKNOWNCOMMAND_STR: &str = "Unknown command";

// 451    ERR_NOTREGISTERED
//               ":You have not registered"

//          - Returned by the server to indicate that the client
//            MUST be registered before the server will allow it
//            to be parsed in detail.
pub const ERR_NOTREGISTERED_NB: u16 = 451;
pub const ERR_NOTREGISTERED_STR: &str = ":You have not registered";

// 461    ERR_NEEDMOREPARAMS
//               "<command> :Not enough parameters"

//          - Returned by the server by numerous commands to
//            indicate to the client that it didn't supply enough
//            parameters.
pub const ERR_NEEDMOREPARAMS_NB: u16 = 461;
pub const ERR_NEEDMOREPARAMS_STR: &str = "Not enough parameters";

// pub const ERR_NEEDMOREPARAMS_NB: u16 = 0;
pub const ERR_UMODEUNKNOWNFLAG_NB: u16 = 501;
pub const ERR_UMODEUNKNOWNFLAG_STR: &str = "Unknown MODE flag";

pub const ERR_USERSDONTMATCH_NB: u16 = 502;
pub const ERR_USERSDONTMATCH_STR: &str = "Cannot change mode for other users";

// for Query User MODE
pub const RPL_UMODEIS_NB: u16 = 221;
