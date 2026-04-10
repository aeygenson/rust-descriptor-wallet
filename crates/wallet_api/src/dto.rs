use serde::{Deserialize, Serialize};
use wallet_core::model::{
    WalletPsbtInfo,
    WalletPublishedTxInfo,
    WalletSignedPsbtInfo,
    WalletTxInfo,
    WalletUtxoInfo,
};

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
    pub replaceable: bool,
    pub net_value: i64,
    pub fee: Option<u64>,
    pub fee_rate_sat_per_vb: Option<u64>,
}

// Conversion from core model
impl From<WalletTxInfo> for WalletTxDto {
    fn from(value: WalletTxInfo) -> Self {
        Self {
            txid: value.txid,
            confirmed: value.confirmed,
            confirmation_height: value.confirmation_height,
            direction: value.direction.as_str().to_string(),
            replaceable: value.replaceable,
            net_value: value.net_value,
            fee: value.fee.map(Into::into),
            fee_rate_sat_per_vb: value.fee_rate_sat_per_vb,
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
impl From<WalletUtxoInfo> for WalletUtxoDto {
    fn from(value: WalletUtxoInfo) -> Self {
        Self {
            outpoint: value.outpoint,
            value: value.value.as_u64(),
            confirmed: value.confirmed,
            confirmation_height: value.confirmation_height,
            address: value.address,
            keychain: value.keychain.as_str().to_string(),
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

/// PSBT information for unsigned transaction creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletPsbtDto {
    pub psbt_base64: String,
    pub txid: String,
    pub original_txid: Option<String>,
    pub to_address: String,
    pub amount_sat: u64,
    pub fee_sat: u64,
    pub fee_rate_sat_per_vb: u64,
    pub replaceable: bool,
    pub change_amount_sat: Option<u64>,
    pub selected_utxo_count: usize,
    pub input_count: usize,
    pub output_count: usize,
    pub recipient_count: usize,
    pub estimated_vsize: u64,
}

impl From<WalletPsbtInfo> for WalletPsbtDto {
    fn from(value: WalletPsbtInfo) -> Self {
        Self {
            psbt_base64: value.psbt_base64,
            txid: value.txid,
            original_txid: value.original_txid,
            to_address: value.to_address,
            amount_sat: value.amount_sat.as_u64(),
            fee_sat: value.fee_sat.as_u64(),
            fee_rate_sat_per_vb: value.fee_rate_sat_per_vb,
            replaceable: value.replaceable,
            change_amount_sat: value.change_amount_sat.map(|v| v.as_u64()),
            selected_utxo_count: value.selected_utxo_count,
            input_count: value.input_count,
            output_count: value.output_count,
            recipient_count: value.recipient_count,
            estimated_vsize: value.estimated_vsize,
        }
    }
}

/// Signed PSBT information returned after signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSignedPsbtDto {
    pub psbt_base64: String,
    pub modified: bool,
    pub finalized: bool,
    pub txid: String,
    pub signing_status: String,
}

impl From<WalletSignedPsbtInfo> for WalletSignedPsbtDto {
    fn from(value: WalletSignedPsbtInfo) -> Self {
        let signing_status = value.signing_status().as_str().to_string();

        Self {
            psbt_base64: value.psbt_base64,
            modified: value.modified,
            finalized: value.finalized,
            txid: value.txid,
            signing_status,
        }
    }
}

/// Published transaction information returned after broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletPublishedTxDto {
    pub txid: String,
    pub replaceable: Option<bool>,
}

impl From<WalletPublishedTxInfo> for WalletPublishedTxDto {
    fn from(value: WalletPublishedTxInfo) -> Self {
        Self {
            txid: value.txid,
            replaceable: value.replaceable,
        }
    }
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
