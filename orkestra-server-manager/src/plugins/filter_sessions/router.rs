use axum::{routing::get, Router};

use super::controller::filter_sessions;

pub fn service() -> Router {
    Router::new().route("/filter_sessions", get(filter_sessions))
}
