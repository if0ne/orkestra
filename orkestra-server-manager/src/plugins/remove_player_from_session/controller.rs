use axum::{response::IntoResponse, Extension, Json};
use tracing::{info, info_span};

use crate::{
    plugins::remove_player_from_session::use_case,
    shared::{
        context::Context,
        services::sesser::Sesser,
        utils::{bad_request_json, just_ok},
    },
};

use super::dto::RemovePlayerFromSessionRequest;

pub async fn remove_player_from_session<S: Sesser>(
    Extension(context): Extension<Context<S>>,
    Json(request): Json<RemovePlayerFromSessionRequest>,
) -> impl IntoResponse {
    let span = info_span!("remove_player_from_session");
    let _guard = span.enter();

    info!(
        target: "remove_player_from_session",
        event = "Handle request",
        request = "Remove player from session",
        "session id" = %request.server_id,
        "player id" = ?request.player_id,
    );

    let session = use_case::remove_player_from_session(
        context.sesser(),
        request.player_id,
        request.server_id,
    )
    .await;

    match session {
        Ok(_) => just_ok(),
        Err(err) => bad_request_json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}
