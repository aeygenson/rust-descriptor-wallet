use wallet_core::{WalletConfig, WalletService};

use crate::esplora_broadcast::EsploraBroadcaster;
use crate::esplora_sync::sync_wallet_esplora;
use crate::WalletSyncResult;

#[derive(Debug, Default, Clone, Copy)]
pub struct WalletSyncService;

impl WalletSyncService {
    pub fn new() -> Self {
        Self
    }

    /// High-level sync entry point used by the API layer
    pub async fn sync(
        &self,
        wallet: &mut WalletService,
        config: &WalletConfig,
    ) -> WalletSyncResult<()> {
        // delegate to esplora backend
        sync_wallet_esplora(wallet, config).await?;

        Ok(())
    }

    /// Build the Esplora broadcaster used by higher layers for tx publication.
    ///
    /// This keeps Esplora-specific construction inside the sync crate so API
    /// and runtime layers do not need to know about concrete backend types.
    pub fn broadcaster(&self, config: &WalletConfig) -> EsploraBroadcaster {
        EsploraBroadcaster::new(config.esplora_url.clone())
    }

    /// Broadcast a raw transaction hex through the configured Esplora backend.
    pub fn broadcast_tx_hex(
        &self,
        config: &WalletConfig,
        tx_hex: &str,
    ) -> WalletSyncResult<()> {
        let broadcaster = self.broadcaster(config);
        broadcaster.broadcast_tx_hex(tx_hex)?;

        Ok(())
    }
}