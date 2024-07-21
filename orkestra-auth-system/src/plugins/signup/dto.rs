use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SignupData {
    pub username: String,
    pub password: String,
}
