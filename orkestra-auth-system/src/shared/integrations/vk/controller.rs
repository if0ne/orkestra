use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Query},
    response::IntoResponse,
    Extension,
};

use tracing::{error, info, info_span, Instrument};

use crate::shared::{
    integrations::vk::dto::UserProfileResponse,
    utils::{bad_request_json, just_ok, ok},
};

use super::{
    api::VkService,
    dto::{UserProfileData, VkAuthData},
};

pub async fn auth(
    Extension(context): Extension<VkService>,
    ConnectInfo(ip): ConnectInfo<SocketAddr>,
    Query(request): Query<VkAuthData>,
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

pub async fn get_user_profile(
    Extension(context): Extension<VkService>,
    Query(request): Query<UserProfileData>,
) -> impl IntoResponse {
    let span = info_span!("vk_user_profile");
    let _guard = span.enter();

    info!(event = "Request user profile in VK", username = request.uid);

    let result = context
        .get_user_profile(&request.uid)
        .in_current_span()
        .await;

    match result {
        Ok(data) => {
            info!(event = "Successfully got user profile");

            let response = UserProfileResponse {
                nickname: data.nick,
            };

            ok(response)
        }
        Err(err) => {
            error!(event = err.errmsg);

            bad_request_json(serde_json::json!({
                "error": err.errmsg
            }))
        }
    }
}
