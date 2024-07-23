use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct JoinSessionRequest {
    pub server_id: Uuid,
}
