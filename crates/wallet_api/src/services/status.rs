use std::sync::Arc;

use crate::{ WalletApiResult};

use wallet_core::WalletCore;
use wallet_storage::WalletStorage;

pub async fn status(
    core: &Arc<WalletCore>,
    _storage: &WalletStorage,
) -> WalletApiResult<String> {
    let health = core
        .health_check()?;
    
    Ok(health.to_string())
}