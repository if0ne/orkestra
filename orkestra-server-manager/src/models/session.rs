use std::{collections::HashSet, net::SocketAddrV4};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub addr: SocketAddrV4,
    pub title: String,
    pub code: String,
    pub max_players: u32,
    pub players: HashSet<Id>,
}

#[derive(Debug, Deserialize)]
pub struct SessionConfig {
    pub max_players: u32,
    pub game_map: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Id(pub String);

#[derive(Debug, Clone)]
pub enum UpdateSession {
    AddPlayer(Id),
    RemovePlayer(Id),
}
