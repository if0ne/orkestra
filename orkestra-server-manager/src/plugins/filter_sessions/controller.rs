use axum::{
    extract::{Extension, Query},
    response::IntoResponse,
};
use tracing::{info, info_span};

use crate::{
    plugins::filter_sessions::{dto::SessionPresent, use_case},
    shared::{context::Context, services::sesser::Sesser, utils::ok_json},
};

use super::dto::FilterParams;

pub async fn filter_sessions<S: Sesser>(
    Extension(context): Extension<Context<S>>,
    Query(request): Query<FilterParams>,
) -> impl IntoResponse {
    let span = info_span!("filter_sessions");
    let _guard = span.enter();

    info!(
        target: "filter_sessions",
        event = "Handle request",
        request = "Filter sessions",
    );

    let sessions = use_case::filter_sessions(context, request.code)
        .into_iter()
        .map(|session| SessionPresent {
            id: session.id,
            title: session.title,
        })
        .collect::<Vec<_>>();

    info!(
        target: "filter_sessions",
        event = "Got fetch result",
        "session number" = sessions.len()
    );

    ok_json(serde_json::json!({
        "servers": sessions
    }))
}
