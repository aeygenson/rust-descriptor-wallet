
mod common;

use common::*;
use serial_test::file_serial;
use wallet_api::factory::build_default_api;

#[tokio::test]
#[file_serial]
async fn address_persists_receive_history() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-address")
        .await
        .expect("clone wallet for address test");
    let wallet_name = wallet_name.as_str();

    let before = api.list_receive_addresses(&wallet_name).await?;
    let generated = api.address(&wallet_name).await?;
    let after = api.list_receive_addresses(&wallet_name).await?;

    assert!(generated.address.starts_with("bcrt1"));
    assert_eq!(generated.bitcoin_uri, format!("bitcoin:{}", generated.address));
    assert!(after.len() >= before.len());
    assert!(after.iter().any(|entry| entry.address == generated.address));

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn list_receive_addresses_returns_generated_history() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-address")
        .await
        .expect("clone wallet for address test");
    let wallet_name = wallet_name.as_str();

    let generated = api.address(&wallet_name).await?;
    let history = api.list_receive_addresses(&wallet_name).await?;

    let found = history
        .iter()
        .find(|entry| entry.address == generated.address)
        .expect("generated address should be present in receive history");

    assert_eq!(found.keychain, generated.keychain);
    assert_eq!(found.index, generated.index);
    assert_eq!(found.bitcoin_uri, generated.bitcoin_uri);

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn label_receive_address_updates_label() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-address")
        .await
        .expect("clone wallet for address test");
    let wallet_name = wallet_name.as_str();

    let generated = api.address(&wallet_name).await?;
    let labeled = api
        .label_receive_address(&wallet_name, &generated.address, "Integration label")
        .await?;

    assert_eq!(labeled.address, generated.address);
    assert_eq!(labeled.label.as_deref(), Some("Integration label"));
    assert!(labeled.updated_at.is_some());

    let history = api.list_receive_addresses(&wallet_name).await?;
    let found = history
        .iter()
        .find(|entry| entry.address == generated.address)
        .expect("labeled address should be present in receive history");

    assert_eq!(found.label.as_deref(), Some("Integration label"));

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn clear_receive_address_label_removes_label() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-address")
        .await
        .expect("clone wallet for address test");
    let wallet_name = wallet_name.as_str();

    let generated = api.address(&wallet_name).await?;
    api.label_receive_address(&wallet_name, &generated.address, "Temporary label")
        .await?;

    let cleared = api
        .clear_receive_address_label(&wallet_name, &generated.address)
        .await?;

    assert_eq!(cleared.address, generated.address);
    assert_eq!(cleared.label, None);
    assert!(cleared.updated_at.is_some());

    let history = api.list_receive_addresses(&wallet_name).await?;
    let found = history
        .iter()
        .find(|entry| entry.address == generated.address)
        .expect("cleared address should be present in receive history");

    assert_eq!(found.label, None);

    Ok(())
}


#[tokio::test]
#[file_serial]
async fn label_receive_address_missing_address_returns_error() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-address")
        .await
        .expect("clone wallet for address test");
    let wallet_name = wallet_name.as_str();

    let result = api
        .label_receive_address(
            &wallet_name,
            "bcrt1qmissingreceiveaddress0000000000000000000000",
            "Missing label",
        )
        .await;

    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn address_book_create_list_get_and_delete_entry() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-address")
        .await
        .expect("clone wallet for address test");
    let wallet_name = wallet_name.as_str();

    let generated = api.address(&wallet_name).await?;
    let label = format!("Integration contact {}", address_suffix(&generated.address));
    let notes = Some("Created by wallet_api address integration test".to_string());

    let created = api
        .create_address_book_entry(
            &wallet_name,
            &label,
            &generated.address,
            notes.clone(),
        )
        .await?;

    assert_eq!(created.wallet_name, wallet_name);
    assert_eq!(created.label, label);
    assert_eq!(created.address, generated.address);
    assert_eq!(created.notes, notes);
    assert_eq!(created.network, "regtest");

    let listed = api.list_address_book_entries(&wallet_name).await?;
    assert!(listed.iter().any(|entry| entry.address == generated.address));

    let fetched = api
        .get_address_book_entry(&wallet_name, &generated.address)
        .await?
        .expect("created address book entry should be fetchable by address");

    assert_eq!(fetched.label, created.label);
    assert_eq!(fetched.address, created.address);

    let deleted = api
        .delete_address_book_entry(&wallet_name, &generated.address)
        .await?;

    assert!(deleted);

    let after_delete = api
        .get_address_book_entry(&wallet_name, &generated.address)
        .await?;

    assert!(after_delete.is_none());

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn address_book_duplicate_address_returns_error() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-address")
        .await
        .expect("clone wallet for address test");
    let wallet_name = wallet_name.as_str();

    let generated = api.address(&wallet_name).await?;
    let label = format!("Duplicate address {}", address_suffix(&generated.address));

    api.create_address_book_entry(
        &wallet_name,
        &label,
        &generated.address,
        None,
    )
    .await?;

    let duplicate = api
        .create_address_book_entry(
            &wallet_name,
            &format!("{label} copy"),
            &generated.address,
            None,
        )
        .await;

    assert!(duplicate.is_err());

    let _ = api
        .delete_address_book_entry(&wallet_name, &generated.address)
        .await;

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn address_book_duplicate_label_returns_error() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-address")
        .await
        .expect("clone wallet for address test");
    let wallet_name = wallet_name.as_str();

    let first = api.address(&wallet_name).await?;
    let second = api.address(&wallet_name).await?;
    let label = format!("Duplicate label {}", address_suffix(&first.address));

    api.create_address_book_entry(&wallet_name, &label, &first.address, None)
        .await?;

    let duplicate = api
        .create_address_book_entry(&wallet_name, &label, &second.address, None)
        .await;

    assert!(duplicate.is_err());

    let _ = api.delete_address_book_entry(&wallet_name, &first.address).await;
    let _ = api.delete_address_book_entry(&wallet_name, &second.address).await;

    Ok(())
}

fn address_suffix(address: &str) -> &str {
    let suffix_len = 8.min(address.len());
    &address[address.len() - suffix_len..]
}
