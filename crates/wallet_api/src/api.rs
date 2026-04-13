use std::sync::Arc;

use crate::factory::build_default_api;
use crate::service::{wallet, inspect, psbt, registry};
use crate::WalletApiResult;

use crate::model::{
    WalletCpfpPsbtDto,
    WalletDetailsDto,
    WalletPsbtDto,
    TxBroadcastResultDto,
    WalletSignedPsbtDto,
    WalletStatusDto,
    WalletSummaryDto,
    WalletTxDto,
    WalletUtxoDto,
};

use wallet_core::WalletCore;
use wallet_storage::WalletStorage;
use wallet_sync::WalletSyncService;

#[derive(Debug)]
pub struct WalletApi {
    core: Arc<WalletCore>,
    storage: WalletStorage,
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
        Self { core, storage, sync }
    }

    pub async fn status(&self, name: &str) -> WalletApiResult<WalletStatusDto> {
        wallet::status(&self.storage, name).await
    }

    pub async fn list_wallets(&self) -> WalletApiResult<Vec<WalletSummaryDto>> {
        registry::list_wallets(&self.storage).await
    }

    pub async fn get_wallet(&self, name: &str) -> WalletApiResult<WalletDetailsDto> {
        registry::get_wallet(&self.storage, name).await
    }

    pub async fn import_wallet(&self, file_path: &str) -> WalletApiResult<()> {
        registry::import_wallet(&self.storage, file_path).await
    }

    pub async fn delete_wallet(&self, name: &str) -> WalletApiResult<()> {
        registry::delete_wallet(&self.storage, name).await
    }

    pub async fn address(&self, name: &str) -> WalletApiResult<String> {
        wallet::address(&self.storage, name).await
    }

    pub async fn sync_wallet(&self, name: &str) -> WalletApiResult<()> {
        wallet::sync(&self.storage, name).await
    }

    pub async fn balance(&self, name: &str) -> WalletApiResult<u64> {
        wallet::balance(&self.storage, name).await
    }

    pub async fn txs(&self, name: &str) -> WalletApiResult<Vec<WalletTxDto>> {
        inspect::txs(&self.storage, name).await
    }

    pub async fn utxos(&self, name: &str) -> WalletApiResult<Vec<WalletUtxoDto>> {
        inspect::utxos(&self.storage, name).await
    }

    pub async fn create_psbt(
        &self,
        name: &str,
        to_address: &str,
        amount_sat: u64,
        fee_rate_sat_per_vb: u64,
    ) -> WalletApiResult<WalletPsbtDto> {
        psbt::create(
            &self.storage,
            name,
            to_address,
            amount_sat,
            fee_rate_sat_per_vb,
        )
        .await
    }

    pub async fn sign_psbt(
        &self,
        name: &str,
        psbt_base64: &str,
    ) -> WalletApiResult<WalletSignedPsbtDto> {
        psbt::sign(&self.storage, name, psbt_base64).await
    }

    pub async fn publish_psbt(
        &self,
        name: &str,
        psbt_base64: &str,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        psbt::publish(&self.storage, name, psbt_base64).await
    }

    pub async fn bump_fee_psbt(
        &self,
        name: &str,
        txid: &str,
        fee_rate_sat_per_vb: u64,
    ) -> WalletApiResult<WalletPsbtDto> {
        psbt::bump_fee_psbt(&self.storage, name, txid, fee_rate_sat_per_vb).await
    }

    pub async fn bump_fee(
        &self,
        name: &str,
        txid: &str,
        fee_rate_sat_per_vb: u64,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        psbt::bump_fee(&self.storage, name, txid, fee_rate_sat_per_vb).await
    }

    pub async fn cpfp_psbt(
        &self,
        name: &str,
        parent_txid: &str,
        selected_outpoint: &str,
        fee_rate_sat_per_vb: u64,
    ) -> WalletApiResult<WalletCpfpPsbtDto> {
        psbt::cpfp_psbt(
            &self.storage,
            name,
            parent_txid,
            selected_outpoint,
            fee_rate_sat_per_vb,
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
            name,
            parent_txid,
            selected_outpoint,
            fee_rate_sat_per_vb,
        )
        .await
    }

    pub async fn send_psbt(
        &self,
        name: &str,
        to_address: &str,
        amount_sat: u64,
        fee_rate_sat_per_vb: u64,
    ) -> WalletApiResult<TxBroadcastResultDto> {
        let created = self
            .create_psbt(name, to_address, amount_sat, fee_rate_sat_per_vb)
            .await?;

        let signed = self
            .sign_psbt(name, &created.psbt_base64)
            .await?;

        if !signed.finalized {
            return Err(crate::WalletApiError::SendNotFinalized);
        }

        self.publish_psbt(name, &signed.psbt_base64).await
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
}