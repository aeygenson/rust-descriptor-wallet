use bitcoin::Network;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WalletDescriptors {
    pub external: String,
    pub internal: String,
}

#[derive(Debug, Clone)]
pub enum SyncBackendConfig {
    Esplora { url: String },
    Electrum { url: String },
}

#[derive(Debug, Clone)]
pub enum BroadcastBackendConfig {
    Esplora { url: String },
    Rpc {
        url: String,
        rpc_user: String,
        rpc_pass: String,
    },
}

#[derive(Debug, Clone)]
pub struct WalletBackendConfig {
    pub sync: SyncBackendConfig,
    pub broadcast: Option<BroadcastBackendConfig>,
}

#[derive(Debug, Clone)]
pub struct WalletConfig {
    pub network: Network,
    pub descriptors: WalletDescriptors,
    pub backend: WalletBackendConfig,
    pub db_path: PathBuf,
    /// If true, wallet is watch-only (no private keys, no signing).
    pub is_watch_only: bool,
}

impl WalletConfig {
    pub fn external_descriptor(&self) -> &str {
        &self.descriptors.external
    }

    pub fn internal_descriptor(&self) -> &str {
        &self.descriptors.internal
    }
}
