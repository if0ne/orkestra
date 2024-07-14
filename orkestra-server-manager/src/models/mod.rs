use std::{net::SocketAddrV4, sync::atomic::AtomicU32};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub static GLOBAL_CODE: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Default)]
pub struct SessionsInMemory {
    pub sessions: DashMap<Uuid, Session>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    pub id: Uuid,
    pub addr: SocketAddrV4,
    pub title: String,
    pub code: String,
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
