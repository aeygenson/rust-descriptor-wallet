use wallet_core::{
    config::{BroadcastBackendConfig, SyncBackendConfig},
    WalletConfig, WalletService,
};

use crate::backend::{
    core_rpc::broadcast::{get_core_rpc_tip_height, CoreRpcBroadcaster},
    electrum::sync::{get_electrum_tip_height, sync_wallet_electrum},
    esplora::{
        broadcast::EsploraBroadcaster,
        sync::{get_esplora_tip_height, sync_wallet_esplora},
    },
    mock::broadcast::NoopBroadcaster,
};
use crate::broadcast::TxBroadcaster;
use crate::model::{BackendProfile, BroadcastBackendKind, SyncBackendKind};
use crate::WalletSyncResult;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Copy)]
struct BackendHealthProbe {
    esplora_tip_height: fn(&str) -> WalletSyncResult<u32>,
    electrum_tip_height: fn(&str) -> WalletSyncResult<u32>,
    core_rpc_tip_height: fn(&str, &str, &str) -> WalletSyncResult<u32>,
}

impl BackendHealthProbe {
    const fn real() -> Self {
        Self {
            esplora_tip_height: get_esplora_tip_height,
            electrum_tip_height: get_electrum_tip_height,
            core_rpc_tip_height: get_core_rpc_tip_height,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WalletSyncService {
    health_probe: BackendHealthProbe,
}

impl Default for WalletSyncService {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletSyncService {
    pub fn new() -> Self {
        Self {
            health_probe: BackendHealthProbe::real(),
        }
    }

    #[cfg(test)]
    fn with_health_probe(health_probe: BackendHealthProbe) -> Self {
        Self { health_probe }
    }

    /// High-level sync entry point used by the API layer
    pub async fn sync(
        &self,
        wallet: &mut WalletService,
        config: &WalletConfig,
    ) -> WalletSyncResult<()> {
        let profile = self.backend_profile(config);
        info!(
            sync = profile.sync_label(),
            broadcast = ?profile.broadcast_label(),
            "starting wallet sync"
        );

        match &config.backend.sync {
            SyncBackendConfig::Esplora { .. } => {
                debug!("using esplora sync backend");
                sync_wallet_esplora(wallet, config).await?
            }
            SyncBackendConfig::Electrum { .. } => {
                debug!("using electrum sync backend");
                sync_wallet_electrum(wallet, config).await?
            }
        }

        info!("wallet sync completed successfully");

        Ok(())
    }

    fn backend_profile(&self, config: &WalletConfig) -> BackendProfile {
        let sync = match &config.backend.sync {
            SyncBackendConfig::Esplora { .. } => SyncBackendKind::Esplora,
            SyncBackendConfig::Electrum { .. } => SyncBackendKind::Electrum,
        };

        let broadcast = config.backend.broadcast.as_ref().map(|b| match b {
            BroadcastBackendConfig::Esplora { .. } => BroadcastBackendKind::Esplora,
            BroadcastBackendConfig::Rpc { .. } => BroadcastBackendKind::CoreRpc,
        });

        BackendProfile::new(sync, broadcast)
    }

    async fn check_esplora_backend(&self, url: &str) -> (bool, Option<u32>, Option<String>) {
        let probe = self.health_probe.esplora_tip_height;
        let url = url.to_string();

        match tokio::task::spawn_blocking(move || probe(&url)).await {
            Ok(Ok(height)) => (true, Some(height), None),
            Ok(Err(err)) => (false, None, Some(err.to_string())),
            Err(err) => (
                false,
                None,
                Some(format!("Esplora health check task failed: {err}")),
            ),
        }
    }

    async fn check_electrum_backend(&self, url: &str) -> (bool, Option<u32>, Option<String>) {
        let probe = self.health_probe.electrum_tip_height;
        let url = url.to_string();

        match tokio::task::spawn_blocking(move || probe(&url)).await {
            Ok(Ok(height)) => (true, Some(height), None),
            Ok(Err(err)) => (false, None, Some(err.to_string())),
            Err(err) => (
                false,
                None,
                Some(format!("Electrum health check task failed: {err}")),
            ),
        }
    }

    async fn check_core_rpc_backend(
        &self,
        rpc_url: &str,
        rpc_user: &str,
        rpc_pass: &str,
    ) -> (bool, Option<u32>, Option<String>) {
        let probe = self.health_probe.core_rpc_tip_height;
        let rpc_url = rpc_url.to_string();
        let rpc_user = rpc_user.to_string();
        let rpc_pass = rpc_pass.to_string();

        match tokio::task::spawn_blocking(move || probe(&rpc_url, &rpc_user, &rpc_pass)).await {
            Ok(Ok(height)) => (true, Some(height), None),
            Ok(Err(err)) => (false, None, Some(err.to_string())),
            Err(err) => (
                false,
                None,
                Some(format!("Bitcoin Core RPC health check task failed: {err}")),
            ),
        }
    }

    pub fn broadcast_tx_hex(&self, config: &WalletConfig, tx_hex: &str) -> WalletSyncResult<()> {
        let profile = self.backend_profile(config);
        info!(
            broadcast = ?profile.broadcast_label(),
            "starting transaction broadcast"
        );

        match config.backend.broadcast.as_ref() {
            Some(BroadcastBackendConfig::Esplora { url }) => {
                debug!("using esplora broadcast backend");
                let b = EsploraBroadcaster::new(url.clone());
                b.broadcast_tx_hex(tx_hex)
            }
            Some(BroadcastBackendConfig::Rpc {
                url,
                rpc_user,
                rpc_pass,
            }) => {
                debug!("using core rpc broadcast backend");
                let b = CoreRpcBroadcaster::new(url.clone(), rpc_user.clone(), rpc_pass.clone());
                b.broadcast_tx_hex(tx_hex)
            }
            None => {
                warn!("no broadcast backend configured, using noop broadcaster");
                // fallback mock (useful for tests)
                let b = NoopBroadcaster;
                b.broadcast_tx_hex(tx_hex)
            }
        }
    }
    /// Lightweight backend health check (no wallet mutation, no sync)
    pub async fn health(
        &self,
        config: &WalletConfig,
    ) -> WalletSyncResult<crate::model::BackendHealth> {
        let profile = self.backend_profile(config);

        debug!(
            sync = profile.sync_label(),
            broadcast = ?profile.broadcast_label(),
            "starting backend health check"
        );

        // --- Sync backend + tip check ---
        let (sync_backend_reachable, bitcoin_tip_reachable, tip_height, mut message) =
            match &config.backend.sync {
                SyncBackendConfig::Esplora { url } => {
                    let (reachable, height, error) = self.check_esplora_backend(url).await;
                    (reachable, height.is_some(), height, error)
                }
                SyncBackendConfig::Electrum { url } => {
                    let (reachable, height, error) = self.check_electrum_backend(url).await;
                    (reachable, height.is_some(), height, error)
                }
            };

        // --- Broadcast backend check ---
        let broadcast_backend_reachable = match config.backend.broadcast.as_ref() {
            Some(BroadcastBackendConfig::Esplora { url }) => {
                let (reachable, _, error) = self.check_esplora_backend(url).await;
                if let Some(error) = error {
                    let current = message.take();
                    message = Some(match current {
                        Some(existing) => format!("{existing}; {error}"),
                        None => error,
                    });
                }
                reachable
            }
            Some(BroadcastBackendConfig::Rpc {
                url,
                rpc_user,
                rpc_pass,
            }) => {
                let (reachable, _, error) =
                    self.check_core_rpc_backend(url, rpc_user, rpc_pass).await;
                if let Some(error) = error {
                    let current = message.take();
                    message = Some(match current {
                        Some(existing) => format!("{existing}; {error}"),
                        None => error,
                    });
                }
                reachable
            }
            None => {
                warn!("no broadcast backend configured");
                let current = message.take();
                message = Some(match current {
                    Some(existing) => format!("{existing}; no broadcast backend configured"),
                    None => "no broadcast backend configured".to_string(),
                });
                false
            }
        };

        let health = crate::model::BackendHealth::new(
            sync_backend_reachable,
            bitcoin_tip_reachable,
            broadcast_backend_reachable,
            tip_height,
            message,
        );

        info!(
            sync = health.sync_backend_reachable,
            tip = health.bitcoin_tip_reachable,
            broadcast = health.broadcast_backend_reachable,
            "backend health check completed"
        );

        Ok(health)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WalletSyncError;

    use std::path::PathBuf;

    use bitcoin::Network;
    use wallet_core::config::{WalletBackendConfig, WalletDescriptors};

    fn ok_height(_: &str) -> WalletSyncResult<u32> {
        Ok(123)
    }

    fn ok_rpc_height(_: &str, _: &str, _: &str) -> WalletSyncResult<u32> {
        Ok(456)
    }

    fn fail_height(_: &str) -> WalletSyncResult<u32> {
        Err(WalletSyncError::BackendHealth("backend down".to_string()))
    }

    fn fail_rpc_height(_: &str, _: &str, _: &str) -> WalletSyncResult<u32> {
        Err(WalletSyncError::BackendHealth("rpc down".to_string()))
    }

    fn config(sync: SyncBackendConfig, broadcast: Option<BroadcastBackendConfig>) -> WalletConfig {
        WalletConfig {
            network: Network::Regtest,
            descriptors: WalletDescriptors {
                external: "wpkh([00000000/84h/1h/0h]tpubDUMMY/0/*)".to_string(),
                internal: "wpkh([00000000/84h/1h/0h]tpubDUMMY/1/*)".to_string(),
            },
            backend: WalletBackendConfig { sync, broadcast },
            db_path: PathBuf::from("/tmp/wallet-sync-health-test.db"),
            is_watch_only: true,
        }
    }

    #[tokio::test]
    async fn health_reports_esplora_sync_and_esplora_broadcast_ok() {
        let service = WalletSyncService::with_health_probe(BackendHealthProbe {
            esplora_tip_height: ok_height,
            electrum_tip_height: fail_height,
            core_rpc_tip_height: fail_rpc_height,
        });

        let config = config(
            SyncBackendConfig::Esplora {
                url: "http://127.0.0.1:3002".to_string(),
            },
            Some(BroadcastBackendConfig::Esplora {
                url: "http://127.0.0.1:3002".to_string(),
            }),
        );

        let health = service
            .health(&config)
            .await
            .expect("health should succeed");

        assert!(health.sync_backend_reachable);
        assert!(health.bitcoin_tip_reachable);
        assert!(health.broadcast_backend_reachable);
        assert_eq!(health.tip_height, Some(123));
        assert_eq!(health.message, None);
    }

    #[tokio::test]
    async fn health_reports_electrum_sync_ok_without_broadcast_backend() {
        let service = WalletSyncService::with_health_probe(BackendHealthProbe {
            esplora_tip_height: fail_height,
            electrum_tip_height: ok_height,
            core_rpc_tip_height: fail_rpc_height,
        });

        let config = config(
            SyncBackendConfig::Electrum {
                url: "tcp://127.0.0.1:60401".to_string(),
            },
            None,
        );

        let health = service
            .health(&config)
            .await
            .expect("health should succeed");

        assert!(health.sync_backend_reachable);
        assert!(health.bitcoin_tip_reachable);
        assert!(!health.broadcast_backend_reachable);
        assert_eq!(health.tip_height, Some(123));
        assert_eq!(
            health.message.as_deref(),
            Some("no broadcast backend configured")
        );
    }

    #[tokio::test]
    async fn health_reports_core_rpc_broadcast_ok() {
        let service = WalletSyncService::with_health_probe(BackendHealthProbe {
            esplora_tip_height: ok_height,
            electrum_tip_height: fail_height,
            core_rpc_tip_height: ok_rpc_height,
        });

        let config = config(
            SyncBackendConfig::Esplora {
                url: "http://127.0.0.1:3002".to_string(),
            },
            Some(BroadcastBackendConfig::Rpc {
                url: "http://127.0.0.1:18443".to_string(),
                rpc_user: "user".to_string(),
                rpc_pass: "pass".to_string(),
            }),
        );

        let health = service
            .health(&config)
            .await
            .expect("health should succeed");

        assert!(health.sync_backend_reachable);
        assert!(health.bitcoin_tip_reachable);
        assert!(health.broadcast_backend_reachable);
        assert_eq!(health.tip_height, Some(123));
        assert_eq!(health.message, None);
    }

    #[tokio::test]
    async fn health_reports_core_rpc_broadcast_failure_message() {
        let service = WalletSyncService::with_health_probe(BackendHealthProbe {
            esplora_tip_height: ok_height,
            electrum_tip_height: fail_height,
            core_rpc_tip_height: fail_rpc_height,
        });

        let config = config(
            SyncBackendConfig::Esplora {
                url: "http://127.0.0.1:3002".to_string(),
            },
            Some(BroadcastBackendConfig::Rpc {
                url: "http://127.0.0.1:18443".to_string(),
                rpc_user: "user".to_string(),
                rpc_pass: "pass".to_string(),
            }),
        );

        let health = service
            .health(&config)
            .await
            .expect("health should succeed");

        assert!(health.sync_backend_reachable);
        assert!(health.bitcoin_tip_reachable);
        assert!(!health.broadcast_backend_reachable);
        assert_eq!(health.tip_height, Some(123));
        assert!(health
            .message
            .as_deref()
            .is_some_and(|message| message.contains("rpc down")));
    }

    #[tokio::test]
    async fn health_reports_sync_backend_failure_but_still_checks_broadcast_backend() {
        let service = WalletSyncService::with_health_probe(BackendHealthProbe {
            esplora_tip_height: fail_height,
            electrum_tip_height: fail_height,
            core_rpc_tip_height: ok_rpc_height,
        });

        let config = config(
            SyncBackendConfig::Esplora {
                url: "http://127.0.0.1:3002".to_string(),
            },
            Some(BroadcastBackendConfig::Rpc {
                url: "http://127.0.0.1:18443".to_string(),
                rpc_user: "user".to_string(),
                rpc_pass: "pass".to_string(),
            }),
        );

        let health = service
            .health(&config)
            .await
            .expect("health should succeed");

        assert!(!health.sync_backend_reachable);
        assert!(!health.bitcoin_tip_reachable);
        assert!(health.broadcast_backend_reachable);
        assert_eq!(health.tip_height, None);
        assert!(health
            .message
            .as_deref()
            .is_some_and(|message| message.contains("backend down")));
    }
}
