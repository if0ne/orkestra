use serde::Deserialize;

use crate::models::session::SessionConfig;

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub config: SessionConfig,
}
