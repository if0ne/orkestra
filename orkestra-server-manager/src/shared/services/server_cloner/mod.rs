pub mod simple_server_cloner;

use anyhow::Result;

pub trait ServerCloner {
    fn clone_server_repo(&self) -> Result<()>;
}
