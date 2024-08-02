use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RemovePlayerFromSessionError {
    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),
}
