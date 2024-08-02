use serde::Deserialize;

use crate::models::session::{Id, SessionConfig};

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub creator_id: Id,
    pub config: SessionConfig,
}
