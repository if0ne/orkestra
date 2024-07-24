use axum::{response::IntoResponse, Extension, Json};
use tracing::{info, info_span};

use crate::{
    plugins::join_session::use_case,
    shared::{
        context::Context,
        services::sesser::Sesser,
        utils::{bad_request_json, ok_json},
    },
};

use super::dto::JoinSessionRequest;

pub async fn join_session<S: Sesser>(
    Extension(context): Extension<Context<S>>,
    Json(request): Json<JoinSessionRequest>,
) -> impl IntoResponse {
    let span = info_span!("join_session");
    let _guard = span.enter();

    info!(
        target: "join_session",
        event = "Handle request",
        request = "Join session",
        "session id" = %request.server_id
    );

    let session = use_case::join_session(context.sesser(), request.server_id).await;

    match session {
        Ok(addr) => ok_json(serde_json::json!({
            "connection": addr.to_string()
        })),
        Err(err) => bad_request_json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}
