use dashmap::{DashMap, DashSet};

pub type UserId = u64;
type Nick = String;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct User {
    id: UserId,
    nick: Nick,
}
