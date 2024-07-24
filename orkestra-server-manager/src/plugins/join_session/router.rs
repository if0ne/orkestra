use axum::{routing::post, Router};

use crate::shared::services::sesser::Sesser;

use super::controller::join_session;

pub fn service<S: Sesser>() -> Router {
    Router::new().route("/join_session", post(join_session::<S>))
}
