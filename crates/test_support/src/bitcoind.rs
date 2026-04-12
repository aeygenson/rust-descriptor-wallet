

use std::env;
use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use bitcoincore_rpc::{Auth, Client, RpcApi};

const DEFAULT_RPC_HOST: &str = "http://127.0.0.1:18443";
const DEFAULT_RPC_USER: &str = "bitcoin";
const DEFAULT_RPC_PASS: &str = "bitcoin";
const DEFAULT_MINER_WALLET: &str = "miner";

/// Runtime settings for talking to the local regtest `bitcoind` instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitcoindConfig {
    pub rpc_url: String,
    pub rpc_user: String,
    pub rpc_pass: String,
    pub miner_wallet: String,
}

impl BitcoindConfig {
    /// Build config from environment variables, falling back to the local
    /// regtest defaults used by `infra/regtest`.
    pub fn from_env() -> Self {
        Self {
            rpc_url: env::var("WALLET_REGTEST_RPC_URL")
                .unwrap_or_else(|_| DEFAULT_RPC_HOST.to_string()),
            rpc_user: env::var("WALLET_REGTEST_RPC_USER")
                .unwrap_or_else(|_| DEFAULT_RPC_USER.to_string()),
            rpc_pass: env::var("WALLET_REGTEST_RPC_PASS")
                .unwrap_or_else(|_| DEFAULT_RPC_PASS.to_string()),
            miner_wallet: env::var("WALLET_REGTEST_MINER_WALLET")
                .unwrap_or_else(|_| DEFAULT_MINER_WALLET.to_string()),
        }
    }

    /// Build an RPC client for the base node endpoint.
    pub fn client(&self) -> Result<Client> {
        Client::new(
            &self.rpc_url,
            Auth::UserPass(self.rpc_user.clone(), self.rpc_pass.clone()),
        )
        .with_context(|| format!("failed to create bitcoind RPC client for {}", self.rpc_url))
    }

    /// Build an RPC client scoped to the configured miner wallet.
    pub fn miner_wallet_client(&self) -> Result<Client> {
        let wallet_url = format!("{}/wallet/{}", self.rpc_url, self.miner_wallet);
        Client::new(
            &wallet_url,
            Auth::UserPass(self.rpc_user.clone(), self.rpc_pass.clone()),
        )
        .with_context(|| {
            format!(
                "failed to create miner wallet RPC client for {}",
                wallet_url
            )
        })
    }
}

/// Create a base RPC client using environment/default settings.
pub fn rpc_client() -> Result<Client> {
    BitcoindConfig::from_env().client()
}

/// Create an RPC client for the configured miner wallet.
pub fn miner_wallet_client() -> Result<Client> {
    BitcoindConfig::from_env().miner_wallet_client()
}

/// Wait until the local regtest node responds to `getblockchaininfo`.
pub fn wait_for_rpc_ready(timeout: Duration) -> Result<Client> {
    wait_for_rpc_ready_with_config(&BitcoindConfig::from_env(), timeout)
}

/// Wait until the supplied bitcoind config responds to `getblockchaininfo`.
pub fn wait_for_rpc_ready_with_config(
    config: &BitcoindConfig,
    timeout: Duration,
) -> Result<Client> {
    let deadline = Instant::now() + timeout;
    let mut last_error: Option<anyhow::Error> = None;

    while Instant::now() < deadline {
        match config.client() {
            Ok(client) => match client.get_blockchain_info() {
                Ok(_) => return Ok(client),
                Err(e) => last_error = Some(anyhow!(e).context("bitcoind RPC not ready yet")),
            },
            Err(e) => last_error = Some(e),
        }

        sleep(Duration::from_millis(250));
    }

    Err(last_error.unwrap_or_else(|| anyhow!("timed out waiting for bitcoind RPC")))
}

/// Return true when `bitcoind` RPC is currently reachable.
pub fn is_rpc_ready() -> bool {
    rpc_client()
        .and_then(|client| client.get_blockchain_info().map(|_| ()).map_err(Into::into))
        .is_ok()
}

/// Ensure the configured miner wallet is loaded.
pub fn ensure_miner_wallet_loaded(client: &Client, wallet_name: &str) -> Result<()> {
    let wallets = client
        .list_wallets()
        .context("failed to list loaded wallets")?;

    if wallets.iter().any(|loaded| loaded == wallet_name) {
        return Ok(());
    }

    client
        .load_wallet(wallet_name)
        .with_context(|| format!("failed to load miner wallet '{}'", wallet_name))?;

    Ok(())
}

/// Return the current best block height.
pub fn current_block_height(client: &Client) -> Result<u64> {
    let info = client
        .get_blockchain_info()
        .context("failed to fetch blockchain info")?;
    Ok(info.blocks as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_local_regtest_defaults() {
        let config = BitcoindConfig::from_env();

        assert!(!config.rpc_url.is_empty());
        assert!(!config.rpc_user.is_empty());
        assert!(!config.rpc_pass.is_empty());
        assert!(!config.miner_wallet.is_empty());
    }
}