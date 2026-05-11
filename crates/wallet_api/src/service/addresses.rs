

use qrcode::{render::svg, QrCode};
use tracing::{debug, info};

use crate::model::{
    AddressBookEntryDto, ClearReceiveAddressLabelRequestDto, CreateAddressBookEntryRequestDto,
    DeleteAddressBookEntryRequestDto, GetAddressBookEntryRequestDto, LabelReceiveAddressRequestDto,
    ListAddressBookEntriesRequestDto, WalletAddressRequestDto, WalletReceiveAddressHistoryDto,
    WalletReceiveAddressesRequestDto,
};
use crate::service::wallet::load_wallet_config;
use crate::{WalletApiError, WalletApiResult};

use wallet_core::WalletService;
use wallet_storage::WalletStorage;

fn bitcoin_uri(address: &str) -> String {
    format!("bitcoin:{address}")
}

fn qr_svg(payload: &str) -> WalletApiResult<String> {
    let code = QrCode::new(payload.as_bytes())
        .map_err(|err| WalletApiError::QrGeneration(err.to_string()))?;

    Ok(code
        .render::<svg::Color>()
        .min_dimensions(220, 220)
        .dark_color(svg::Color("#0f172a"))
        .light_color(svg::Color("#ffffff"))
        .build())
}

fn attach_qr_svg(
    mut dto: WalletReceiveAddressHistoryDto,
) -> WalletApiResult<WalletReceiveAddressHistoryDto> {
    dto.qr_svg = Some(qr_svg(&dto.bitcoin_uri)?);
    Ok(dto)
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

    let dto = attach_qr_svg(WalletReceiveAddressHistoryDto::from(record))?;

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
        .map(attach_qr_svg)
        .collect::<WalletApiResult<Vec<_>>>()?;

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
    let dto = attach_qr_svg(WalletReceiveAddressHistoryDto::from(record))?;

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
    let dto = attach_qr_svg(WalletReceiveAddressHistoryDto::from(record))?;

    info!(
        "api addresses: clear_receive_address_label success name={} address={}",
        name,
        dto.address
    );

    Ok(dto)
}

pub async fn create_address_book_entry(
    storage: &WalletStorage,
    request: CreateAddressBookEntryRequestDto,
) -> WalletApiResult<AddressBookEntryDto> {
    let CreateAddressBookEntryRequestDto {
        name,
        label,
        address,
        notes,
    } = request;

    debug!(
        "api addresses: create_address_book_entry start name={} address={}",
        name,
        address
    );

    let config = load_wallet_config(storage, &name).await?;
    let network = config.network.to_string();

    let record = storage
        .create_address_book_entry(
            &name,
            &network,
            &label,
            &address,
            notes.as_deref(),
        )
        .await?;
    let dto = AddressBookEntryDto::from(record);

    info!(
        "api addresses: create_address_book_entry success name={} label={} address={}",
        name,
        dto.label,
        dto.address
    );

    Ok(dto)
}

pub async fn list_address_book_entries(
    storage: &WalletStorage,
    request: ListAddressBookEntriesRequestDto,
) -> WalletApiResult<Vec<AddressBookEntryDto>> {
    let ListAddressBookEntriesRequestDto { name } = request;

    debug!(
        "api addresses: list_address_book_entries start name={}",
        name
    );

    let records = storage.list_address_book_entries(&name).await?;
    let dtos = records
        .into_iter()
        .map(AddressBookEntryDto::from)
        .collect::<Vec<_>>();

    info!(
        "api addresses: list_address_book_entries success name={} count={}",
        name,
        dtos.len()
    );

    Ok(dtos)
}

pub async fn get_address_book_entry(
    storage: &WalletStorage,
    request: GetAddressBookEntryRequestDto,
) -> WalletApiResult<Option<AddressBookEntryDto>> {
    let GetAddressBookEntryRequestDto { name, address } = request;

    debug!(
        "api addresses: get_address_book_entry start name={} address={}",
        name,
        address
    );

    let record = storage
        .get_address_book_entry_by_address(&name, &address)
        .await?;
    let dto = record.map(AddressBookEntryDto::from);

    info!(
        "api addresses: get_address_book_entry success name={} address={} found={}",
        name,
        address,
        dto.is_some()
    );

    Ok(dto)
}

pub async fn delete_address_book_entry(
    storage: &WalletStorage,
    request: DeleteAddressBookEntryRequestDto,
) -> WalletApiResult<bool> {
    let DeleteAddressBookEntryRequestDto { name, address } = request;

    debug!(
        "api addresses: delete_address_book_entry start name={} address={}",
        name,
        address
    );

    let deleted = storage
        .delete_address_book_entry(&name, &address)
        .await?;

    info!(
        "api addresses: delete_address_book_entry success name={} address={} deleted={}",
        name,
        address,
        deleted
    );

    Ok(deleted)
}