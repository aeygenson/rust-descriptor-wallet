use crate::model::WalletBackendCapabilities;
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
    Esplora {
        url: String,
    },
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

    /// Returns backend capability information derived from configured backends.
    pub fn backend_capabilities(&self) -> WalletBackendCapabilities {
        let supports_mempool = matches!(
            self.backend.sync,
            SyncBackendConfig::Esplora { .. }
        );

        let supports_fee_estimates = matches!(
            self.backend.sync,
            SyncBackendConfig::Esplora { .. } | SyncBackendConfig::Electrum { .. }
        );

        WalletBackendCapabilities {
            can_sync: true,
            can_broadcast: self.backend.broadcast.is_some(),
            supports_fee_estimates,
            supports_mempool,
        }
    }

    /// Returns true when the configured wallet can broadcast transactions.
    pub fn can_broadcast(&self) -> bool {
        self.backend_capabilities().can_broadcast
    }

    /// Returns true when the configured wallet can synchronize chain data.
    pub fn can_sync(&self) -> bool {
        self.backend_capabilities().can_sync
    }

    /// Returns true when the configured backend supports fee estimates.
    pub fn supports_fee_estimates(&self) -> bool {
        self.backend_capabilities().supports_fee_estimates
    }

    /// Returns true when the configured backend supports mempool-oriented features.
    pub fn supports_mempool(&self) -> bool {
        self.backend_capabilities().supports_mempool
    }
}


#[cfg(test)]
mod tests {
    use super::{
        BroadcastBackendConfig, SyncBackendConfig, WalletBackendConfig, WalletConfig,
        WalletDescriptors,
    };
    use bitcoin::Network;
    use std::path::PathBuf;

    fn config_with_backend(
        sync: SyncBackendConfig,
        broadcast: Option<BroadcastBackendConfig>,
    ) -> WalletConfig {
        WalletConfig {
            network: Network::Signet,
            descriptors: WalletDescriptors {
                external: "wpkh([00000000/84h/1h/0h]tpub/external/*)".to_string(),
                internal: "wpkh([00000000/84h/1h/0h]tpub/internal/*)".to_string(),
            },
            backend: WalletBackendConfig { sync, broadcast },
            db_path: PathBuf::from("/tmp/test-wallet.db"),
            is_watch_only: true,
        }
    }

    #[test]
    fn esplora_backend_reports_sync_fee_estimates_and_mempool_support() {
        let config = config_with_backend(
            SyncBackendConfig::Esplora {
                url: "https://example.com".to_string(),
            },
            Some(BroadcastBackendConfig::Esplora {
                url: "https://example.com".to_string(),
            }),
        );

        let capabilities = config.backend_capabilities();

        assert!(capabilities.can_sync);
        assert!(capabilities.can_broadcast);
        assert!(capabilities.supports_fee_estimates);
        assert!(capabilities.supports_mempool);
        assert!(config.can_sync());
        assert!(config.can_broadcast());
        assert!(config.supports_fee_estimates());
        assert!(config.supports_mempool());
    }

    #[test]
    fn electrum_backend_reports_fee_estimates_without_mempool_support() {
        let config = config_with_backend(
            SyncBackendConfig::Electrum {
                url: "ssl://example.com:50002".to_string(),
            },
            None,
        );

        let capabilities = config.backend_capabilities();

        assert!(capabilities.can_sync);
        assert!(!capabilities.can_broadcast);
        assert!(capabilities.supports_fee_estimates);
        assert!(!capabilities.supports_mempool);
        assert!(config.can_sync());
        assert!(!config.can_broadcast());
        assert!(config.supports_fee_estimates());
        assert!(!config.supports_mempool());
    }
}
