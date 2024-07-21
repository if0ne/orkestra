use axum::{routing::post, Router};

use super::controller::signup;

pub fn service() -> Router {
    Router::new().route("/signup", post(signup))
}
