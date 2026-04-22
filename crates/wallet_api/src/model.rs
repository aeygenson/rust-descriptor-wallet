use serde::{Deserialize, Serialize};
use wallet_core::error::WalletCoreError;
use wallet_core::model::{
    WalletCoinControlInfo, WalletConsolidationInfo, WalletCpfpPsbtInfo, WalletInputSelectionConfig,
    WalletInputSelectionMode, WalletPsbtInfo, WalletSignedPsbtInfo, WalletTxInfo, WalletUtxoInfo,
};
use wallet_core::types::WalletOutPoint;

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
            txid: value.txid.to_string(),
            confirmed: value.confirmed,
            confirmation_height: value.confirmation_height.map(|h| h.as_u32()),
            direction: value.direction.as_str().to_string(),
            replaceable: value.replaceable,
            net_value: value.net_value,
            fee: value.fee.map(Into::into),
            fee_rate_sat_per_vb: value.fee_rate_sat_per_vb.map(|v| v.as_u64()),
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
            outpoint: value.outpoint.to_string(),
            value: value.value.as_u64(),
            confirmed: value.confirmed,
            confirmation_height: value.confirmation_height.map(|h| h.as_u32()),
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

/// DTO input-selection mode used by coin-control and consolidation APIs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WalletInputSelectionModeDto {
    StrictManual,
    ManualWithAutoCompletion,
    AutomaticOnly,
}

impl From<WalletInputSelectionModeDto> for WalletInputSelectionMode {
    fn from(value: WalletInputSelectionModeDto) -> Self {
        match value {
            WalletInputSelectionModeDto::StrictManual => WalletInputSelectionMode::StrictManual,
            WalletInputSelectionModeDto::ManualWithAutoCompletion => {
                WalletInputSelectionMode::ManualWithAutoCompletion
            }
            WalletInputSelectionModeDto::AutomaticOnly => WalletInputSelectionMode::AutomaticOnly,
        }
    }
}

impl From<WalletInputSelectionMode> for WalletInputSelectionModeDto {
    fn from(value: WalletInputSelectionMode) -> Self {
        match value {
            WalletInputSelectionMode::StrictManual => Self::StrictManual,
            WalletInputSelectionMode::ManualWithAutoCompletion => Self::ManualWithAutoCompletion,
            WalletInputSelectionMode::AutomaticOnly => Self::AutomaticOnly,
        }
    }
}

impl std::str::FromStr for WalletInputSelectionModeDto {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "strict-manual" => Ok(Self::StrictManual),
            "manual-with-auto-completion" => Ok(Self::ManualWithAutoCompletion),
            "automatic-only" => Ok(Self::AutomaticOnly),
            other => Err(format!(
                "invalid input selection mode '{}'; expected one of: strict-manual, manual-with-auto-completion, automatic-only",
                other
            )),
        }
    }
}

/// Coin control options for transaction building
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletCoinControlDto {
    pub include_outpoints: Vec<String>,
    pub exclude_outpoints: Vec<String>,
    pub confirmed_only: bool,
    pub selection_mode: Option<WalletInputSelectionModeDto>,
}

impl WalletCoinControlDto {
    /// Convert caller-provided outpoint strings into typed core values.
    ///
    /// DTOs are user/API input, so malformed outpoints must become API errors
    /// instead of panicking inside transaction construction.
    pub fn try_into_core(self) -> Result<WalletCoinControlInfo, WalletCoreError> {
        Ok(WalletCoinControlInfo {
            selection: WalletInputSelectionConfig {
                include_outpoints: parse_outpoints(
                    self.include_outpoints,
                    "WalletCoinControlDto.include_outpoints",
                )?,
                exclude_outpoints: parse_outpoints(
                    self.exclude_outpoints,
                    "WalletCoinControlDto.exclude_outpoints",
                )?,
                confirmed_only: self.confirmed_only,
                selection_mode: self.selection_mode.map(Into::into),
                max_input_count: None,
                min_input_count: None,
                min_utxo_value_sat: None,
                max_utxo_value_sat: None,
                strategy: None,
            },
        })
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
    pub selection_mode: Option<WalletInputSelectionModeDto>,
}

impl WalletConsolidationDto {
    /// Convert caller-provided consolidation controls into typed core values.
    pub fn try_into_core(self) -> Result<WalletConsolidationInfo, WalletCoreError> {
        Ok(WalletConsolidationInfo {
            selection: WalletInputSelectionConfig {
                include_outpoints: parse_outpoints(
                    self.include_outpoints,
                    "WalletConsolidationDto.include_outpoints",
                )?,
                exclude_outpoints: parse_outpoints(
                    self.exclude_outpoints,
                    "WalletConsolidationDto.exclude_outpoints",
                )?,
                confirmed_only: self.confirmed_only,
                selection_mode: self.selection_mode.map(Into::into),
                max_input_count: self.max_input_count,
                min_input_count: self.min_input_count,
                min_utxo_value_sat: self.min_utxo_value_sat,
                max_utxo_value_sat: self.max_utxo_value_sat,
                strategy: self.strategy.map(Into::into),
            },
            max_fee_pct_of_input_value: self
                .max_fee_pct_of_input_value
                .map(wallet_core::types::Percent::from),
        })
    }
}

fn parse_outpoints(
    outpoints: Vec<String>,
    field_name: &str,
) -> Result<Vec<WalletOutPoint>, WalletCoreError> {
    outpoints
        .into_iter()
        .map(|s| {
            WalletOutPoint::parse(&s).map_err(|_| {
                WalletCoreError::CoinControlInvalidOutpoint(format!("{field_name}: {s}"))
            })
        })
        .collect()
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
            psbt_base64: value.psbt_base64.to_string(),
            txid: value.txid.to_string(),
            original_txid: value.original_txid.map(|txid| txid.to_string()),
            to_address: value.to_address,
            amount_sat: value.amount_sat.as_u64(),
            fee_sat: value.fee_sat.as_u64(),
            fee_rate_sat_per_vb: value.fee_rate_sat_per_vb.as_u64(),
            replaceable: value.replaceable,
            change_amount_sat: value.change_amount_sat.map(|v| v.as_u64()),
            selected_utxo_count: value.selected_utxo_count,
            selected_inputs: value
                .selected_inputs
                .into_iter()
                .map(|op| op.to_string())
                .collect(),
            input_count: value.input_count,
            output_count: value.output_count,
            recipient_count: value.recipient_count,
            estimated_vsize: value.estimated_vsize.as_u64(),
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
            psbt_base64: value.psbt_base64.to_string(),
            txid: value.txid.to_string(),
            parent_txid: value.parent_txid.to_string(),
            selected_outpoint: value.selected_outpoint.to_string(),
            input_value_sat: value.input_value_sat.as_u64(),
            child_output_value_sat: value.child_output_value_sat.as_u64(),
            fee_sat: value.fee_sat.as_u64(),
            fee_rate_sat_per_vb: value.fee_rate_sat_per_vb.as_u64(),
            replaceable: value.replaceable,
            estimated_vsize: value.estimated_vsize.as_u64(),
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
            psbt_base64: value.psbt_base64.to_string(),
            modified: value.modified,
            finalized: value.finalized,
            txid: value.txid.to_string(),
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
