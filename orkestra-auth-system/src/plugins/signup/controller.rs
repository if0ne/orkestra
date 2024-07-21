use axum::{response::IntoResponse, Extension, Json};
use tracing::{error, info, info_span, Instrument};

use crate::{
    plugins::signup::use_case,
    shared::{
        context::Context,
        utils::{bad_request_json, just_created},
    },
};

use super::dto::SignupData;

pub async fn signup(
    Extension(context): Extension<Context>,
    Json(request): Json<SignupData>,
) -> impl IntoResponse {
    let span = info_span!("signup");
    let _guard = span.enter();

    info!(
        event = "Request to signup user",
        username = request.username,
    );

    let result = use_case::signup(context.database(), request)
        .in_current_span()
        .await;

    match result {
        Ok(_) => {
            info!(event = "User saved");

            just_created()
        }
        Err(err) => {
            error!(event = %err);

            bad_request_json(serde_json::json!({
                "error": err.to_string()
            }))
        }
    }
}
