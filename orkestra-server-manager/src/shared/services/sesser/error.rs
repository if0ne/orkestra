use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum UpdateSessionError {
    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),

    #[error("Session is full")]
    SessionIsFull,
}
