pub const SERVER_NAME: &str = "192.168.1.34";

// 001    RPL_WELCOME
//               "Welcome to the Internet Relay Network
//                <nick>!<user>@<host>"
pub const RPL_WELCOME_NB: u16 = 1;
pub const RPL_WELCOME_STR: &str = "Welcome to the Internet Relay Network";

// for Query User MODE
pub const RPL_UMODEIS_NB: u16 = 221;

// 331    RPL_NOTOPIC
//        "<channel> :No topic is set"
pub const RPL_NOTOPIC_NB: u16 = 331;
pub const RPL_NOTOPIC_STR: &str = "No topic is set";

// 332    RPL_TOPIC
//        "<channel> :<topic>"
pub const RPL_TOPIC_NB: u16 = 332;

// 353    RPL_NAMREPLY
//        "( "=" / "*" / "@" ) <channel>
//         :[ "@" / "+" ] <nick> *( " " [ "@" / "+" ] <nick> )
//   - "@" is used for secret channels, "*" for private
//     channels, and "=" for others (public channels).
pub const RPL_NAMREPLY_NB: u16 = 353;

// 366    RPL_ENDOFNAMES
//        "<channel> :End of NAMES list"
pub const RPL_ENDOFNAMES_NB: u16 = 353;
pub const RPL_ENDOFNAMES_STR: &str = "End of NAMES list";

// 403    ERR_NOSUCHCHANNEL
//        "<channel name> :No such channel"
//   - Used to indicate the given channel name is invalid.
pub const ERR_NOSUCHCHANNEL_NB: u16 = 403;
pub const ERR_NOSUCHCHANNEL_STR: &str = "No such channel";

// 421    ERR_UNKNOWNCOMMAND
//           "<command> :Unknown command"
pub const ERR_UNKNOWNCOMMAND_NB: u16 = 421;
pub const ERR_UNKNOWNCOMMAND_STR: &str = "Unknown command";

// 433    ERR_NICKNAMEINUSE
//               "<nick> :Nickname is already in use"

//          - Returned when a NICK message is processed that results
//            in an attempt to change to a currently existing
//            nickname.
pub const ERR_NICKNAMEINUSE_NB: u16 = 433;
pub const ERR_NICKNAMEINUSE_STR: &str = "Nickname is already in use";

// 442    ERR_NOTONCHANNEL
//        "<channel> :You're not on that channel"
//        - Returned by the server whenever a client tries to
//          perform a channel affecting command for which the
//          client isn't a member.
pub const ERR_NOTONCHANNEL_NB: u16 = 433;
pub const ERR_NOTONCHANNEL_STR: &str = "You're not on that channel";

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

// 471    ERR_CHANNELISFULL
//        "<channel> :Cannot join channel (+l)"
pub const ERR_CHANNELISFULL_NB: u16 = 471;
pub const ERR_CHANNELISFULL_STR: &str = "Cannot join channel (+l)";

// 473    ERR_INVITEONLYCHAN
//               "<channel> :Cannot join channel (+i)"
pub const ERR_INVITEONLYCHAN_NB: u16 = 473;
pub const ERR_INVITEONLYCHAN_STR: &str = "Cannot join channel (+i)";

// 474    ERR_BANNEDFROMCHAN
//        "<channel> :Cannot join channel (+b)"
pub const ERR_BANNEDFROMCHAN_NB: u16 = 474;
pub const ERR_BANNEDFROMCHAN_STR: &str = "Cannot join channel (+b)";

// 475    ERR_BADCHANNELKEY
//        "<channel> :Cannot join channel (+k)"
pub const ERR_BADCHANNELKEY_NB: u16 = 475;
pub const ERR_BADCHANNELKEY_STR: &str = "Cannot join channel (+k)";

pub const ERR_UMODEUNKNOWNFLAG_NB: u16 = 501;
pub const ERR_UMODEUNKNOWNFLAG_STR: &str = "Unknown MODE flag";

pub const ERR_USERSDONTMATCH_NB: u16 = 502;
pub const ERR_USERSDONTMATCH_STR: &str = "Cannot change mode for other users";

// ERR_NEEDMOREPARAMS
//                ERR_BADCHANMASK
// ERR_NOSUCHCHANNEL               ERR_TOOMANYCHANNELS
// ERR_TOOMANYTARGETS              ERR_UNAVAILRESOURCE
// RPL_TOPIC
