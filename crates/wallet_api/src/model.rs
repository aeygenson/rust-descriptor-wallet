use serde::{Deserialize, Serialize};
use wallet_core::error::WalletCoreError;
use wallet_core::model::{
    WalletBackendCapabilities, WalletBroadcastCandidateInfo, WalletCoinControlInfo,
    WalletConsolidationInfo, WalletConsolidationStrategy, WalletCpfpPsbtInfo,
    WalletInputSelectionConfig, WalletInputSelectionMode, WalletPsbtInfo,
    WalletReceiveAddressInfo, WalletReplacementInfo, WalletSelectionResult, WalletSignedPsbtInfo,
    WalletTxInfo, WalletUtxoInfo,
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

/// Transaction input information for wallet history and parent/child graph inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTxInputDto {
    pub previous_outpoint: String,
}

/// Transaction output information for wallet history and CPFP candidate selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTxOutputDto {
    pub outpoint: String,
    pub value_sat: u64,
    pub address: Option<String>,
    pub is_mine: bool,
    pub keychain: Option<String>,
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
    pub inputs: Vec<WalletTxInputDto>,
    pub outputs: Vec<WalletTxOutputDto>,
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
            inputs: value
                .inputs
                .into_iter()
                .map(|input| WalletTxInputDto {
                    previous_outpoint: input.previous_outpoint.to_string(),
                })
                .collect(),
            outputs: value
                .outputs
                .into_iter()
                .map(|output| WalletTxOutputDto {
                    outpoint: output.outpoint.to_string(),
                    value_sat: output.value.as_u64(),
                    address: output.address,
                    is_mine: output.is_mine,
                    keychain: output
                        .keychain
                        .map(|keychain| keychain.as_str().to_string()),
                })
                .collect(),
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
    pub derivation_index: Option<u32>,
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
            derivation_index: value.derivation_index.map(|index| index.as_u32()),
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

/// Real Bitcoin backend health status for CLI and UI.
///
/// This is intentionally separate from desktop/backend connectivity:
/// desktop connectivity only proves React -> Tauri -> Rust works, while this
/// DTO reports whether the configured Bitcoin infrastructure is reachable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBackendHealthDto {
    pub sync_backend_reachable: bool,
    pub bitcoin_tip_reachable: bool,
    pub broadcast_backend_reachable: bool,
    pub tip_height: Option<u32>,
    pub message: Option<String>,
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

/// Canonical request DTO for creating a standard payment PSBT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePsbtRequestDto {
    pub name: String,
    pub to_address: String,
    pub amount_sat: u64,
    pub fee_rate_sat_per_vb: u64,
    pub replaceable: bool,
    pub coin_control: Option<WalletCoinControlDto>,
}

/// Canonical request DTO for creating and broadcasting a standard payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendRequestDto {
    pub name: String,
    pub to_address: String,
    pub amount_sat: u64,
    pub fee_rate_sat_per_vb: u64,
    pub replaceable: bool,
    pub coin_control: Option<WalletCoinControlDto>,
}

/// Canonical request DTO for send-max PSBT and broadcast flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMaxRequestDto {
    pub name: String,
    pub to_address: String,
    pub fee_rate_sat_per_vb: u64,
    pub replaceable: bool,
    pub coin_control: Option<WalletCoinControlDto>,
}

/// Canonical request DTO for sweep PSBT and broadcast flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepRequestDto {
    pub name: String,
    pub to_address: String,
    pub fee_rate_sat_per_vb: u64,
    pub replaceable: bool,
    pub coin_control: WalletCoinControlDto,
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

/// Canonical request DTO for consolidation PSBT and broadcast flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationRequestDto {
    pub name: String,
    pub fee_rate_sat_per_vb: u64,
    pub replaceable: bool,
    pub consolidation: WalletConsolidationDto,
}

/// Canonical request DTO for signing a PSBT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignPsbtRequestDto {
    pub name: String,
    pub psbt_base64: String,
}

/// Canonical request DTO for publishing a finalized PSBT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishPsbtRequestDto {
    pub name: String,
    pub psbt_base64: String,
}

/// Canonical request DTO for creating an RBF replacement PSBT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BumpFeeRequestDto {
    pub name: String,
    pub txid: String,
    pub fee_rate_sat_per_vb: u64,
}

/// Canonical request DTO for creating a CPFP child PSBT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpfpRequestDto {
    pub name: String,
    pub parent_txid: String,
    pub selected_outpoint: String,
    pub fee_rate_sat_per_vb: u64,
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

/// Canonical request DTO for reading wallet transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTransactionsRequestDto {
    pub name: String,
}

/// Canonical request DTO for reading wallet UTXOs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletUtxosRequestDto {
    pub name: String,
}

/// Canonical request DTO for revealing the next receive address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddressRequestDto {
    pub name: String,
}

/// Canonical request DTO for importing a wallet from a JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWalletRequestDto {
    pub file_path: String,
}

/// Canonical request DTO for deleting a wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWalletRequestDto {
    pub name: String,
}

/// Canonical request DTO for retrieving wallet details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletRequestDto {
    pub name: String,
}

/// API DTO for a generated or discovered receive address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletReceiveAddressDto {
    pub address: String,
    pub keychain: String,
    pub index: Option<u32>,
}

impl From<WalletReceiveAddressInfo> for WalletReceiveAddressDto {
    fn from(info: WalletReceiveAddressInfo) -> Self {
        Self {
            address: info.address,
            keychain: info.keychain.as_str().to_string(),
            index: info.index.map(|i| i.as_u32()),
        }
    }
}

/// API DTO for rich input-selection metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSelectionResultDto {
    pub selected_outpoints: Vec<String>,
    pub auto_selected_count: usize,
    pub manual_selected_count: usize,
    pub excluded_count: usize,
    pub strategy_used: Option<String>,
}

impl From<WalletSelectionResult> for WalletSelectionResultDto {
    fn from(info: WalletSelectionResult) -> Self {
        Self {
            selected_outpoints: info
                .selected_outpoints
                .into_iter()
                .map(|op| op.to_string())
                .collect(),
            auto_selected_count: info.auto_selected_count,
            manual_selected_count: info.manual_selected_count,
            excluded_count: info.excluded_count,
            strategy_used: info.strategy_used.map(consolidation_strategy_to_string),
        }
    }
}

fn consolidation_strategy_to_string(strategy: WalletConsolidationStrategy) -> String {
    match strategy {
        WalletConsolidationStrategy::SmallestFirst => "smallest-first".to_string(),
        WalletConsolidationStrategy::LargestFirst => "largest-first".to_string(),
        WalletConsolidationStrategy::OldestFirst => "oldest-first".to_string(),
    }
}

/// API DTO for backend capability metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBackendCapabilitiesDto {
    pub can_sync: bool,
    pub can_broadcast: bool,
    pub supports_fee_estimates: bool,
    pub supports_mempool: bool,
}

impl From<WalletBackendCapabilities> for WalletBackendCapabilitiesDto {
    fn from(info: WalletBackendCapabilities) -> Self {
        Self {
            can_sync: info.can_sync,
            can_broadcast: info.can_broadcast,
            supports_fee_estimates: info.supports_fee_estimates,
            supports_mempool: info.supports_mempool,
        }
    }
}

/// API DTO for richer broadcast candidate analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBroadcastCandidateDto {
    pub txid: String,
    pub tx_hex: String,
    pub replaceable: bool,
    pub fee: Option<u64>,
    pub fee_rate_sat_per_vb: Option<u64>,
    pub vsize: Option<u64>,
    pub ancestor_count: Option<u32>,
    pub descendant_count: Option<u32>,
}

impl From<WalletBroadcastCandidateInfo> for WalletBroadcastCandidateDto {
    fn from(info: WalletBroadcastCandidateInfo) -> Self {
        Self {
            txid: info.txid.to_string(),
            tx_hex: info.tx_hex.to_string(),
            replaceable: info.replaceable,
            fee: info.fee.map(|f| f.as_u64()),
            fee_rate_sat_per_vb: info.fee_rate.map(|f| f.as_u64()),
            vsize: info.vsize.map(|v| v.as_u64()),
            ancestor_count: info.ancestor_count,
            descendant_count: info.descendant_count,
        }
    }
}

/// API DTO for RBF/replacement transaction lineage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletReplacementDto {
    pub replaced_txid: String,
    pub replacement_txid: String,
    pub replacement_depth: u32,
    pub replacement_chain: Vec<String>,
}

impl From<WalletReplacementInfo> for WalletReplacementDto {
    fn from(info: WalletReplacementInfo) -> Self {
        Self {
            replaced_txid: info.replaced_txid.to_string(),
            replacement_txid: info.replacement_txid.to_string(),
            replacement_depth: info.replacement_depth,
            replacement_chain: info
                .replacement_chain
                .into_iter()
                .map(|txid| txid.to_string())
                .collect(),
        }
    }
}

/// PSBT information for unsigned transaction creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletPsbtDto {
    pub psbt_base64: String,
    pub txid: String,
    pub original_txid: Option<String>,
    pub replacement: Option<WalletReplacementDto>,
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
            replacement: value.replacement.map(Into::into),
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

/// Full wallet import payload (the JSON file content used by storage import/export flows).
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
