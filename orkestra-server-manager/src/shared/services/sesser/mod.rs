use std::future::Future;

use anyhow::Result;
use error::UpdateSessionError;
use uuid::Uuid;

use crate::models::session::{Id, Session, SessionConfig, UpdateSession};

pub mod error;
pub mod inmemory_sesser;

pub trait Sesser: Clone + Send + Sync + 'static {
    fn create_session(
        &self,
        creator_id: Id,
        config: SessionConfig,
    ) -> impl Future<Output = Result<Session>> + Send;

    fn get_by_id(&self, id: Uuid) -> Option<Session>;
    fn get_all_sessions(&self) -> Vec<Session>;

    fn filter_by_code(&self, code: String) -> Vec<Session>;

    fn update_session(
        &self,
        id: Uuid,
        update: UpdateSession,
    ) -> Result<Session, UpdateSessionError>;
}
