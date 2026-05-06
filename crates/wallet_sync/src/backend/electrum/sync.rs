use wallet_core::{config::SyncBackendConfig, WalletConfig, WalletService};

#[cfg(feature = "electrum")]
use std::time::Duration;
#[cfg(feature = "electrum")]
use std::{sync::mpsc, thread};

use tracing::warn;
#[cfg(feature = "electrum")]
use tracing::{debug, info};

use crate::{WalletSyncError, WalletSyncResult};

fn electrum_url_from_config(config: &WalletConfig) -> WalletSyncResult<&str> {
    match &config.backend.sync {
        SyncBackendConfig::Electrum { url } => Ok(url.as_str()),
        other => Err(WalletSyncError::InvalidBackend(format!(
            "electrum sync requested with non-electrum backend: {:?}",
            other
        ))),
    }
}

/// Perform blockchain synchronization through an Electrum backend.
///
/// This backend is intended primarily for local/regtest or Electrum-compatible
/// server setups. Higher layers should call the sync facade from `service.rs`
/// instead of depending directly on this backend-specific function.
///
/// Note:
/// This implementation uses `bdk_electrum` and maps Electrum client failures
/// into backend-agnostic `WalletSyncError` variants.
#[cfg(feature = "electrum")]
pub(crate) async fn sync_wallet_electrum(
    wallet: &mut WalletService,
    config: &WalletConfig,
) -> WalletSyncResult<()> {
    use bdk_electrum::{electrum_client::Client, BdkElectrumClient};

    let url = electrum_url_from_config(config)?;

    info!("starting electrum sync");
    debug!("electrum_url = {}", url);

    const STOP_GAP: usize = 25;
    const BATCH_SIZE: usize = 50;
    const FETCH_PREV_TXOUTS: bool = false;
    const FULL_SCAN_TIMEOUT: Duration = Duration::from_secs(30);

    debug!(
        "starting full scan: stop_gap = {}, batch_size = {}, fetch_prev_txouts = {}",
        STOP_GAP, BATCH_SIZE, FETCH_PREV_TXOUTS
    );
    let request = wallet.wallet_mut().start_full_scan().build();
    let url = url.to_owned();

    let update = tokio::task::spawn_blocking(move || {
        let (tx, rx) = mpsc::sync_channel(1);

        thread::spawn(move || {
            let result = (|| {
                let client = Client::new(&url)
                    .map_err(|e| WalletSyncError::BackendUnavailable(e.to_string()))?;
                debug!("electrum client created");
                let bdk_client = BdkElectrumClient::new(client);
                bdk_client
                    .full_scan(request, STOP_GAP, BATCH_SIZE, FETCH_PREV_TXOUTS)
                    .map_err(|e| WalletSyncError::SyncFailed(e.to_string()))
            })();

            let _ = tx.send(result);
        });

        rx.recv_timeout(FULL_SCAN_TIMEOUT).map_err(|e| match e {
            mpsc::RecvTimeoutError::Timeout => WalletSyncError::SyncFailed(format!(
                "electrum full scan timed out after {} seconds",
                FULL_SCAN_TIMEOUT.as_secs()
            )),
            mpsc::RecvTimeoutError::Disconnected => WalletSyncError::SyncFailed(
                "electrum full scan worker disconnected before returning an update".to_string(),
            ),
        })?
    })
    .await
    .map_err(|e| WalletSyncError::SyncFailed(format!("electrum sync task failed: {e}")))??;

    wallet
        .wallet_mut()
        .apply_update(update)
        .map_err(|e| WalletSyncError::SyncFailed(e.to_string()))?;
    info!("electrum sync completed successfully");
    wallet.persist()?;

    Ok(())
}

/// Fetch the current Electrum chain tip height without mutating wallet state.
///
/// This is used by backend health checks. It intentionally does not perform a
/// wallet sync, does not apply updates, and does not persist anything.
#[cfg(feature = "electrum")]
pub(crate) fn get_electrum_tip_height(url: &str) -> WalletSyncResult<u32> {
    use bdk_electrum::electrum_client::{Client, ElectrumApi};

    debug!("creating electrum client for health check");

    let client = Client::new(url).map_err(|e| {
        warn!("electrum health check failed while creating client: {}", e);
        WalletSyncError::BackendHealth(format!(
            "failed to create Electrum client for health check: {e}"
        ))
    })?;

    let header_notification = client.block_headers_subscribe().map_err(|e| {
        warn!("electrum health check failed while fetching tip: {}", e);
        WalletSyncError::BackendHealth(format!(
            "failed to fetch Electrum tip height during health check: {e}"
        ))
    })?;

    Ok(header_notification.height as u32)
}

/// Fallback implementation when the crate is built without the `electrum`
/// feature enabled.
#[cfg(not(feature = "electrum"))]
pub(crate) async fn sync_wallet_electrum(
    _wallet: &mut WalletService,
    config: &WalletConfig,
) -> WalletSyncResult<()> {
    let url = electrum_url_from_config(config)?;
    warn!(
        "electrum backend requested but feature is disabled: {}",
        url
    );
    Err(WalletSyncError::BackendUnavailable(format!(
        "electrum backend is configured for '{}' but wallet_sync was built without the 'electrum' feature",
        url
    )))
}

/// Fallback health helper when the crate is built without the `electrum`
/// feature enabled.
#[cfg(not(feature = "electrum"))]
pub(crate) fn get_electrum_tip_height(url: &str) -> WalletSyncResult<u32> {
    warn!(
        "electrum health check requested but feature is disabled: {}",
        url
    );
    Err(WalletSyncError::BackendHealth(format!(
        "electrum backend is configured for '{}' but wallet_sync was built without the 'electrum' feature",
        url
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::Network;
    use std::path::PathBuf;
    use wallet_core::config::{BroadcastBackendConfig, WalletBackendConfig, WalletDescriptors};

    fn electrum_config() -> WalletConfig {
        WalletConfig {
            network: Network::Regtest,
            descriptors: WalletDescriptors {
                external: "tr(tprv8ZgxMBicQKsPeAHm72dLDcYEoJxairbBQgcjg1RSYAngBHT2X3eKd9TP6CSns9Ca6dnUeJE5ZhP5EcysL9mMjAtrTvLbvSd9MKc1CdsWU7B/86h/1h/0h/0/*)#c86zrccx".to_string(),
                internal: "tr(tprv8ZgxMBicQKsPeAHm72dLDcYEoJxairbBQgcjg1RSYAngBHT2X3eKd9TP6CSns9Ca6dnUeJE5ZhP5EcysL9mMjAtrTvLbvSd9MKc1CdsWU7B/86h/1h/0h/1/*)#fnlr7dg7".to_string(),
            },
            backend: WalletBackendConfig {
                sync: SyncBackendConfig::Electrum {
                    url: "tcp://127.0.0.1:60401".to_string(),
                },
                broadcast: Some(BroadcastBackendConfig::Rpc {
                    url: "http://127.0.0.1:18443".to_string(),
                    rpc_user: "bitcoin".to_string(),
                    rpc_pass: "bitcoin".to_string(),
                }),
            },
            db_path: PathBuf::from("/tmp/wallet-sync-electrum-test.db"),
            is_watch_only: false,
        }
    }

    fn esplora_config() -> WalletConfig {
        WalletConfig {
            network: Network::Signet,
            descriptors: WalletDescriptors {
                external: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/0/*)#z3x5097m".to_string(),
                internal: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/1/*)#n9r4jswr".to_string(),
            },
            backend: WalletBackendConfig {
                sync: SyncBackendConfig::Esplora {
                    url: "https://mempool.space/signet/api".to_string(),
                },
                broadcast: Some(BroadcastBackendConfig::Esplora {
                    url: "https://mempool.space/signet/api".to_string(),
                }),
            },
            db_path: PathBuf::from("/tmp/wallet-sync-esplora-test.db"),
            is_watch_only: true,
        }
    }

    #[test]
    fn extracts_electrum_url_from_matching_backend() {
        let config = electrum_config();
        let url = electrum_url_from_config(&config).expect("electrum config should be accepted");
        assert_eq!(url, "tcp://127.0.0.1:60401");
    }

    #[test]
    fn rejects_non_electrum_backend() {
        let config = esplora_config();
        let result = electrum_url_from_config(&config);
        assert!(matches!(result, Err(WalletSyncError::InvalidBackend(_))));
    }

    #[cfg(not(feature = "electrum"))]
    #[tokio::test]
    async fn fallback_returns_backend_unavailable_for_electrum_config() {
        let config = electrum_config();
        let mut wallet =
            wallet_core::WalletService::load_or_create(&config).expect("test wallet should load");

        let result = sync_wallet_electrum(&mut wallet, &config).await;
        assert!(matches!(
            result,
            Err(WalletSyncError::BackendUnavailable(_))
        ));
    }
}
