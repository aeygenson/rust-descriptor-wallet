pub mod error;

use std::sync::Arc;
pub  use error::WalletSyncError;
pub  type WalletSyncResult<T> = std::result::Result<T, WalletSyncError>;
use wallet_core::WalletCore;
#[derive(Debug)]
pub  struct WalletSync{
    core: Arc<WalletCore>,
}
impl WalletSync {
    pub fn new(core: Arc<WalletCore>) -> Self {
        Self { core }
    }
    pub  async fn sync(&self) -> WalletSyncResult<()> {
        self.core.health_check()?;
        Ok(())
    }
}

