use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum JoinSessionError {
    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),
}
