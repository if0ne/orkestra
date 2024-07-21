use axum::{response::IntoResponse, Extension, Json};

use tracing::{error, info, info_span, Instrument};

use crate::{
    plugins::login::use_case,
    shared::{
        context::Context,
        utils::{bad_request_json, just_ok},
    },
};

use super::dto::LoginData;

pub async fn login(
    Extension(context): Extension<Context>,
    Json(request): Json<LoginData>,
) -> impl IntoResponse {
    let span = info_span!("signup");
    let _guard = span.enter();

    info!(event = "Request to login user", username = request.username,);

    let result = use_case::login(context.database(), request)
        .in_current_span()
        .await;

    match result {
        Ok(_) => {
            info!(event = "Successfully login");

            just_ok()
        }
        Err(err) => {
            error!(event = %err);

            bad_request_json(serde_json::json!({
                "error": err.to_string()
            }))
        }
    }
}
