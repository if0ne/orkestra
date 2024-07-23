use axum::{routing::post, Router};

use super::controller::create_session;

pub fn service() -> Router {
    Router::new().route("/create_session", post(create_session))
}
