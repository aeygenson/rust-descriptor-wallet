use serde::{Deserialize, Serialize};

/// Lightweight wallet summary for listing and UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSummaryDto {
    pub name: String,
    pub network: String,
    pub is_watch_only: bool,
}

/// Detailed wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDetailsDto {
    pub name: String,
    pub network: String,
    pub external_descriptor: String,
    pub internal_descriptor: String,
    pub esplora_url: String,
    pub is_watch_only: bool,
}

/// Transaction information for wallet history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTxDto {
    pub txid: String,
    pub confirmed: bool,
    pub confirmation_height: Option<u32>,
    pub direction: String,
    pub net_value: i64,
    pub fee: Option<u64>,
}

// Conversion from core model
impl From<wallet_core::model::WalletTxInfo> for WalletTxDto {
    fn from(value: wallet_core::model::WalletTxInfo) -> Self {
        Self {
            txid: value.txid,
            confirmed: value.confirmed,
            confirmation_height: value.confirmation_height,
            direction: value.direction,
            net_value: value.net_value,
            fee: value.fee,
        }
    }
}

/// UTXO information for wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletUtxoDto {
    pub outpoint: String,
    pub value: u64,
    pub confirmed: bool,
    pub confirmation_height: Option<u32>,
    pub address: Option<String>,
    pub keychain: String,
}

// Conversion from core model
impl From<wallet_core::model::WalletUtxoInfo> for WalletUtxoDto {
    fn from(value: wallet_core::model::WalletUtxoInfo) -> Self {
        Self {
            outpoint: value.outpoint,
            value: value.value,
            confirmed: value.confirmed,
            confirmation_height: value.confirmation_height,
            address: value.address,
            keychain: value.keychain,
        }
    }
}

/// High-level wallet status for CLI and UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStatusDto {
    pub balance: u64,
    pub utxo_count: usize,
    pub last_block_height: Option<u32>,
}

/// Import wallet request (from JSON or CLI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWalletDto {
    pub name: String,
    pub network: String,
    pub external_descriptor: String,
    pub internal_descriptor: String,
    pub esplora_url: String,
    pub is_watch_only: bool,
}
