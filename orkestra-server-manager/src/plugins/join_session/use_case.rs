use std::net::SocketAddrV4;

use uuid::Uuid;

use crate::shared::services::sesser::Sesser;

use super::error::JoinSessionError;

pub async fn join_session<S: Sesser>(
    sesser: S,
    id: Uuid,
) -> Result<SocketAddrV4, JoinSessionError> {
    match sesser.get_by_id(id) {
        Some(session) => Ok(session.addr),
        None => Err(JoinSessionError::SessionNotFound(id)),
    }
}
