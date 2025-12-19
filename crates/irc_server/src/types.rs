use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Hash, Eq, Copy)]
pub struct ClientId(pub usize);
impl Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Target {
    Nickname(Nickname),
    ServerName(Hostname),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum MessageTo {
    ChannelName(ChannelName),
    Nickname(Nickname),
    TargetMask(TargetMask),
    UserHostServer((Username, Option<Host>, Hostname)),
    UserHost((Username, Host)),
    NickUserHost((Nickname, Username, Host)),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct TargetMask(pub String);
impl Display for TargetMask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Nickname(pub String);
impl Display for Nickname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Username(pub String);
impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Realname(pub String);
impl Display for Realname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ChannelName(pub String);
impl Display for ChannelName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Topic(pub String);
impl Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Host {
    Hostname(Hostname),
    IpAddr(IpAddr),
}
impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Host::Hostname(host) => write!(f, "{host}"),
            Host::IpAddr(ip) => write!(f, "{ip}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Hostname(pub String);
impl Display for Hostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum IpAddr {
    Ip4Addr(Ip4Addr),
    Ip6Addr(Ip6Addr),
}
impl Display for IpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpAddr::Ip4Addr(ip) => write!(f, "{ip}"),
            IpAddr::Ip6Addr(ip) => write!(f, "{ip}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Ip6Addr(pub String);
impl Display for Ip6Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Ip4Addr(pub String);
impl Display for Ip4Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
