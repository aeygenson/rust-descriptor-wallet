use std::sync::Arc;
use std::time::Instant;

use tracing::{debug, info};

use crate::api::WalletApi;
use crate::WalletApiResult;

use wallet_core::WalletCore;
use wallet_storage::WalletStorage;
use wallet_sync::WalletSyncService;

/// Build the default `WalletApi` facade with all dependencies wired.
///
/// Request DTOs are intentionally handled by the API/service boundary; the
/// factory only composes long-lived dependencies.
pub async fn build_default_api() -> WalletApiResult<WalletApi> {
    info!("building default WalletApi");
    let started_at = Instant::now();
    // Core (domain logic)
    let core = Arc::new(WalletCore::new());
    debug!("wallet core created and wrapped in Arc");

    // Storage (SQLite via sqlx)
    debug!("connecting wallet storage");
    let storage = WalletStorage::connect().await?;
    debug!("wallet storage connected successfully");

    // Run migrations once on startup
    debug!("running wallet storage migrations");
    storage.migrate().await?;
    debug!("wallet storage migrations completed successfully");

    // Sync service (network sync + broadcast backends)
    let sync = WalletSyncService::new();
    debug!("wallet sync service created successfully");

    info!(
        elapsed_ms = started_at.elapsed().as_millis(),
        "default WalletApi built successfully"
    );
    Ok(WalletApi::from_parts(core, storage, sync))
}
