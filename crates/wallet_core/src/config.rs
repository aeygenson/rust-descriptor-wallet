use bitcoin::Network;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WalletConfig {
    pub network: Network,
    pub external_descriptor: String,
    pub internal_descriptor: String,
    pub db_path: PathBuf,
    pub esplora_url: String,
    /// If true, wallet is watch-only (no private keys, no signing).
    pub is_watch_only: bool,
}
