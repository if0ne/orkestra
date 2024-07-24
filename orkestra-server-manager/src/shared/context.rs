use std::sync::Arc;

use anyhow::Result;

use super::{config::AppConfig, services::sesser::Sesser};

#[derive(Clone)]
pub struct Context<S: Sesser> {
    inner: Arc<ContextInner<S>>,
}

struct ContextInner<S: Sesser> {
    pub sesser: S,

    pub project_name: String,
    pub repo_path: String,
}

impl<S: Sesser> Context<S> {
    pub fn new(config: &AppConfig, sesser: S) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(ContextInner {
                sesser,
                project_name: config.project_name.clone(),
                repo_path: config.repo_path.clone(),
            }),
        })
    }

    pub fn project_name(&self) -> &str {
        &self.inner.project_name
    }

    pub fn repo_path(&self) -> &str {
        &self.inner.repo_path
    }

    pub fn sesser(&self) -> S {
        self.inner.sesser.clone()
    }
}
