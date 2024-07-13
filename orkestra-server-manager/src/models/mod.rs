use std::net::SocketAddrV4;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct SessionsInMemory {
    pub sessions: DashMap<Uuid, Session>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    pub id: Uuid,
    pub addr: SocketAddrV4,
    pub title: String,
}
