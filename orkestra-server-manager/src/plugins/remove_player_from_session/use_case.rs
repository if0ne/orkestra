use uuid::Uuid;

use crate::{
    models::session::{Id, UpdateSession},
    shared::services::sesser::{error::UpdateSessionError, Sesser},
};

use super::error::RemovePlayerFromSessionError;

pub async fn remove_player_from_session<S: Sesser>(
    sesser: S,
    player_id: Id,
    id: Uuid,
) -> Result<(), RemovePlayerFromSessionError> {
    match sesser.update_session(id, UpdateSession::RemovePlayer(player_id)) {
        Ok(_) => Ok(()),
        Err(err) => match err {
            UpdateSessionError::SessionNotFound(id) => {
                Err(RemovePlayerFromSessionError::SessionNotFound(id))
            }
            _ => unreachable!(),
        },
    }
}
