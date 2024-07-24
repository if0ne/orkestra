use axum::{routing::get, Router};

use crate::shared::services::sesser::Sesser;

use super::controller::filter_sessions;

pub fn service<S: Sesser>() -> Router {
    Router::new().route("/filter_sessions", get(filter_sessions::<S>))
}
