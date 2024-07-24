use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateSessionError {
    #[error("Start server error: {0}")]
    StartServer(String),
}
