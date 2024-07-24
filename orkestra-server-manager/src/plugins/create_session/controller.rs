use axum::{extract::Extension, response::IntoResponse, Json};

use tracing::{info, info_span, Instrument};

use crate::{
    plugins::create_session::use_case,
    shared::{
        services::sesser::Sesser,
        utils::{bad_request_json, ok_json},
    },
    Context,
};

use super::dto::CreateSessionRequest;

pub async fn create_session<S: Sesser>(
    Extension(context): Extension<Context<S>>,
    Json(request): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    let span = info_span!("create_session");
    let _guard = span.enter();

    info!(
        event = "Handle request",
        request = "Create Session",
        config = ?request.config
    );

    let session = use_case::create_session(context.sesser(), request.config)
        .in_current_span()
        .await;

    match session {
        Ok(session) => ok_json(serde_json::json!({
            "connection": session.addr.to_string()
        })),
        Err(error) => bad_request_json(serde_json::json!({
            "error": error.to_string()
        })),
    }
}
