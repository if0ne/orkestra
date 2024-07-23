use axum::{Extension, Router};
use tower_http::trace::TraceLayer;

use crate::plugins::*;

use super::context::Context;

pub fn base_router(router: Router) -> Router {
    Router::new().nest("/auth", router)
}

pub fn v1(context: Context) -> Router {
    let login = login::router::service();
    let signup = signup::router::service();

    let merged = Router::new()
        .merge(login)
        .merge(signup)
        .layer(Extension(context));

    let v1 = Router::new()
        .nest("/v1", merged)
        .layer(TraceLayer::new_for_http());

    base_router(v1)
}
