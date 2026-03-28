use std::sync::Arc;

use crate::{WalletApiError, WalletApiResult};

use wallet_core::WalletCore;
use wallet_storage::WalletStorage;
use wallet_sync::WalletSync;

pub async fn status(
    core: &Arc<WalletCore>,
    _storage: &WalletStorage,
    sync: &WalletSync,
) -> WalletApiResult<String> {
    sync.sync()
        .await?;
    
    let health = core
        .health_check()?;
    
    Ok(health.to_string())
}