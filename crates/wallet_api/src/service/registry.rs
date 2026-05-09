use crate::WalletApiResult;
use tracing::{debug, info};
use wallet_storage::WalletStorage;

use crate::model::{
    DeleteWalletRequestDto, GetWalletRequestDto, ImportWalletRequestDto,
    WalletDetailsDto, WalletSummaryDto,
};

/// List all wallets
pub async fn list_wallets(storage: &WalletStorage) -> WalletApiResult<Vec<WalletSummaryDto>> {
    debug!("api registry: list_wallets start");

    let wallets = storage.list_wallets().await?;

    let summaries: Vec<_> = wallets
        .into_iter()
        .map(|w| WalletSummaryDto {
            name: w.name,
            network: w.network,
            is_watch_only: w.is_watch_only,
        })
        .collect();

    info!(
        "api registry: list_wallets success total={} watch_only={}",
        summaries.len(),
        summaries.iter().filter(|w| w.is_watch_only).count()
    );

    Ok(summaries)
}

/// Import wallet from JSON file
pub async fn import_wallet(
    storage: &WalletStorage,
    request: ImportWalletRequestDto,
) -> WalletApiResult<()> {
    let ImportWalletRequestDto { file_path } = request;

    debug!("api registry: import_wallet start path={}", file_path);

    storage.import_wallet_from_file(&file_path).await?;

    info!("api registry: import_wallet success path={}", file_path);

    Ok(())
}

/// Delete wallet by name
pub async fn delete_wallet(
    storage: &WalletStorage,
    request: DeleteWalletRequestDto,
) -> WalletApiResult<()> {
    let DeleteWalletRequestDto { name } = request;

    debug!("api registry: delete_wallet start name={}", name);

    storage.delete_wallet(&name).await?;

    info!("api registry: delete_wallet success name={}", name);

    Ok(())
}

/// Get wallet details
pub async fn get_wallet(
    storage: &WalletStorage,
    request: GetWalletRequestDto,
) -> WalletApiResult<WalletDetailsDto> {
    let GetWalletRequestDto { name } = request;

    debug!("api registry: get_wallet start name={}", name);

    let wallet = storage.get_wallet_by_name(&name).await?;

    let sync_backend = wallet.parse_sync_backend().map_err(|e| {
        crate::WalletApiError::InvalidInput(format!(
            "invalid sync backend for wallet '{}': {}",
            name, e
        ))
    })?;

    let broadcast_backend = wallet.parse_broadcast_backend().map_err(|e| {
        crate::WalletApiError::InvalidInput(format!(
            "invalid broadcast backend for wallet '{}': {}",
            name, e
        ))
    })?;

    let dto = WalletDetailsDto {
        name: wallet.name,
        network: wallet.network,
        descriptors: crate::model::WalletDescriptorsDto {
            external: wallet.external_descriptor,
            internal: wallet.internal_descriptor,
        },
        backend: crate::model::WalletBackendDto {
            sync: sync_backend.into(),
            broadcast: broadcast_backend.map(Into::into),
        },
        is_watch_only: wallet.is_watch_only,
    };

    info!(
        "api registry: get_wallet success name={} watch_only={} has_internal_descriptor={} has_broadcast_backend={}",
        dto.name,
        dto.is_watch_only,
        !dto.descriptors.internal.is_empty(),
        dto.backend.broadcast.is_some()
    );

    Ok(dto)
}
