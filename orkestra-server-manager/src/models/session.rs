use std::net::SocketAddrV4;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    pub id: Uuid,
    pub addr: SocketAddrV4,
    pub title: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionConfig {
    pub max_players: u32,
    pub game_map: String,
    pub title: String,
}
