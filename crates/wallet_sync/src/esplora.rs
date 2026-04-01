use bdk_esplora::{esplora_client::Builder, EsploraExt};
use wallet_core::{WalletConfig,WalletService};
use crate::{WalletSyncResult};
/// Perform blockchain synchronization through the configured Esplora endpoint.
///
/// This function is internal to the `wallet_sync` crate. The public entry point
/// should remain `WalletSync` from `service.rs` so the rest of the application
/// does not depend directly on Esplora-specific details.
pub(crate) async fn sync_wallet_esplora(
    wallet: &mut WalletService,
    config: &WalletConfig,
)-> WalletSyncResult<()>{
    let client = Builder::new(&config.esplora_url).build_blocking();
    let request = wallet.wallet_mut().start_full_scan().build();
    let update = client
        .full_scan(request,5,25)?;
    wallet.wallet_mut().apply_update(update)?;
    wallet.persist()?;
    Ok(())
}
