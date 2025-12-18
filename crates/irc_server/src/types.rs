pub enum Target {
    Channel(String),
    Nickname(String),
    HostNameMask(String),
    ServerNameMask(String),
    UseMask(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Nickname(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct Channel(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum Host {
    Hostname(Hostname),
    IpAddr(IpAddr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hostname(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum IpAddr {
    Ip4Addr(Ip4Addr),
    Ip6Addr(Ip6Addr)
}
#[derive(Debug, Clone, PartialEq)]
pub struct Ip6Addr(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct Ip4Addr(pub String);