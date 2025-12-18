use std::fmt::Display;

pub enum Target {
    Channel(String),
    Nickname(String),
    HostNameMask(String),
    ServerNameMask(String),
    UseMask(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Nickname(pub String);
impl Display for Nickname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Username(pub String);
impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Realname(pub String);
impl Display for Realname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Channel(pub String);
impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Hostname(pub String);
impl Display for Hostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Ip6Addr(pub String);
impl Display for Ip6Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Ip4Addr(pub String);
impl Display for Ip4Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
