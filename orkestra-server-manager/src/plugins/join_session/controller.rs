use axum::{response::IntoResponse, Extension, Json};
use tracing::{info, info_span};

use crate::shared::{
    context::Context,
    services::sesser::Sesser,
    utils::{bad_request_json, ok_json},
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

    let Some(server) = context.sesser().get_by_id(request.server_id) else {
        return bad_request_json(serde_json::json!({
            "error": "Session not found"
        }));
    };

    ok_json(serde_json::json!({
        "connection": server.addr.to_string()
    }))
}
