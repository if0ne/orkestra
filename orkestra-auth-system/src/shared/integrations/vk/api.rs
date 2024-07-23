use std::{net::Ipv4Addr, sync::Arc};
use tracing::Instrument;

use super::error::{VkAuthError, VkResult};

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

        let response = response.json::<VkResult<()>>().await.unwrap(/* VK Doc: The server sends a response in JSON format with utf-8 encoding.*/);

        let VkResult::Err(error) = response else {
            return Ok(());
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
