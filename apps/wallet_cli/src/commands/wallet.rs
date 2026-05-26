use anyhow::Result;
use std::path::Path;
use wallet_api::model::{
    AddressBookEntryDto, WalletBackendHealthDto, WalletDetailsDto, WalletLockedUtxoDto,
    WalletReceiveAddressHistoryDto, WalletStatusDto, WalletSummaryDto,
};
use wallet_api::WalletApi;

pub async fn list_wallets(api: &WalletApi) -> Result<()> {
    let wallets: Vec<WalletSummaryDto> = api.list_wallets().await?;

    if wallets.is_empty() {
        println!("No wallets found.");
    } else {
        for w in wallets {
            println!(
                "name={} network={} watch_only={}",
                w.name, w.network, w.is_watch_only
            );
        }
    }

    Ok(())
}

pub async fn backend_health(api: &WalletApi, name: &str) -> Result<()> {
    let health: WalletBackendHealthDto = api.backend_health(name).await?;

    println!(
        "sync_backend={}",
        if health.sync_backend_reachable {
            "ok"
        } else {
            "error"
        }
    );
    println!(
        "bitcoin_tip={} height={}",
        if health.bitcoin_tip_reachable {
            "ok"
        } else {
            "error"
        },
        health
            .tip_height
            .map(|height| height.to_string())
            .unwrap_or_else(|| "n/a".to_string())
    );
    println!(
        "broadcast_backend={}",
        if health.broadcast_backend_reachable {
            "ok"
        } else {
            "error"
        }
    );

    if let Some(message) = health.message {
        println!("message={message}");
    }

    Ok(())
}

fn print_receive_address(addr: &WalletReceiveAddressHistoryDto, show_qr_svg: bool) {
    println!("address={}", addr.address);
    println!("keychain={}", addr.keychain);
    println!(
        "index={}",
        addr.index
            .map(|index| index.to_string())
            .unwrap_or_else(|| "n/a".to_string())
    );
    println!("bitcoin_uri={}", addr.bitcoin_uri);

    if let Some(label) = &addr.label {
        println!("label={label}");
    }

    println!("created_at={}", addr.created_at);

    if let Some(updated_at) = &addr.updated_at {
        println!("updated_at={updated_at}");
    }

    if let Some(qr_svg) = &addr.qr_svg {
        println!("qr_svg_length={}", qr_svg.len());

        if show_qr_svg {
            println!("qr_svg={qr_svg}");
        }
    }
}

pub async fn address(api: &WalletApi, name: &str, show_qr_svg: bool) -> Result<()> {
    let addr: WalletReceiveAddressHistoryDto = api.address(name).await?;

    print_receive_address(&addr, show_qr_svg);

    Ok(())
}

pub async fn list_receive_addresses(api: &WalletApi, name: &str, show_qr_svg: bool) -> Result<()> {
    let addresses: Vec<WalletReceiveAddressHistoryDto> = api.list_receive_addresses(name).await?;

    if addresses.is_empty() {
        println!("No receive addresses found for wallet {name}.");
    } else {
        for addr in addresses {
            print_receive_address(&addr, show_qr_svg);
            println!("---");
        }
    }

    Ok(())
}

pub async fn label_receive_address(
    api: &WalletApi,
    name: &str,
    address: &str,
    label: &str,
) -> Result<()> {
    let addr: WalletReceiveAddressHistoryDto = api
        .label_receive_address(name, address, label)
        .await?;

    print_receive_address(&addr, false);

    Ok(())
}


pub async fn clear_receive_address_label(
    api: &WalletApi,
    name: &str,
    address: &str,
) -> Result<()> {
    let addr: WalletReceiveAddressHistoryDto = api
        .clear_receive_address_label(name, address)
        .await?;

    print_receive_address(&addr, false);

    Ok(())
}

fn print_address_book_entry(entry: &AddressBookEntryDto) {
    println!("wallet={}", entry.wallet_name);
    println!("network={}", entry.network);
    println!("label={}", entry.label);
    println!("address={}", entry.address);

    if let Some(notes) = &entry.notes {
        println!("notes={notes}");
    }

    println!("created_at={}", entry.created_at);

    if let Some(updated_at) = &entry.updated_at {
        println!("updated_at={updated_at}");
    }
}

pub async fn create_address_book_entry(
    api: &WalletApi,
    name: &str,
    label: &str,
    address: &str,
    notes: Option<String>,
) -> Result<()> {
    let entry: AddressBookEntryDto = api
        .create_address_book_entry(name, label, address, notes)
        .await?;

    print_address_book_entry(&entry);

    Ok(())
}

pub async fn list_address_book_entries(api: &WalletApi, name: &str) -> Result<()> {
    let entries: Vec<AddressBookEntryDto> = api.list_address_book_entries(name).await?;

    if entries.is_empty() {
        println!("No address book entries found for wallet {name}.");
    } else {
        for entry in entries {
            print_address_book_entry(&entry);
            println!("---");
        }
    }

    Ok(())
}

pub async fn get_address_book_entry(api: &WalletApi, name: &str, address: &str) -> Result<()> {
    match api.get_address_book_entry(name, address).await? {
        Some(entry) => print_address_book_entry(&entry),
        None => println!("No address book entry found for wallet {name} and address {address}."),
    }

    Ok(())
}

pub async fn delete_address_book_entry(api: &WalletApi, name: &str, address: &str) -> Result<()> {
    let deleted = api.delete_address_book_entry(name, address).await?;

    if deleted {
        println!("Deleted address book entry for wallet {name} address {address}.");
    } else {
        println!("No address book entry found for wallet {name} address {address}.");
    }

    Ok(())
}

fn print_locked_utxo(entry: &WalletLockedUtxoDto) {
    println!("wallet={}", entry.wallet_name);
    println!("outpoint={}", entry.outpoint);

    if let Some(reason) = &entry.reason {
        println!("reason={reason}");
    }

    println!("locked_at={}", entry.locked_at);

    if let Some(updated_at) = &entry.updated_at {
        println!("updated_at={updated_at}");
    }
}

pub async fn lock_utxo(
    api: &WalletApi,
    name: &str,
    outpoint: &str,
    reason: Option<String>,
) -> Result<()> {
    let locked = api.lock_utxo(name, outpoint, reason).await?;

    if let Some(entry) = locked.locked_utxos.iter().find(|entry| entry.outpoint == outpoint) {
        print_locked_utxo(entry);
    } else {
        println!("Locked UTXO {outpoint} for wallet {name}.");
    }

    Ok(())
}

pub async fn unlock_utxo(api: &WalletApi, name: &str, outpoint: &str) -> Result<()> {
    api.unlock_utxo(name, outpoint).await?;
    println!("Unlocked UTXO {outpoint} for wallet {name}.");

    Ok(())
}

pub async fn list_locked_utxos(api: &WalletApi, name: &str) -> Result<()> {
    let locked = api.locked_utxos(name).await?;

    if locked.is_empty() {
        println!("No locked UTXOs found for wallet {name}.");
    } else {
        for entry in locked {
            print_locked_utxo(&entry);
            println!("---");
        }
    }

    Ok(())
}

pub async fn sync_wallet(api: &WalletApi, name: &str) -> Result<()> {
    api.sync(name).await?;
    let status: WalletStatusDto = api.status(name).await?;

    println!("Synced wallet {name}");
    print_wallet_status(&status);

    Ok(())
}

pub async fn status(api: &WalletApi, name: &str) -> Result<()> {
    let status: WalletStatusDto = api.status(name).await?;

    println!("wallet={name}");
    print_wallet_status(&status);

    Ok(())
}

pub async fn balance(api: &WalletApi, name: &str) -> Result<()> {
    let status: WalletStatusDto = api.status(name).await?;

    println!("balance={} sats", status.balance);
    println!("utxos={}", status.utxo_count);

    Ok(())
}

fn print_wallet_status(status: &WalletStatusDto) {
    println!("balance={} sats", status.balance);
    println!("utxos={}", status.utxo_count);
    println!(
        "last_block_height={}",
        status
            .last_block_height
            .map(|height| height.to_string())
            .unwrap_or_else(|| "n/a".to_string())
    );
}

pub async fn get_wallet(api: &WalletApi, name: &str) -> Result<()> {
    let wallet: WalletDetailsDto = api.get_wallet(name).await?;

    println!("name={}", wallet.name);
    println!("network={}", wallet.network);
    println!("watch_only={}", wallet.is_watch_only);
    // descriptors
    println!("external_descriptor={}", wallet.descriptors.external);
    println!("internal_descriptor={}", wallet.descriptors.internal);

    // backend
    match &wallet.backend.sync {
        wallet_api::model::SyncBackendDto::Esplora { url } => {
            println!("sync_backend=esplora url={}", url);
        }
        wallet_api::model::SyncBackendDto::Electrum { url } => {
            println!("sync_backend=electrum url={}", url);
        }
    }

    match &wallet.backend.broadcast {
        Some(wallet_api::model::BroadcastBackendDto::Esplora { url }) => {
            println!("broadcast_backend=esplora url={}", url);
        }
        Some(wallet_api::model::BroadcastBackendDto::Rpc { url, .. }) => {
            println!("broadcast_backend=core_rpc url={}", url);
        }
        None => {
            println!("broadcast_backend=none");
        }
    }

    Ok(())
}

pub async fn import_wallet(api: &WalletApi, file: &Path) -> Result<()> {
    api.import_wallet(file.to_string_lossy().as_ref()).await?;
    println!("Imported wallet from {}", file.display());
    Ok(())
}

pub async fn delete_wallet(api: &WalletApi, name: &str) -> Result<()> {
    api.delete_wallet(name).await?;
    println!("Deleted wallet {name}");
    Ok(())
}
