use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,

    pub project_name: String,
    pub repo_path: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        Ok(envy::from_env::<Self>()?)
    }
}
