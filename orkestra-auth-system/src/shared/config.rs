use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database_host: String,
    pub database_port: u16,
    pub database_username: String,
    pub database_password: String,

    pub port: u16,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        Ok(envy::from_env::<Self>()?)
    }
}
