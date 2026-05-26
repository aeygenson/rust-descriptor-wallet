

use tracing::{debug, info};
use wallet_storage::WalletStorage;

use crate::model::{
    WalletLockedUtxoDto, WalletLockedUtxosDto, WalletLockedUtxosRequestDto,
    WalletLockUtxosRequestDto, WalletUnlockUtxosRequestDto,
};
use crate::WalletApiResult;

pub async fn lock_utxos(
    storage: &WalletStorage,
    request: WalletLockUtxosRequestDto,
) -> WalletApiResult<WalletLockedUtxosDto> {
    let WalletLockUtxosRequestDto {
        name,
        outpoints,
        reason,
    } = request;

    debug!(
        "api locked_utxos: lock start name={} outpoints={} has_reason={}",
        name,
        outpoints.len(),
        reason.is_some()
    );

    let mut locked_utxos = Vec::with_capacity(outpoints.len());

    for outpoint in outpoints {
        let record = storage
            .lock_utxo(&name, &outpoint, reason.as_deref())
            .await?;
        locked_utxos.push(record.into());
    }

    info!(
        "api locked_utxos: lock success name={} count={}",
        name,
        locked_utxos.len()
    );

    Ok(WalletLockedUtxosDto {
        wallet_name: name,
        locked_utxos,
    })
}

pub async fn unlock_utxos(
    storage: &WalletStorage,
    request: WalletUnlockUtxosRequestDto,
) -> WalletApiResult<WalletLockedUtxosDto> {
    let WalletUnlockUtxosRequestDto { name, outpoints } = request;

    debug!(
        "api locked_utxos: unlock start name={} outpoints={}",
        name,
        outpoints.len()
    );

    for outpoint in outpoints {
        storage.unlock_utxo(&name, &outpoint).await?;
    }

    let locked_utxos: Vec<WalletLockedUtxoDto> = storage
        .list_locked_utxos(&name)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    info!(
        "api locked_utxos: unlock success name={} remaining_count={}",
        name,
        locked_utxos.len()
    );

    Ok(WalletLockedUtxosDto {
        wallet_name: name,
        locked_utxos,
    })
}

pub async fn list_locked_utxos(
    storage: &WalletStorage,
    request: WalletLockedUtxosRequestDto,
) -> WalletApiResult<WalletLockedUtxosDto> {
    let WalletLockedUtxosRequestDto { name } = request;

    debug!("api locked_utxos: list start name={}", name);

    let locked_utxos: Vec<WalletLockedUtxoDto> = storage
        .list_locked_utxos(&name)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    info!(
        "api locked_utxos: list success name={} count={}",
        name,
        locked_utxos.len()
    );

    Ok(WalletLockedUtxosDto {
        wallet_name: name,
        locked_utxos,
    })
}