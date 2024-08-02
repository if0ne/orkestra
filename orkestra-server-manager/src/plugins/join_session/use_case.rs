use std::net::SocketAddrV4;

use uuid::Uuid;

use crate::{
    models::session::{Id, UpdateSession},
    shared::services::sesser::{error::UpdateSessionError, Sesser},
};

use super::error::JoinSessionError;

pub async fn join_session<S: Sesser>(
    sesser: S,
    player_id: Id,
    id: Uuid,
) -> Result<SocketAddrV4, JoinSessionError> {
    match sesser.update_session(id, UpdateSession::AddPlayer(player_id)) {
        Ok(session) => Ok(session.addr),
        Err(err) => match err {
            UpdateSessionError::SessionNotFound(id) => Err(JoinSessionError::SessionNotFound(id)),
            UpdateSessionError::SessionIsFull => Err(JoinSessionError::SessionIsFull),
        },
    }
}
