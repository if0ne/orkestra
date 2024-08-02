use axum::{routing::post, Router};

use crate::shared::services::sesser::Sesser;

use super::controller::remove_player_from_session;

pub fn service<S: Sesser>() -> Router {
    Router::new().route(
        "/remove_player_from_session",
        post(remove_player_from_session::<S>),
    )
}
