use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use bitcoin::{Address, BlockHash, Txid};
use tracing::{debug, info};

use crate::bitcoind::{wait_for_rpc_ready, BitcoindConfig};
use crate::paths::{
    regtest_fund_script, regtest_mine_script, regtest_reset_script, regtest_start_script,
    regtest_stop_script,
};
use crate::rpc::{
    block_height, fund_address, fund_address_sats, mine_blocks, miner_balance, ready_miner_client,
};

/// High-level helper for working with the local regtest environment.
#[derive(Debug, Clone)]
pub struct RegtestEnv {
    pub config: BitcoindConfig,
}

impl RegtestEnv {
    /// Construct using environment/default configuration.
    pub fn new() -> Self {
        Self {
            config: BitcoindConfig::from_env(),
        }
    }

    /// Start the regtest services using the project scripts.
    pub fn start(&self) -> Result<()> {
        let script = regtest_start_script()?;
        run_script(&script).context("failed to start regtest services")?;

        // Wait until RPC is ready
        let _ = wait_for_rpc_ready(Duration::from_secs(10))?;
        info!("regtest services are up");
        Ok(())
    }

    /// Stop the regtest services.
    pub fn stop(&self) -> Result<()> {
        let script = regtest_stop_script()?;
        run_script(&script).context("failed to stop regtest services")?;
        info!("regtest services stopped");
        Ok(())
    }

    /// Reset the regtest environment (clean chain/data).
    pub fn reset(&self) -> Result<()> {
        let script = regtest_reset_script()?;
        run_script(&script).context("failed to reset regtest environment")?;
        info!("regtest environment reset");
        Ok(())
    }

    /// Mine `blocks` blocks using RPC.
    pub fn mine(&self, blocks: u64) -> Result<Vec<BlockHash>> {
        info!(blocks, "mining blocks via RPC");
        mine_blocks(blocks)
    }

    /// Mine `blocks` blocks using the provided shell script (if preferred).
    pub fn mine_via_script(&self) -> Result<()> {
        let script = regtest_mine_script()?;
        run_script(&script).context("failed to mine blocks via script")?;
        Ok(())
    }

    /// Fund an address using BTC units.
    pub fn fund_btc(&self, address: &Address, amount_btc: f64) -> Result<Txid> {
        info!(address = %address, amount_btc, "funding address (BTC)");
        fund_address(address, amount_btc)
    }

    /// Fund an address using satoshis.
    pub fn fund_sats(&self, address: &Address, sats: u64) -> Result<Txid> {
        info!(address = %address, sats, "funding address (sats)");
        fund_address_sats(address, sats)
    }

    /// Fund using the shell script helper.
    pub fn fund_via_script(&self, address: &str, amount_btc: f64) -> Result<()> {
        let script = regtest_fund_script()?;
        run_script_with_args(&script, &[address, &amount_btc.to_string()])
            .context("failed to fund via script")?;
        Ok(())
    }

    /// Return current chain height.
    pub fn height(&self) -> Result<u64> {
        block_height()
    }

    /// Return miner wallet balance.
    pub fn miner_balance(&self) -> Result<bitcoin::Amount> {
        miner_balance()
    }

    /// Ensure miner wallet is ready and return RPC client.
    pub fn miner_client(&self) -> Result<bitcoincore_rpc::Client> {
        ready_miner_client()
    }
}

fn run_script(path: &Path) -> Result<()> {
    debug!(script = %path.display(), "running script");
    let status = Command::new("bash")
        .arg(path)
        .status()
        .with_context(|| format!("failed to execute script {}", path.display()))?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "script {} failed with status {}",
            path.display(),
            status
        ));
    }

    Ok(())
}

fn run_script_with_args(path: &Path, args: &[&str]) -> Result<()> {
    debug!(script = %path.display(), ?args, "running script with args");
    let mut cmd = Command::new("bash");
    cmd.arg(path);
    for a in args {
        cmd.arg(a);
    }

    let status = cmd
        .status()
        .with_context(|| format!("failed to execute script {}", path.display()))?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "script {} failed with status {}",
            path.display(),
            status
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct_env() {
        let env = RegtestEnv::new();
        assert!(!env.config.rpc_url.is_empty());
    }
}
