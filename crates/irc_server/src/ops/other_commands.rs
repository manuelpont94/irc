pub enum IrcServiceQueryCommands {
    SERVLIST,
    SQUERY,
    WHO,
    WHOIS,
    WHOWAS,
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
