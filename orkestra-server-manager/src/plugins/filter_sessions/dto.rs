use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct FilterParams {
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionPresent {
    pub id: Uuid,
    pub title: String,
}
