use axum::{routing::get, Extension, Router};
use tower_http::trace::TraceLayer;

use crate::shared::router::base_router;

use super::{
    api::VkService,
    controller::{auth, get_user_profile},
};

pub fn vk_integration(vk_service: VkService) -> Router {
    let app = Router::new()
        .route("/auth", get(auth))
        .route("/user/profile", get(get_user_profile))
        .layer(Extension(vk_service));

    let app = Router::new()
        .nest_service("/vk", app)
        .layer(TraceLayer::new_for_http());

    base_router(app)
}
