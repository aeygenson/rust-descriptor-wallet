use std::sync::Arc;
use tracing::debug;

use crate::factory::build_default_api;
use crate::service::{inspect, psbt, registry, wallet};
use crate::WalletApiResult;

use crate::model::{
    BumpFeeRequestDto, ConsolidationRequestDto, CpfpRequestDto, CreatePsbtRequestDto,
    DeleteWalletRequestDto, GetWalletRequestDto, ImportWalletRequestDto, PublishPsbtRequestDto,
    SendMaxRequestDto, SignPsbtRequestDto, SweepRequestDto, TxBroadcastResultDto,
    WalletAddressRequestDto, WalletBackendHealthDto, WalletCoinControlDto,
    WalletConsolidationDto, WalletCpfpPsbtDto, WalletDetailsDto, WalletPsbtDto,
    WalletReceiveAddressDto, WalletSignedPsbtDto, WalletStatusDto, WalletSummaryDto,
    WalletTransactionsRequestDto, WalletTxDto, WalletUtxoDto, WalletUtxosRequestDto,
};

use wallet_core::WalletCore;
use wallet_storage::WalletStorage;
use wallet_sync::WalletSyncService;

#[derive(Debug)]
pub struct WalletApi {
    core: Arc<WalletCore>,
    pub  storage: WalletStorage,
    sync: WalletSyncService,
}

impl WalletApi {
    pub async fn new() -> WalletApiResult<Self> {
        build_default_api().await
    }

    pub fn from_parts(
        core: Arc<WalletCore>,
        storage: WalletStorage,
        sync: WalletSyncService,
    ) -> Self {
        Self {
            core,
            storage,
            sync,
        }
    }

    pub async fn status(&self, name: &str) -> WalletApiResult<WalletStatusDto> {
        wallet::status(&self.storage, name).await
    }

    pub async fn list_wallets(&self) -> WalletApiResult<Vec<WalletSummaryDto>> {
        registry::list_wallets(&self.storage).await
    }

    pub async fn get_wallet(&self, name: &str) -> WalletApiResult<WalletDetailsDto> {
        registry::get_wallet(
            &self.storage,
            GetWalletRequestDto {
                name: name.to_string(),
            },
        )
        .await
    }

    pub async fn import_wallet(&self, file_path: &str) -> WalletApiResult<()> {
        registry::import_wallet(
            &self.storage,
            ImportWalletRequestDto {
                file_path: file_path.to_string(),
            },
        )
        .await
    }

    pub async fn delete_wallet(&self, name: &str) -> WalletApiResult<()> {
        registry::delete_wallet(
            &self.storage,
            DeleteWalletRequestDto {
                name: name.to_string(),
            },
        )
        .await
    }

    pub async fn address(&self, name: &str) -> WalletApiResult<WalletReceiveAddressDto> {
        wallet::address(
            &self.storage,
            WalletAddressRequestDto {
                name: name.to_string(),
            },
        )
        .await
    }

    pub async fn sync(&self, name: &str) -> WalletApiResult<()> {
        wallet::sync(&self.storage, name).await
    }

    pub async fn backend_health(&self, name: &str) -> WalletApiResult<WalletBackendHealthDto> {
        wallet::backend_health(&self.storage, name).await
    }

    pub async fn balance(&self, name: &str) -> WalletApiResult<u64> {
        wallet::balance(&self.storage, name).await
    }

    /// Return wallet transaction history rows for CLI/API/UI.
    ///
    /// Each `WalletTxDto` includes input previous outpoints and wallet-owned
    /// spendable outputs, allowing the UI to inspect parent/child relationships
    /// and derive CPFP candidate outpoints without guessing.
    pub async fn txs(&self, name: &str) -> WalletApiResult<Vec<WalletTxDto>> {
        inspect::txs(
            &self.storage,
            WalletTransactionsRequestDto {
                name: name.to_string(),
            },
        )
        .await
    }

    pub async fn utxos(&self, name: &str) -> WalletApiResult<Vec<WalletUtxoDto>> {
        inspect::utxos(
            &self.storage,
            WalletUtxosRequestDto {
                name: name.to_string(),
            },
        )
        .await
    }

    pub async fn create_psbt(
        &self,
        name: &str,
        to_address: &str,
        amount_sat: u64,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        confirmed_only: bool,
    ) -> WalletApiResult<WalletPsbtDto> {
        debug!(
            "api: create_psbt name={} to={} amount_sat={} fee_rate_sat_per_vb={} replaceable={} confirmed_only={}",
            name,
            to_address,
            amount_sat,
            fee_rate_sat_per_vb,
            replaceable,
            confirmed_only,
        );

        let coin_control = if confirmed_only {
            Some(WalletCoinControlDto {
                confirmed_only: true,
                ..Default::default()
            })
        } else {
            None
        };

        psbt::create(
            &self.storage,
            CreatePsbtRequestDto {
                name: name.to_string(),
                to_address: to_address.to_string(),
                amount_sat,
                fee_rate_sat_per_vb,
                replaceable,
                coin_control,
            },
        )
        .await
    }

    pub async fn create_psbt_with_coin_control(
        &self,
        name: &str,
        to_address: &str,
        amount_sat: u64,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        coin_control: WalletCoinControlDto,
    ) -> WalletApiResult<WalletPsbtDto> {
        debug!(
            "api: create_psbt_with_coin_control name={} to={} amount_sat={} fee_rate_sat_per_vb={} replaceable={} confirmed_only={} selection_mode={:?}",
            name,
            to_address,
            amount_sat,
            fee_rate_sat_per_vb,
            replaceable,
            coin_control.confirmed_only,
            coin_control.selection_mode,
        );

        psbt::create(
            &self.storage,
            CreatePsbtRequestDto {
                name: name.to_string(),
                to_address: to_address.to_string(),
                amount_sat,
                fee_rate_sat_per_vb,
                replaceable,
                coin_control: Some(coin_control),
            },
        )
        .await
    }

    pub async fn create_send_max_psbt(
        &self,
        name: &str,
        to_address: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
    ) -> WalletApiResult<WalletPsbtDto> {
        debug!(
            "api: create_send_max_psbt name={} to={} fee_rate_sat_per_vb={} replaceable={}",
            name, to_address, fee_rate_sat_per_vb, replaceable,
        );

        psbt::create_send_max(
            &self.storage,
            SendMaxRequestDto {
                name: name.to_string(),
                to_address: to_address.to_string(),
                fee_rate_sat_per_vb,
                replaceable,
                coin_control: None,
            },
        )
        .await
    }

    pub async fn create_send_max_psbt_with_coin_control(
        &self,
        name: &str,
        to_address: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        coin_control: WalletCoinControlDto,
    ) -> WalletApiResult<WalletPsbtDto> {
        debug!(
            "api: create_send_max_psbt_with_coin_control name={} to={} fee_rate_sat_per_vb={} replaceable={} confirmed_only={} selection_mode={:?}",
            name,
            to_address,
            fee_rate_sat_per_vb,
            replaceable,
            coin_control.confirmed_only,
            coin_control.selection_mode,
        );

        psbt::create_send_max(
            &self.storage,
            SendMaxRequestDto {
                name: name.to_string(),
                to_address: to_address.to_string(),
                fee_rate_sat_per_vb,
                replaceable,
                coin_control: Some(coin_control),
            },
        )
        .await
    }

    pub async fn create_sweep_psbt(
        &self,
        name: &str,
        to_address: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        coin_control: WalletCoinControlDto,
    ) -> WalletApiResult<WalletPsbtDto> {
        debug!(
            "api: create_sweep_psbt name={} to={} fee_rate_sat_per_vb={} replaceable={} confirmed_only={} selection_mode={:?}",
            name,
            to_address,
            fee_rate_sat_per_vb,
            replaceable,
            coin_control.confirmed_only,
            coin_control.selection_mode,
        );

        psbt::create_sweep(
            &self.storage,
            SweepRequestDto {
                name: name.to_string(),
                to_address: to_address.to_string(),
                fee_rate_sat_per_vb,
                replaceable,
                coin_control,
            },
        )
        .await
    }

    /// Create a consolidation PSBT via the API boundary.
    ///
    /// This API layer remains DTO/string-based. Any outpoint strings inside
    /// `WalletConsolidationDto` are parsed into typed `WalletOutPoint` values
    /// in lower layers (`wallet_api::model` -> `wallet_core`).
    pub async fn create_consolidation_psbt(
        &self,
        name: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        consolidation: WalletConsolidationDto,
    ) -> WalletApiResult<WalletPsbtDto> {
        debug!(
            "api: create_consolidation_psbt name={} fee_rate_sat_per_vb={} replaceable={} confirmed_only={} selection_mode={:?}",
            name,
            fee_rate_sat_per_vb,
            replaceable,
            consolidation.confirmed_only,
            consolidation.selection_mode,
        );

        psbt::create_consolidation(
            &self.storage,
            ConsolidationRequestDto {
                name: name.to_string(),
                fee_rate_sat_per_vb,
                replaceable,
                consolidation,
            },
        )
        .await
    }

    pub async fn create_consolidation(
        &self,
        name: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        consolidation: WalletConsolidationDto,
    ) -> WalletApiResult<WalletPsbtDto> {
        self.create_consolidation_psbt(name, fee_rate_sat_per_vb, replaceable, consolidation)
            .await
    }

    pub async fn consolidate_and_broadcast(
        &self,
        name: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        consolidation: WalletConsolidationDto,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        psbt::consolidate(
            &self.storage,
            ConsolidationRequestDto {
                name: name.to_string(),
                fee_rate_sat_per_vb,
                replaceable,
                consolidation,
            },
        )
        .await
    }

    pub async fn consolidate(
        &self,
        name: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        consolidation: WalletConsolidationDto,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        self.consolidate_and_broadcast(name, fee_rate_sat_per_vb, replaceable, consolidation)
            .await
    }

    pub async fn sweep_and_broadcast(
        &self,
        name: &str,
        to_address: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        coin_control: WalletCoinControlDto,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        psbt::sweep(
            &self.storage,
            SweepRequestDto {
                name: name.to_string(),
                to_address: to_address.to_string(),
                fee_rate_sat_per_vb,
                replaceable,
                coin_control,
            },
        )
        .await
    }

    pub async fn send_consolidation_psbt(
        &self,
        name: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        consolidation: WalletConsolidationDto,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        self.consolidate_and_broadcast(name, fee_rate_sat_per_vb, replaceable, consolidation)
            .await
    }

    pub async fn send_psbt_with_coin_control(
        &self,
        name: &str,
        to_address: &str,
        amount_sat: u64,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        coin_control: WalletCoinControlDto,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        let created = self
            .create_psbt_with_coin_control(
                name,
                to_address,
                amount_sat,
                fee_rate_sat_per_vb,
                replaceable,
                coin_control,
            )
            .await?;

        self.sign_and_publish(name, &created.psbt_base64).await
    }

    pub async fn send_max_psbt(
        &self,
        name: &str,
        to_address: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        let created = self
            .create_send_max_psbt(name, to_address, fee_rate_sat_per_vb, replaceable)
            .await?;

        self.sign_and_publish(name, &created.psbt_base64).await
    }

    pub async fn send_max_psbt_with_coin_control(
        &self,
        name: &str,
        to_address: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        coin_control: WalletCoinControlDto,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        let created = self
            .create_send_max_psbt_with_coin_control(
                name,
                to_address,
                fee_rate_sat_per_vb,
                replaceable,
                coin_control,
            )
            .await?;

        self.sign_and_publish(name, &created.psbt_base64).await
    }

    pub async fn send_sweep_psbt(
        &self,
        name: &str,
        to_address: &str,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        coin_control: WalletCoinControlDto,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        self.sweep_and_broadcast(
            name,
            to_address,
            fee_rate_sat_per_vb,
            replaceable,
            coin_control,
        )
        .await
    }

    pub async fn sign_psbt(
        &self,
        name: &str,
        psbt_base64: &str,
    ) -> WalletApiResult<WalletSignedPsbtDto> {
        psbt::sign(
            &self.storage,
            SignPsbtRequestDto {
                name: name.to_string(),
                psbt_base64: psbt_base64.to_string(),
            },
        )
        .await
    }

    pub async fn publish_psbt(
        &self,
        name: &str,
        psbt_base64: &str,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        psbt::publish(
            &self.storage,
            PublishPsbtRequestDto {
                name: name.to_string(),
                psbt_base64: psbt_base64.to_string(),
            },
        )
        .await
    }

    pub async fn bump_fee_psbt(
        &self,
        name: &str,
        txid: &str,
        fee_rate_sat_per_vb: u64,
    ) -> WalletApiResult<WalletPsbtDto> {
        psbt::bump_fee_psbt(
            &self.storage,
            BumpFeeRequestDto {
                name: name.to_string(),
                txid: txid.to_string(),
                fee_rate_sat_per_vb,
            },
        )
        .await
    }

    pub async fn bump_fee(
        &self,
        name: &str,
        txid: &str,
        fee_rate_sat_per_vb: u64,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        psbt::bump_fee(
            &self.storage,
            BumpFeeRequestDto {
                name: name.to_string(),
                txid: txid.to_string(),
                fee_rate_sat_per_vb,
            },
        )
        .await
    }

    /// Create a CPFP PSBT via the API boundary.
    ///
    /// The selected outpoint remains a string at the API boundary and is
    /// converted into a typed `WalletOutPoint` inside the PSBT service layer.
    pub async fn cpfp_psbt(
        &self,
        name: &str,
        parent_txid: &str,
        selected_outpoint: &str,
        fee_rate_sat_per_vb: u64,
    ) -> WalletApiResult<WalletCpfpPsbtDto> {
        psbt::cpfp_psbt(
            &self.storage,
            CpfpRequestDto {
                name: name.to_string(),
                parent_txid: parent_txid.to_string(),
                selected_outpoint: selected_outpoint.to_string(),
                fee_rate_sat_per_vb,
            },
        )
        .await
    }

    pub async fn cpfp(
        &self,
        name: &str,
        parent_txid: &str,
        selected_outpoint: &str,
        fee_rate_sat_per_vb: u64,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        psbt::cpfp(
            &self.storage,
            CpfpRequestDto {
                name: name.to_string(),
                parent_txid: parent_txid.to_string(),
                selected_outpoint: selected_outpoint.to_string(),
                fee_rate_sat_per_vb,
            },
        )
        .await
    }

    pub async fn send_psbt(
        &self,
        name: &str,
        to_address: &str,
        amount_sat: u64,
        fee_rate_sat_per_vb: u64,
        replaceable: bool,
        confirmed_only: bool,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        let created = self
            .create_psbt(
                name,
                to_address,
                amount_sat,
                fee_rate_sat_per_vb,
                replaceable,
                confirmed_only,
            )
            .await?;

        self.sign_and_publish(name, &created.psbt_base64).await
    }

    pub fn core(&self) -> &Arc<WalletCore> {
        &self.core
    }

    pub fn storage(&self) -> &WalletStorage {
        &self.storage
    }

    pub fn sync_service(&self) -> &WalletSyncService {
        &self.sync
    }

    pub async fn sign_and_publish(
        &self,
        name: &str,
        psbt_base64: &str,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        let signed = self.sign_psbt(name, psbt_base64).await?;

        if !signed.finalized {
            return Err(crate::WalletApiError::SendNotFinalized);
        }

        self.publish_psbt(name, &signed.psbt_base64).await
    }
}
