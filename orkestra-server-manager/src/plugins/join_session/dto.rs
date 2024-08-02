use serde::Deserialize;
use uuid::Uuid;

use crate::models::session::Id;

#[derive(Debug, Deserialize)]
pub struct JoinSessionRequest {
    pub server_id: Uuid,
    pub player_id: Id,
}
