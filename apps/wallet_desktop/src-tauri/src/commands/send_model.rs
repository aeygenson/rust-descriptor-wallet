use wallet_api::model::{
    BumpFeeRequestDto, ConsolidationRequestDto, CpfpRequestDto,
    CreatePsbtRequestDto, PublishPsbtRequestDto, SendMaxRequestDto,
    SignPsbtRequestDto, SweepRequestDto, WalletCoinControlDto,
    WalletConsolidationDto, WalletConsolidationStrategyDto,
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


impl From<CreatePsbtRequest> for CreatePsbtRequestDto {
    fn from(request: CreatePsbtRequest) -> Self {
        Self {
            name: request.wallet_name,
            to_address: request.address,
            amount_sat: request.amount_sat,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
            replaceable: request.replaceable,
            coin_control: if request.confirmed_only {
                Some(WalletCoinControlDto {
                    confirmed_only: true,
                    ..Default::default()
                })
            } else {
                None
            },
        }
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


impl From<CreatePsbtWithCoinControlRequest> for CreatePsbtRequestDto {
    fn from(request: CreatePsbtWithCoinControlRequest) -> Self {
        Self {
            name: request.wallet_name,
            to_address: request.address,
            amount_sat: request.amount_sat,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
            replaceable: request.replaceable,
            coin_control: Some(request.coin_control.into()),
        }
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


impl From<CreateSendMaxPsbtRequest> for SendMaxRequestDto {
    fn from(request: CreateSendMaxPsbtRequest) -> Self {
        Self {
            name: request.base.wallet_name,
            to_address: request.base.address,
            fee_rate_sat_per_vb: request.base.fee_rate_sat_vb,
            replaceable: request.base.replaceable,
            coin_control: None,
        }
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


impl From<CreateSendMaxPsbtWithCoinControlRequest> for SendMaxRequestDto {
    fn from(request: CreateSendMaxPsbtWithCoinControlRequest) -> Self {
        Self {
            name: request.wallet_name,
            to_address: request.address,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
            replaceable: request.replaceable,
            coin_control: Some(request.coin_control.into()),
        }
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


impl From<CreateSweepPsbtRequest> for SweepRequestDto {
    fn from(request: CreateSweepPsbtRequest) -> Self {
        Self {
            name: request.base.wallet_name,
            to_address: request.base.address,
            fee_rate_sat_per_vb: request.base.fee_rate_sat_vb,
            replaceable: request.base.replaceable,
            coin_control: request.base.coin_control.into(),
        }
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


impl From<CreateConsolidationPsbtRequest> for ConsolidationRequestDto {
    fn from(request: CreateConsolidationPsbtRequest) -> Self {
        Self {
            name: request.wallet_name,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
            replaceable: request.replaceable,
            consolidation: request.consolidation.into(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignPsbtRequest {
    pub wallet_name: String,
    pub psbt_base64: String,
}


impl From<SignPsbtRequest> for SignPsbtRequestDto {
    fn from(request: SignPsbtRequest) -> Self {
        Self {
            name: request.wallet_name,
            psbt_base64: request.psbt_base64,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishPsbtRequest {
    pub wallet_name: String,
    pub psbt_base64: String,
}


impl From<PublishPsbtRequest> for PublishPsbtRequestDto {
    fn from(request: PublishPsbtRequest) -> Self {
        Self {
            name: request.wallet_name,
            psbt_base64: request.psbt_base64,
        }
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

impl From<SendPsbtRequest> for CreatePsbtRequestDto {
    fn from(request: SendPsbtRequest) -> Self {
        Self {
            name: request.wallet_name,
            to_address: request.address,
            amount_sat: request.amount_sat,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
            replaceable: request.replaceable,
            coin_control: if request.confirmed_only {
                Some(WalletCoinControlDto {
                    confirmed_only: true,
                    ..Default::default()
                })
            } else {
                None
            },
        }
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

impl From<SendPsbtWithCoinControlRequest> for CreatePsbtRequestDto {
    fn from(request: SendPsbtWithCoinControlRequest) -> Self {
        let mut coin_control: WalletCoinControlDto = request.coin_control.into();
        coin_control.confirmed_only = request.confirmed_only;

        Self {
            name: request.wallet_name,
            to_address: request.address,
            amount_sat: request.amount_sat,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
            replaceable: request.replaceable,
            coin_control: Some(coin_control),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMaxPsbtRequest {
    #[serde(flatten)]
    pub base: SendMaxRequestBase,
}

impl From<SendMaxPsbtRequest> for SendMaxRequestDto {
    fn from(request: SendMaxPsbtRequest) -> Self {
        Self {
            name: request.base.wallet_name,
            to_address: request.base.address,
            fee_rate_sat_per_vb: request.base.fee_rate_sat_vb,
            replaceable: request.base.replaceable,
            coin_control: None,
        }
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

impl From<SendMaxPsbtWithCoinControlRequest> for SendMaxRequestDto {
    fn from(request: SendMaxPsbtWithCoinControlRequest) -> Self {
        Self {
            name: request.wallet_name,
            to_address: request.address,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
            replaceable: request.replaceable,
            coin_control: Some(request.coin_control.into()),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SweepPsbtRequest {
    #[serde(flatten)]
    pub base: SweepRequestBase,
}

impl From<SweepPsbtRequest> for SweepRequestDto {
    fn from(request: SweepPsbtRequest) -> Self {
        Self {
            name: request.base.wallet_name,
            to_address: request.base.address,
            fee_rate_sat_per_vb: request.base.fee_rate_sat_vb,
            replaceable: request.base.replaceable,
            coin_control: request.base.coin_control.into(),
        }
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

impl From<ConsolidatePsbtRequest> for ConsolidationRequestDto {
    fn from(request: ConsolidatePsbtRequest) -> Self {
        Self {
            name: request.wallet_name,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
            replaceable: request.replaceable,
            consolidation: request.consolidation.into(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpFeePsbtRequest {
    pub wallet_name: String,
    pub txid: String,
    pub fee_rate_sat_vb: u64,
}

impl From<BumpFeePsbtRequest> for BumpFeeRequestDto {
    fn from(request: BumpFeePsbtRequest) -> Self {
        Self {
            name: request.wallet_name,
            txid: request.txid,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpFeeRequest {
    pub wallet_name: String,
    pub txid: String,
    pub fee_rate_sat_vb: u64,
}

impl From<BumpFeeRequest> for BumpFeeRequestDto {
    fn from(request: BumpFeeRequest) -> Self {
        Self {
            name: request.wallet_name,
            txid: request.txid,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
        }
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

impl From<CpfpPsbtRequest> for CpfpRequestDto {
    fn from(request: CpfpPsbtRequest) -> Self {
        Self {
            name: request.wallet_name,
            parent_txid: request.parent_txid,
            selected_outpoint: request.selected_outpoint,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
        }
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

impl From<CpfpRequest> for CpfpRequestDto {
    fn from(request: CpfpRequest) -> Self {
        Self {
            name: request.wallet_name,
            parent_txid: request.parent_txid,
            selected_outpoint: request.selected_outpoint,
            fee_rate_sat_per_vb: request.fee_rate_sat_vb,
        }
    }
}
