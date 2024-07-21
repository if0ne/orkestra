use axum::{routing::post, Router};

use super::controller::login;

pub fn service() -> Router {
    Router::new().route("/login", post(login))
}
