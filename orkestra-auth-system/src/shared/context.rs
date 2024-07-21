#![allow(unused)]

use std::sync::Arc;

use super::database::Database;

#[derive(Clone)]
pub struct Context {
    inner: Arc<ContextInner>,
}

struct ContextInner {
    database: Database,
}

impl Context {
    pub fn new(database: Database) -> Self {
        Self {
            inner: Arc::new(ContextInner { database }),
        }
    }

    pub fn database(&self) -> &Database {
        &self.inner.database
    }
}

const fn is_send<T: Send>() {}
const fn is_sync<T: Sync>() {}

const SEND_TEST: () = is_send::<Context>();
const SYNC_TEST: () = is_sync::<Context>();
