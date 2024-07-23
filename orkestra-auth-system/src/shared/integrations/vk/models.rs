use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct VkUserProfileData {
    pub uid: u64,
    pub nick: String,
    pub avatar: String,
    pub birthyear: String,
    pub sex: String,
    pub slug: String,
}
