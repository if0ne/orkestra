use tracing::Instrument;

use crate::{
    models::session::{Id, Session, SessionConfig},
    shared::services::sesser::Sesser,
};

use super::error::CreateSessionError;

pub async fn create_session<S: Sesser>(
    sesser: S,
    creator_id: Id,
    config: SessionConfig,
) -> Result<Session, CreateSessionError> {
    sesser
        .create_session(creator_id, config)
        .in_current_span()
        .await
        .map_err(|err| CreateSessionError::StartServer(err.to_string()))
}
