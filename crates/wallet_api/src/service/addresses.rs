

use tracing::{debug, info};

use crate::model::{
    ClearReceiveAddressLabelRequestDto, LabelReceiveAddressRequestDto, WalletAddressRequestDto,
    WalletReceiveAddressHistoryDto, WalletReceiveAddressesRequestDto,
};
use crate::service::wallet::load_wallet_config;
use crate::WalletApiResult;

use wallet_core::WalletService;
use wallet_storage::WalletStorage;

fn bitcoin_uri(address: &str) -> String {
    format!("bitcoin:{address}")
}

pub async fn address(
    storage: &WalletStorage,
    request: WalletAddressRequestDto,
) -> WalletApiResult<WalletReceiveAddressHistoryDto> {
    let WalletAddressRequestDto { name } = request;

    debug!("api addresses: address start name={}", name);

    let config = load_wallet_config(storage, &name).await?;
    let mut wallet = WalletService::load_or_create(&config)?;

    let address = wallet.next_receive_address()?;

    let address_string = address.address.to_string();
    let keychain = address.keychain.as_str().to_string();
    let address_index = address.index.map(|i| i.as_u32() as i64);
    let bitcoin_uri = bitcoin_uri(&address_string);

    let record = storage
        .record_receive_address(
            &name,
            &address_string,
            &keychain,
            address_index,
            &bitcoin_uri,
        )
        .await?;

    let dto = WalletReceiveAddressHistoryDto::from(record);

    info!(
        "api addresses: address success name={} keychain={} index={:?}",
        name,
        dto.keychain,
        dto.index
    );

    Ok(dto)
}

pub async fn list_receive_addresses(
    storage: &WalletStorage,
    request: WalletReceiveAddressesRequestDto,
) -> WalletApiResult<Vec<WalletReceiveAddressHistoryDto>> {
    let WalletReceiveAddressesRequestDto { name } = request;

    debug!("api addresses: list_receive_addresses start name={}", name);

    let records = storage.list_receive_addresses(&name).await?;
    let dtos = records
        .into_iter()
        .map(WalletReceiveAddressHistoryDto::from)
        .collect::<Vec<_>>();

    info!(
        "api addresses: list_receive_addresses success name={} count={}",
        name,
        dtos.len()
    );

    Ok(dtos)
}

pub async fn label_receive_address(
    storage: &WalletStorage,
    request: LabelReceiveAddressRequestDto,
) -> WalletApiResult<WalletReceiveAddressHistoryDto> {
    let LabelReceiveAddressRequestDto {
        name,
        address,
        label,
    } = request;

    debug!(
        "api addresses: label_receive_address start name={} address={}",
        name,
        address
    );

    let record = storage
        .label_receive_address(&name, &address, &label)
        .await?;
    let dto = WalletReceiveAddressHistoryDto::from(record);

    info!(
        "api addresses: label_receive_address success name={} address={}",
        name,
        dto.address
    );

    Ok(dto)
}

pub async fn clear_receive_address_label(
    storage: &WalletStorage,
    request: ClearReceiveAddressLabelRequestDto,
) -> WalletApiResult<WalletReceiveAddressHistoryDto> {
    let ClearReceiveAddressLabelRequestDto { name, address } = request;

    debug!(
        "api addresses: clear_receive_address_label start name={} address={}",
        name,
        address
    );

    let record = storage
        .clear_receive_address_label(&name, &address)
        .await?;
    let dto = WalletReceiveAddressHistoryDto::from(record);

    info!(
        "api addresses: clear_receive_address_label success name={} address={}",
        name,
        dto.address
    );

    Ok(dto)
}