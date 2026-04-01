use std::sync::Arc;

use crate::factory::build_default_api;
use crate::services::{runtime, status, wallets};
use crate::WalletApiResult;

use crate::dto::{WalletDetailsDto, WalletSummaryDto};

use wallet_core::WalletCore;
use wallet_storage::WalletStorage;
use wallet_sync::WalletSync;

#[derive(Debug)]
pub struct WalletApi {
    core: Arc<WalletCore>,
    storage: WalletStorage,
    sync: WalletSync,
}

impl WalletApi {
    pub async fn new() -> WalletApiResult<Self> {
        build_default_api().await
    }

    pub fn from_parts(
        core: Arc<WalletCore>,
        storage: WalletStorage,
        sync: WalletSync,
    ) -> Self {
        Self { core, storage, sync }
    }

    pub async fn status(&self) -> WalletApiResult<String> {
        status::status(&self.core, &self.storage).await
    }

    pub async fn list_wallets(&self) -> WalletApiResult<Vec<WalletSummaryDto>> {
        wallets::list_wallets(&self.storage).await
    }

    pub async fn get_wallet(&self, name: &str) -> WalletApiResult<WalletDetailsDto> {
        wallets::get_wallet(&self.storage, name).await
    }

    pub async fn import_wallet(&self, file_path: &str) -> WalletApiResult<()> {
        wallets::import_wallet(&self.storage, file_path).await
    }

    pub async fn delete_wallet(&self, name: &str) -> WalletApiResult<()> {
        wallets::delete_wallet(&self.storage, name).await
    }

    pub async fn address(&self, name: &str) -> WalletApiResult<String> {
        runtime::address(&self.storage, name).await
    }

    pub async fn sync_wallet(&self, name: &str) -> WalletApiResult<()> {
        runtime::sync(&self.storage, name).await
    }

    pub async fn balance(&self, name: &str) -> WalletApiResult<u64> {
        runtime::balance(&self.storage, name).await
    }

    pub fn core(&self) -> &Arc<WalletCore> {
        &self.core
    }

    pub fn storage(&self) -> &WalletStorage {
        &self.storage
    }

    pub fn sync_service(&self) -> &WalletSync {
        &self.sync
    }
}