

use std::sync::Arc;

use crate::api::WalletApi;
use crate::{ WalletApiResult};

use tracing::{debug, info};

use wallet_core::WalletCore;
use wallet_storage::WalletStorage;
use wallet_sync::WalletSyncService;

/// Build default WalletApi with all dependencies wired.
pub async fn build_default_api() -> WalletApiResult<WalletApi> {
    info!("building default WalletApi");
    // Core (domain logic)
    let core = Arc::new(WalletCore::new());
    debug!("wallet core created");

    // Storage (SQLite via sqlx)
    debug!("connecting wallet storage");
    let storage = WalletStorage::connect().await?;
    debug!("wallet storage connected");

    // Run migrations once on startup
    debug!("running wallet storage migrations");
    storage
        .migrate()
        .await?;
    debug!("wallet storage migrations complete");

    // Sync service (network sync + broadcast backends)
    let sync = WalletSyncService::new();
    debug!("wallet sync service created");

    info!("default WalletApi built successfully");
    Ok(WalletApi::from_parts(core, storage, sync))
}