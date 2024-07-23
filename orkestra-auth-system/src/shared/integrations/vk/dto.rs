use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VkAuthData {
    pub uid: String,
    pub hash: String,
}
