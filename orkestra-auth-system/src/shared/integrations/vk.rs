use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, sync::Arc};
use thiserror::Error;
use tracing::Instrument;

#[derive(Clone, Debug)]
pub struct VkService {
    client: reqwest::Client,
    game_id: Arc<str>,
    secret: Arc<str>,
}

impl VkService {
    const BASE_URL: &'static str = "https://vkplay.ru/app";

    pub fn new(game_id: &str, secret: &str) -> Self {
        let client = reqwest::Client::new();

        Self {
            client,
            game_id: Arc::from(game_id),
            secret: Arc::from(secret),
        }
    }

    pub async fn auth(&self, uid: &str, hash: &str, ip: Ipv4Addr) -> Result<(), VkAuthError> {
        let sign = self.calc_sign(serde_json::json!({
            "appid": self.game_id,
            "uid": uid,
            "hash": hash,
            "ip": ip.to_string()
        }));

        let url = format!("{}/{}/gas", Self::BASE_URL, self.game_id);

        let response = self
            .client
            .get(url)
            .query(&[
                ("uid", uid),
                ("hash", hash),
                ("ip", &ip.to_string()),
                ("sign", &sign),
            ])
            .send()
            .in_current_span()
            .await;

        let Ok(response) = response else {
            return Err(VkAuthError::InternalError);
        };

        let body = response.json::<VkResponse<()>>().await.unwrap(/* VK Doc: The server sends a response in JSON format with utf-8 encoding.*/);

        if body.status == "ok" {
            return Ok(());
        }

        let VkResult::Err(error) = body.result.unwrap() else {
            unreachable!();
        };

        match error.errcode {
            0 => Err(VkAuthError::InvalidUserOrSign(error.errmsg)),
            10 => Err(VkAuthError::InvalidHashParameter),
            20 => Err(VkAuthError::WhitelistError),
            30 => Err(VkAuthError::UserWhitelistError),
            40 => Err(VkAuthError::UserIsBanned(error.errmsg)),
            50 => Err(VkAuthError::NoPayment),
            _ => unreachable!(),
        }
    }

    fn calc_sign(&self, json: serde_json::Value) -> String {
        let json = format!("{json}{}", self.secret);
        let digest = md5::compute(json);

        format!("{:x}", digest)
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum VkAuthError {
    #[error("Send request error")]
    InternalError,

    #[error("Invalid user or sign: {0}")]
    InvalidUserOrSign(String),

    #[error("Invalid hash parameter")]
    InvalidHashParameter,

    #[error("Access is restricted to whitelist")]
    WhitelistError,

    #[error("Access is restricted to whitelist")]
    UserWhitelistError,

    #[error("User is banned: {0}")]
    UserIsBanned(String),

    #[error("User has not paid for this game (for P2P games)")]
    NoPayment,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct VkResponse<T> {
    status: String,

    #[serde(flatten)]
    result: Option<VkResult<T>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct VkError {
    errcode: i64,
    errmsg: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
enum VkResult<T> {
    Ok(T),
    Err(VkError),
}

#[derive(Debug, Deserialize)]
struct VkProfileData {
    uid: u64,
    nick: String,
    birthyear: String,
    sex: String,
    slug: String,
}

#[cfg(test)]
mod tests {
    use crate::shared::integrations::vk::{VkError, VkResponse, VkResult};

    #[test]
    fn vk_response_ok_serialize() {
        let json: VkResponse<()> = VkResponse {
            status: "ok".to_string(),
            result: None,
        };

        let data = serde_json::to_string(&json);
        assert!(data.is_ok());
        let data = data.unwrap();

        assert_eq!(data, "{\"status\":\"ok\"}");
    }

    #[test]
    fn vk_response_error_serialize() {
        let json: VkResponse<()> = VkResponse {
            status: "error".to_string(),
            result: Some(VkResult::Err(VkError {
                errcode: 0,
                errmsg: "test".to_string(),
            })),
        };

        let data = serde_json::to_string(&json);
        assert!(data.is_ok());
        let data = data.unwrap();

        assert_eq!(
            data,
            "{\"status\":\"error\",\"errcode\":0,\"errmsg\":\"test\"}"
        );
    }

    #[test]
    fn vk_response_ok_parse() {
        let json = serde_json::json!({
            "status": "ok"
        })
        .to_string();

        let data = serde_json::from_str::<VkResponse<()>>(&json);
        assert!(data.is_ok());

        let data = data.unwrap();

        assert_eq!(data.status, "ok");
        assert_eq!(data.result, None);
    }

    #[test]
    fn vk_response_error_parse() {
        let json = serde_json::json!({
            "status": "error",
            "errcode": 0,
            "errmsg": "gas_invalid_sign"
        })
        .to_string();

        let data = serde_json::from_str::<VkResponse<()>>(&json);
        assert!(data.is_ok());

        let data = data.unwrap();

        assert_eq!(data.status, "error");
        assert_eq!(
            data.result,
            Some(VkResult::Err(VkError {
                errcode: 0,
                errmsg: "gas_invalid_sign".to_string()
            }))
        );
    }
}
