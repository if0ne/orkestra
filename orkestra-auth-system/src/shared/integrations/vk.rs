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
struct VkError {
    errcode: i64,
    errmsg: String,
}

#[derive(Debug, PartialEq, Eq)]
enum VkResult<T> {
    Ok(T),
    Err(VkError),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct VkProfileData {
    uid: u64,
    nick: String,
    avatar: String,
    birthyear: String,
    sex: String,
    slug: String,
}

#[allow(unused)]
impl<'de, T> Deserialize<'de> for VkResult<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum VkResultInner<T> {
            Ok(T),
            Err(VkError),
        }

        #[derive(Deserialize)]
        struct ResultWrapper<TT> {
            status: String,

            #[serde(flatten)]
            result: Option<VkResultInner<TT>>,
        }

        let wrapper = ResultWrapper::<T>::deserialize(deserializer)?;

        unsafe {
            match wrapper.result {
                Some(VkResultInner::Ok(result)) => Ok(VkResult::Ok(result)),
                Some(VkResultInner::Err(result)) => Ok(VkResult::Err(result)),
                None => Ok(VkResult::Ok(std::mem::zeroed())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::shared::integrations::vk::{VkError, VkProfileData, VkResult};

    #[test]
    fn vk_response_ok_parse() {
        let json = serde_json::json!({
            "status": "ok"
        })
        .to_string();

        let data = serde_json::from_str::<VkResult<()>>(&json);
        assert!(data.is_ok());

        let data = data.unwrap();

        assert_eq!(data, VkResult::Ok(()));
    }

    #[test]
    fn vk_response_structure_parse() {
        let json = serde_json::json!({
            "status": "ok",
            "uid": 0,
            "nick": "test",
            "avatar": "http://test.com/test",
            "birthyear": "01.01.2001",
            "sex": "male",
            "slug": "test"
        })
        .to_string();

        let data = serde_json::from_str::<VkResult<VkProfileData>>(&json);
        assert!(data.is_ok());

        let data = data.unwrap();

        assert_eq!(
            data,
            VkResult::Ok(VkProfileData {
                uid: 0,
                nick: "test".to_string(),
                avatar: "http://test.com/test".to_string(),
                birthyear: "01.01.2001".to_string(),
                sex: "male".to_string(),
                slug: "test".to_string()
            })
        );
    }

    #[test]
    fn vk_response_error_parse() {
        let json = serde_json::json!({
            "status": "error",
            "errcode": 0,
            "errmsg": "gas_invalid_sign"
        })
        .to_string();

        let data = serde_json::from_str::<VkResult<()>>(&json);
        assert!(data.is_ok());

        let data = data.unwrap();

        assert_eq!(
            data,
            VkResult::Err(VkError {
                errcode: 0,
                errmsg: "gas_invalid_sign".to_string()
            })
        );
    }
}
