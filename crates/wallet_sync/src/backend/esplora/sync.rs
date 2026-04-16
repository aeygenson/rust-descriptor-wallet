use bdk_esplora::{esplora_client::Builder, EsploraExt};
use wallet_core::{config::SyncBackendConfig, WalletConfig, WalletService};

use crate::{WalletSyncError, WalletSyncResult};

use tracing::{debug, info, warn};

/// Perform blockchain synchronization through an Esplora backend.
///
/// This function is internal to the `wallet_sync` crate. Higher layers should
/// call the sync facade from `service.rs` instead of depending directly on this
/// backend-specific implementation.
pub(crate) async fn sync_wallet_esplora(
    wallet: &mut WalletService,
    config: &WalletConfig,
) -> WalletSyncResult<()> {
    let url = match &config.backend.sync {
        SyncBackendConfig::Esplora { url } => url,
        other => {
            return Err(WalletSyncError::InvalidBackend(format!(
                "esplora sync requested with non-esplora backend: {:?}",
                other
            )));
        }
    };

    info!("starting esplora sync");
    debug!("esplora_url = {}", url);

    const PARALLEL_REQUESTS: usize = 5;
    const STOP_GAP: usize = 25;

    debug!("creating esplora client");
    let client = Builder::new(url).build_blocking();
    let request = wallet.wallet_mut().start_full_scan().build();
    debug!(
        "starting full scan: parallel = {}, stop_gap = {}",
        PARALLEL_REQUESTS, STOP_GAP
    );
    let update = client
        .full_scan(request, PARALLEL_REQUESTS, STOP_GAP)
        .map_err(|e| {
            warn!("esplora full_scan failed: {}", e);
            WalletSyncError::SyncFailed(e.to_string())
        })?;

    wallet.wallet_mut().apply_update(update)?;
    info!("esplora sync completed successfully");
    wallet.persist()?;

    Ok(())
}
