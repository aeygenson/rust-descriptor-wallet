

use std::sync::Arc;

use wallet_core::{WalletConfig, WalletCore, WalletService};

use crate::{WalletSyncResult};
use crate::esplora::sync_wallet_esplora;

#[derive(Debug)]
pub struct WalletSync {
    core: Arc<WalletCore>,
}

impl WalletSync {
    pub fn new(core: Arc<WalletCore>) -> Self {
        Self { core }
    }

    /// High-level sync entry point used by the API layer
    pub async fn sync(
        &self,
        wallet: &mut WalletService,
        config: &WalletConfig,
    ) -> WalletSyncResult<()> {
        // basic sanity check
        self.core.health_check()?;

        // delegate to esplora backend
        sync_wallet_esplora(wallet, config).await?;

        Ok(())
    }
}