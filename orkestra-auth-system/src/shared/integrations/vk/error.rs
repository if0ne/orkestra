use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum VkResult<T> {
    Ok(T),
    Err(VkError),
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

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct VkError {
    pub errcode: i64,
    pub errmsg: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VkAuthError {
    #[error("Internal request error")]
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
