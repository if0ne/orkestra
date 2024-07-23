use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct VkAuthData {
    pub uid: String,
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct UserProfileData {
    pub uid: String,
}

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub nickname: String,
}
