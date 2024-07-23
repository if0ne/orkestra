use axum::{routing::post, Router};

use super::controller::join_session;

pub fn service() -> Router {
    Router::new().route("/join_session", post(join_session))
}
