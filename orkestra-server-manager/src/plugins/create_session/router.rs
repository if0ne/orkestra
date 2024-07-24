use axum::{routing::post, Router};

use crate::shared::services::sesser::Sesser;

use super::controller::create_session;

pub fn service<S: Sesser>() -> Router {
    Router::new().route("/create_session", post(create_session::<S>))
}
