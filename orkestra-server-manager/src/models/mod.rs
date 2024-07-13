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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionPresent {
    pub id: Uuid,
    pub title: String,
}

impl From<Session> for SessionPresent {
    fn from(value: Session) -> Self {
        SessionPresent {
            id: value.id,
            title: value.title,
        }
    }
}