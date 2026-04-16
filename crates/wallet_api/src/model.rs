use serde::{Deserialize, Serialize};
use wallet_core::model::{
    WalletCoinControlInfo, WalletConsolidationInfo, WalletCpfpPsbtInfo, WalletPsbtInfo,
    WalletSignedPsbtInfo, WalletTxInfo, WalletUtxoInfo,
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
    pub descriptors: WalletDescriptorsDto,
    pub backend: WalletBackendDto,
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

/// Coin control options for transaction building
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletCoinControlDto {
    pub include_outpoints: Vec<String>,
    pub exclude_outpoints: Vec<String>,
    pub confirmed_only: bool,
}

// Conversion into core model
impl From<WalletCoinControlDto> for WalletCoinControlInfo {
    fn from(value: WalletCoinControlDto) -> Self {
        Self {
            include_outpoints: value.include_outpoints,
            exclude_outpoints: value.exclude_outpoints,
            confirmed_only: value.confirmed_only,
        }
    }
}

/// DTO strategy used when automatically selecting UTXOs for consolidation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WalletConsolidationStrategyDto {
    SmallestFirst,
    LargestFirst,
    OldestFirst,
}

impl From<WalletConsolidationStrategyDto> for wallet_core::model::WalletConsolidationStrategy {
    fn from(value: WalletConsolidationStrategyDto) -> Self {
        match value {
            WalletConsolidationStrategyDto::SmallestFirst => {
                wallet_core::model::WalletConsolidationStrategy::SmallestFirst
            }
            WalletConsolidationStrategyDto::LargestFirst => {
                wallet_core::model::WalletConsolidationStrategy::LargestFirst
            }
            WalletConsolidationStrategyDto::OldestFirst => {
                wallet_core::model::WalletConsolidationStrategy::OldestFirst
            }
        }
    }
}

impl std::str::FromStr for WalletConsolidationStrategyDto {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "smallest-first" => Ok(Self::SmallestFirst),
            "largest-first" => Ok(Self::LargestFirst),
            "oldest-first" => Ok(Self::OldestFirst),
            other => Err(format!(
                "invalid consolidation strategy '{}'; expected one of: smallest-first, largest-first, oldest-first",
                other
            )),
        }
    }
}

/// Consolidation options for wallet-internal UTXO consolidation flows
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletConsolidationDto {
    pub include_outpoints: Vec<String>,
    pub exclude_outpoints: Vec<String>,
    pub confirmed_only: bool,
    pub max_input_count: Option<usize>,

    pub min_input_count: Option<usize>,
    pub min_utxo_value_sat: Option<u64>,
    pub max_utxo_value_sat: Option<u64>,
    pub max_fee_pct_of_input_value: Option<u8>,
    pub strategy: Option<WalletConsolidationStrategyDto>,
}

// Conversion into core model
impl From<WalletConsolidationDto> for WalletConsolidationInfo {
    fn from(value: WalletConsolidationDto) -> Self {
        Self {
            include_outpoints: value.include_outpoints,
            exclude_outpoints: value.exclude_outpoints,
            confirmed_only: value.confirmed_only,
            max_input_count: value.max_input_count,
            min_input_count: value.min_input_count,
            min_utxo_value_sat: value.min_utxo_value_sat,
            max_utxo_value_sat: value.max_utxo_value_sat,
            max_fee_pct_of_input_value: value.max_fee_pct_of_input_value,
            strategy: value.strategy.map(Into::into),
        }
    }
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
    pub selected_inputs: Vec<String>,
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
            selected_inputs: value.selected_inputs,
            input_count: value.input_count,
            output_count: value.output_count,
            recipient_count: value.recipient_count,
            estimated_vsize: value.estimated_vsize,
        }
    }
}

/// CPFP PSBT information for child-pays-for-parent transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCpfpPsbtDto {
    pub psbt_base64: String,
    pub txid: String,
    pub parent_txid: String,
    pub selected_outpoint: String,
    pub input_value_sat: u64,
    pub child_output_value_sat: u64,
    pub fee_sat: u64,
    pub fee_rate_sat_per_vb: u64,
    pub replaceable: bool,
    pub estimated_vsize: u64,
}

impl From<WalletCpfpPsbtInfo> for WalletCpfpPsbtDto {
    fn from(value: WalletCpfpPsbtInfo) -> Self {
        Self {
            psbt_base64: value.psbt_base64,
            txid: value.txid,
            parent_txid: value.parent_txid,
            selected_outpoint: value.selected_outpoint,
            input_value_sat: value.input_value_sat.as_u64(),
            child_output_value_sat: value.child_output_value_sat.as_u64(),
            fee_sat: value.fee_sat.as_u64(),
            fee_rate_sat_per_vb: value.fee_rate_sat_per_vb,
            replaceable: value.replaceable,
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

/// Broadcast result returned after sending a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxBroadcastResultDto {
    pub txid: String,
    pub replaceable: Option<bool>,
}

/// Import wallet request (from JSON or CLI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWalletDto {
    pub name: String,
    pub network: String,
    pub descriptors: WalletDescriptorsDto,
    pub backend: WalletBackendDto,
    pub is_watch_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDescriptorsDto {
    pub external: String,
    pub internal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBackendDto {
    pub sync: SyncBackendDto,
    pub broadcast: Option<BroadcastBackendDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyncBackendDto {
    Esplora { url: String },
    Electrum { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BroadcastBackendDto {
    Esplora {
        url: String,
    },
    Rpc {
        url: String,
        rpc_user: String,
        rpc_pass: String,
    },
}

// Conversion from storage-layer backend models
impl From<wallet_storage::models::SyncBackendFile> for SyncBackendDto {
    fn from(value: wallet_storage::models::SyncBackendFile) -> Self {
        match value {
            wallet_storage::models::SyncBackendFile::Esplora { url } => Self::Esplora { url },
            wallet_storage::models::SyncBackendFile::Electrum { url } => Self::Electrum { url },
        }
    }
}

impl From<wallet_storage::models::BroadcastBackendFile> for BroadcastBackendDto {
    fn from(value: wallet_storage::models::BroadcastBackendFile) -> Self {
        match value {
            wallet_storage::models::BroadcastBackendFile::Esplora { url } => Self::Esplora { url },
            wallet_storage::models::BroadcastBackendFile::Rpc {
                url,
                rpc_user,
                rpc_pass,
            } => Self::Rpc {
                url,
                rpc_user,
                rpc_pass,
            },
        }
    }
}
