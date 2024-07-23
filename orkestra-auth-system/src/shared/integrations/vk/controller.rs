use std::net::SocketAddr;

use axum::{extract::ConnectInfo, response::IntoResponse, Extension, Json};

use tracing::{error, info, info_span, Instrument};

use crate::shared::utils::{bad_request_json, just_ok};

use super::{api::VkService, dto::VkAuthData};

pub async fn auth(
    Extension(context): Extension<VkService>,
    ConnectInfo(ip): ConnectInfo<SocketAddr>,
    Json(request): Json<VkAuthData>,
) -> impl IntoResponse {
    let span = info_span!("vk_auth");
    let _guard = span.enter();

    info!(
        event = "Request to login user in VK",
        username = request.uid
    );

    let SocketAddr::V4(ip) = ip else {
        return bad_request_json(serde_json::json!({
            "error": "Couldn't parse request ip address"
        }));
    };

    info!(
        event = "Extracted request ip address",
        ip = ?ip
    );

    let result = context
        .auth(&request.uid, &request.hash, *ip.ip())
        .in_current_span()
        .await;

    match result {
        Ok(_) => {
            info!(event = "Successfully login");

            just_ok()
        }
        Err(err) => {
            error!(event = %err);

            bad_request_json(serde_json::json!({
                "error": err.to_string()
            }))
        }
    }
}
