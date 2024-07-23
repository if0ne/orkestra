use tracing::{debug, info, info_span};

use crate::shared::context::Context;

use super::ServerCloner;

#[derive(Clone)]
pub struct SimplerServerCloner {
    context: Context,
}

impl SimplerServerCloner {
    pub fn new(context: Context) -> Self {
        Self { context }
    }
}

impl ServerCloner for SimplerServerCloner {
    fn clone_server_repo(&self) -> anyhow::Result<()> {
        let span = info_span!("clone_server_repo");
        let _guard = span.enter();

        info!(event = "Start cloning server repository",);

        debug!(event = "Clean up old version of server repository",);

        let _ = std::fs::remove_dir_all(self.context.project_name());

        debug!(event = "Clone server repository",);

        let _ = std::process::Command::new("git")
            .arg("clone")
            .arg(self.context.repo_path())
            .output()?;

        debug!(event = "Fetch lfs files",);

        let _ = std::process::Command::new("git")
            .arg("lfs")
            .arg("fetch")
            .current_dir(self.context.project_name())
            .output()?;

        debug!(event = "Pull lfs files",);

        let _ = std::process::Command::new("git")
            .arg("lfs")
            .arg("pull")
            .current_dir(self.context.project_name())
            .output()?;

        info!(event = "Server repository was successfully downloaded",);

        Ok(())
    }
}
