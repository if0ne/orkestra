use std::future::Future;

use anyhow::Result;
use uuid::Uuid;

use crate::models::session::{Session, SessionConfig};

pub mod inmemory_sesser;

pub trait Sesser: Clone + Send + Sync + 'static {
    fn create_session(&self, config: SessionConfig)
        -> impl Future<Output = Result<Session>> + Send;

    fn get_by_id(&self, id: Uuid) -> Option<Session>;
    fn get_all_sessions(&self) -> Vec<Session>;

    fn filter_by_code(&self, code: String) -> Vec<Session>;
}
