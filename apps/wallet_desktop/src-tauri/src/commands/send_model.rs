use wallet_api::model::{
    WalletCoinControlDto, WalletConsolidationDto, WalletConsolidationStrategyDto,
    WalletInputSelectionModeDto,
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePsbtRequest {
    pub wallet_name: String,
    pub address: String,
    pub amount_sat: u64,
    pub fee_rate_sat_vb: u64,
    pub replaceable: bool,
    pub confirmed_only: bool,
}

impl CreatePsbtRequest {
    pub fn into_parts(self) -> (String, String, u64, u64, bool, bool) {
        (
            self.wallet_name,
            self.address,
            self.amount_sat,
            self.fee_rate_sat_vb,
            self.replaceable,
            self.confirmed_only,
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinControlRequest {
    pub include_outpoints: Vec<String>,
    pub exclude_outpoints: Vec<String>,
    pub confirmed_only: bool,
    pub selection_mode: Option<WalletInputSelectionModeDto>,
}

impl From<CoinControlRequest> for WalletCoinControlDto {
    fn from(request: CoinControlRequest) -> Self {
        Self {
            include_outpoints: request.include_outpoints,
            exclude_outpoints: request.exclude_outpoints,
            confirmed_only: request.confirmed_only,
            selection_mode: request.selection_mode,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePsbtWithCoinControlRequest {
    pub wallet_name: String,
    pub address: String,
    pub amount_sat: u64,
    pub fee_rate_sat_vb: u64,
    pub replaceable: bool,
    pub confirmed_only: bool,
    pub coin_control: CoinControlRequest,
}

impl CreatePsbtWithCoinControlRequest {
    pub fn into_parts(self) -> (String, String, u64, u64, bool, bool, WalletCoinControlDto) {
        (
            self.wallet_name,
            self.address,
            self.amount_sat,
            self.fee_rate_sat_vb,
            self.replaceable,
            self.confirmed_only,
            self.coin_control.into(),
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMaxRequestBase {
    pub wallet_name: String,
    pub address: String,
    pub fee_rate_sat_vb: u64,
    pub replaceable: bool,
}

impl SendMaxRequestBase {
    pub fn into_parts(self) -> (String, String, u64, bool) {
        (
            self.wallet_name,
            self.address,
            self.fee_rate_sat_vb,
            self.replaceable,
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSendMaxPsbtRequest {
    #[serde(flatten)]
    pub base: SendMaxRequestBase,
}

impl CreateSendMaxPsbtRequest {
    pub fn into_parts(self) -> (String, String, u64, bool) {
        self.base.into_parts()
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSendMaxPsbtWithCoinControlRequest {
    pub wallet_name: String,
    pub address: String,
    pub fee_rate_sat_vb: u64,
    pub replaceable: bool,
    pub coin_control: CoinControlRequest,
}

impl CreateSendMaxPsbtWithCoinControlRequest {
    pub fn into_parts(self) -> (String, String, u64, bool, WalletCoinControlDto) {
        (
            self.wallet_name,
            self.address,
            self.fee_rate_sat_vb,
            self.replaceable,
            self.coin_control.into(),
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SweepRequestBase {
    pub wallet_name: String,
    pub address: String,
    pub fee_rate_sat_vb: u64,
    pub replaceable: bool,
    pub coin_control: CoinControlRequest,
}

impl SweepRequestBase {
    pub fn into_parts(self) -> (String, String, u64, bool, WalletCoinControlDto) {
        (
            self.wallet_name,
            self.address,
            self.fee_rate_sat_vb,
            self.replaceable,
            self.coin_control.into(),
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSweepPsbtRequest {
    #[serde(flatten)]
    pub base: SweepRequestBase,
}

impl CreateSweepPsbtRequest {
    pub fn into_parts(self) -> (String, String, u64, bool, WalletCoinControlDto) {
        self.base.into_parts()
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationRequest {
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

impl From<ConsolidationRequest> for WalletConsolidationDto {
    fn from(request: ConsolidationRequest) -> Self {
        Self {
            include_outpoints: request.include_outpoints,
            exclude_outpoints: request.exclude_outpoints,
            confirmed_only: request.confirmed_only,
            max_input_count: request.max_input_count,
            min_input_count: request.min_input_count,
            min_utxo_value_sat: request.min_utxo_value_sat,
            max_utxo_value_sat: request.max_utxo_value_sat,
            max_fee_pct_of_input_value: request.max_fee_pct_of_input_value,
            strategy: request.strategy,
            selection_mode: request.selection_mode,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConsolidationPsbtRequest {
    pub wallet_name: String,
    pub fee_rate_sat_vb: u64,
    pub replaceable: bool,
    pub consolidation: ConsolidationRequest,
}

impl CreateConsolidationPsbtRequest {
    pub fn into_parts(self) -> (String, u64, bool, WalletConsolidationDto) {
        (
            self.wallet_name,
            self.fee_rate_sat_vb,
            self.replaceable,
            self.consolidation.into(),
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignPsbtRequest {
    pub wallet_name: String,
    pub psbt_base64: String,
}

impl SignPsbtRequest {
    pub fn into_parts(self) -> (String, String) {
        (self.wallet_name, self.psbt_base64)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishPsbtRequest {
    pub wallet_name: String,
    pub psbt_base64: String,
}

impl PublishPsbtRequest {
    pub fn into_parts(self) -> (String, String) {
        (self.wallet_name, self.psbt_base64)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPsbtRequest {
    pub wallet_name: String,
    pub address: String,
    pub amount_sat: u64,
    pub fee_rate_sat_vb: u64,
    pub replaceable: bool,
    pub confirmed_only: bool,
}

impl SendPsbtRequest {
    pub fn into_parts(self) -> (String, String, u64, u64, bool, bool) {
        (
            self.wallet_name,
            self.address,
            self.amount_sat,
            self.fee_rate_sat_vb,
            self.replaceable,
            self.confirmed_only,
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPsbtWithCoinControlRequest {
    pub wallet_name: String,
    pub address: String,
    pub amount_sat: u64,
    pub fee_rate_sat_vb: u64,
    pub replaceable: bool,
    pub confirmed_only: bool,
    pub coin_control: CoinControlRequest,
}

impl SendPsbtWithCoinControlRequest {
    pub fn into_parts(self) -> (String, String, u64, u64, bool, bool, WalletCoinControlDto) {
        (
            self.wallet_name,
            self.address,
            self.amount_sat,
            self.fee_rate_sat_vb,
            self.replaceable,
            self.confirmed_only,
            self.coin_control.into(),
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMaxPsbtRequest {
    #[serde(flatten)]
    pub base: SendMaxRequestBase,
}

impl SendMaxPsbtRequest {
    pub fn into_parts(self) -> (String, String, u64, bool) {
        self.base.into_parts()
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMaxPsbtWithCoinControlRequest {
    pub wallet_name: String,
    pub address: String,
    pub fee_rate_sat_vb: u64,
    pub replaceable: bool,
    pub coin_control: CoinControlRequest,
}

impl SendMaxPsbtWithCoinControlRequest {
    pub fn into_parts(self) -> (String, String, u64, bool, WalletCoinControlDto) {
        (
            self.wallet_name,
            self.address,
            self.fee_rate_sat_vb,
            self.replaceable,
            self.coin_control.into(),
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SweepPsbtRequest {
    #[serde(flatten)]
    pub base: SweepRequestBase,
}

impl SweepPsbtRequest {
    pub fn into_parts(self) -> (String, String, u64, bool, WalletCoinControlDto) {
        self.base.into_parts()
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidatePsbtRequest {
    pub wallet_name: String,
    pub fee_rate_sat_vb: u64,
    pub replaceable: bool,
    pub consolidation: ConsolidationRequest,
}

impl ConsolidatePsbtRequest {
    pub fn into_parts(self) -> (String, u64, bool, WalletConsolidationDto) {
        (
            self.wallet_name,
            self.fee_rate_sat_vb,
            self.replaceable,
            self.consolidation.into(),
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpFeePsbtRequest {
    pub wallet_name: String,
    pub txid: String,
    pub fee_rate_sat_vb: u64,
}

impl BumpFeePsbtRequest {
    pub fn into_parts(self) -> (String, String, u64) {
        (self.wallet_name, self.txid, self.fee_rate_sat_vb)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpFeeRequest {
    pub wallet_name: String,
    pub txid: String,
    pub fee_rate_sat_vb: u64,
}

impl BumpFeeRequest {
    pub fn into_parts(self) -> (String, String, u64) {
        (self.wallet_name, self.txid, self.fee_rate_sat_vb)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpfpPsbtRequest {
    pub wallet_name: String,
    pub parent_txid: String,
    pub selected_outpoint: String,
    pub fee_rate_sat_vb: u64,
}

impl CpfpPsbtRequest {
    pub fn into_parts(self) -> (String, String, String, u64) {
        (
            self.wallet_name,
            self.parent_txid,
            self.selected_outpoint,
            self.fee_rate_sat_vb,
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpfpRequest {
    pub wallet_name: String,
    pub parent_txid: String,
    pub selected_outpoint: String,
    pub fee_rate_sat_vb: u64,
}

impl CpfpRequest {
    pub fn into_parts(self) -> (String, String, String, u64) {
        (
            self.wallet_name,
            self.parent_txid,
            self.selected_outpoint,
            self.fee_rate_sat_vb,
        )
    }
}
