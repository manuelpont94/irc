//    421    ERR_UNKNOWNCOMMAND
//           "<command> :Unknown command"
pub const ERR_UNKNOWNCOMMAND_NB: u16 = 421;
pub const ERR_UNKNOWNCOMMAND_STR: &'static str = "Unknown command";

// 461    ERR_NEEDMOREPARAMS
//               "<command> :Not enough parameters"

//          - Returned by the server by numerous commands to
//            indicate to the client that it didn't supply enough
//            parameters.
pub const ERR_NEEDMOREPARAMS_NB: u16 = 461;
pub const ERR_NEEDMOREPARAMS_STR: &'static str = "Not enough parameters";
