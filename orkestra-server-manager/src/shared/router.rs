use axum::{Extension, Router};
use tower_http::trace::TraceLayer;

use crate::plugins::*;

use super::{context::Context, services::sesser::Sesser};

pub fn base_router(router: Router) -> Router {
    Router::new().nest("/api", router)
}

pub fn v1<S: Sesser + 'static>(context: Context<S>) -> Router {
    let create_session = create_session::router::service();
    let join_session = join_session::router::service();
    let filter_sessions = filter_sessions::router::service();

    let merged = Router::new()
        .merge(create_session)
        .merge(join_session)
        .merge(filter_sessions)
        .layer(Extension(context));

    let v1 = Router::new()
        .nest("/v1", merged)
        .layer(TraceLayer::new_for_http());

    base_router(v1)
}
