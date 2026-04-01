

use std::sync::Arc;

use crate::api::WalletApi;
use crate::{ WalletApiResult};

use wallet_core::WalletCore;
use wallet_storage::WalletStorage;
use wallet_sync::WalletSync;

/// Build default WalletApi with all dependencies wired.
pub async fn build_default_api() -> WalletApiResult<WalletApi> {
    // Core (domain logic)
    let core = Arc::new(WalletCore::new());

    // Storage (SQLite via sqlx)
    let storage = WalletStorage::connect()
        .await?;

    // Run migrations once on startup
    storage
        .migrate()
        .await?;

    // Sync service (network / esplora)
    let sync = WalletSync::new(Arc::clone(&core));

    Ok(WalletApi::from_parts(core, storage, sync))
}