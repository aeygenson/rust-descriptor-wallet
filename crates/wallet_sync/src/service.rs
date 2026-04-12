use wallet_core::{config::{BroadcastBackendConfig, SyncBackendConfig}, WalletConfig, WalletService};

use crate::backend::{
    core_rpc::broadcast::CoreRpcBroadcaster,
    esplora::{broadcast::EsploraBroadcaster, sync::sync_wallet_esplora},
    electrum::sync::sync_wallet_electrum,
    mock::broadcast::NoopBroadcaster,
};
use crate::model::{BackendProfile, BroadcastBackendKind, SyncBackendKind};
use crate::broadcast::TxBroadcaster;
use crate::WalletSyncResult;
use tracing::{debug, info, warn};

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
        let profile = self.backend_profile(config);
        info!(
            sync = profile.sync_label(),
            broadcast = ?profile.broadcast_label(),
            "starting wallet sync"
        );

        match &config.backend.sync {
            SyncBackendConfig::Esplora { .. } => {
                debug!("using esplora sync backend");
                sync_wallet_esplora(wallet, config).await?
            }
            SyncBackendConfig::Electrum { .. } => {
                debug!("using electrum sync backend");
                sync_wallet_electrum(wallet, config).await?
            }
        }

        info!("wallet sync completed successfully");

        Ok(())
    }

    fn backend_profile(&self, config: &WalletConfig) -> BackendProfile {
        let sync = match &config.backend.sync {
            SyncBackendConfig::Esplora { .. } => SyncBackendKind::Esplora,
            SyncBackendConfig::Electrum { .. } => SyncBackendKind::Electrum,
        };

        let broadcast = config.backend.broadcast.as_ref().map(|b| match b {
            BroadcastBackendConfig::Esplora { .. } => BroadcastBackendKind::Esplora,
            BroadcastBackendConfig::Rpc { .. } => BroadcastBackendKind::CoreRpc,
        });

        BackendProfile::new(sync, broadcast)
    }

    pub fn broadcast_tx_hex(
        &self,
        config: &WalletConfig,
        tx_hex: &str,
    ) -> WalletSyncResult<()> {
        let profile = self.backend_profile(config);
        info!(
            broadcast = ?profile.broadcast_label(),
            "starting transaction broadcast"
        );

        match config.backend.broadcast.as_ref() {
            Some(BroadcastBackendConfig::Esplora { url }) => {
                debug!("using esplora broadcast backend");
                let b = EsploraBroadcaster::new(url.clone());
                b.broadcast_tx_hex(tx_hex)
            }
            Some(BroadcastBackendConfig::Rpc {
                url,
                rpc_user,
                rpc_pass,
            }) => {
                debug!("using core rpc broadcast backend");
                let b = CoreRpcBroadcaster::new(url.clone(), rpc_user.clone(), rpc_pass.clone());
                b.broadcast_tx_hex(tx_hex)
            }
            None => {
                warn!("no broadcast backend configured, using noop broadcaster");
                // fallback mock (useful for tests)
                let b = NoopBroadcaster;
                b.broadcast_tx_hex(tx_hex)
            }
        }
    }
}