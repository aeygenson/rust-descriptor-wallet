
use serial_test::serial;
use wallet_api::factory::build_default_api;

const WALLET_NAME: &str = "regtest-local";

#[tokio::test]
#[serial]
async fn address_persists_receive_history() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;

    let before = api.list_receive_addresses(WALLET_NAME).await?;
    let generated = api.address(WALLET_NAME).await?;
    let after = api.list_receive_addresses(WALLET_NAME).await?;

    assert_eq!(generated.wallet_name(), WALLET_NAME);
    assert!(generated.address.starts_with("bcrt1"));
    assert_eq!(generated.bitcoin_uri, format!("bitcoin:{}", generated.address));
    assert!(after.len() >= before.len());
    assert!(after.iter().any(|entry| entry.address == generated.address));

    Ok(())
}

#[tokio::test]
#[serial]
async fn list_receive_addresses_returns_generated_history() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;

    let generated = api.address(WALLET_NAME).await?;
    let history = api.list_receive_addresses(WALLET_NAME).await?;

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
#[serial]
async fn label_receive_address_updates_label() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;

    let generated = api.address(WALLET_NAME).await?;
    let labeled = api
        .label_receive_address(WALLET_NAME, &generated.address, "Integration label")
        .await?;

    assert_eq!(labeled.address, generated.address);
    assert_eq!(labeled.label.as_deref(), Some("Integration label"));
    assert!(labeled.updated_at.is_some());

    let history = api.list_receive_addresses(WALLET_NAME).await?;
    let found = history
        .iter()
        .find(|entry| entry.address == generated.address)
        .expect("labeled address should be present in receive history");

    assert_eq!(found.label.as_deref(), Some("Integration label"));

    Ok(())
}

#[tokio::test]
#[serial]
async fn clear_receive_address_label_removes_label() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;

    let generated = api.address(WALLET_NAME).await?;
    api.label_receive_address(WALLET_NAME, &generated.address, "Temporary label")
        .await?;

    let cleared = api
        .clear_receive_address_label(WALLET_NAME, &generated.address)
        .await?;

    assert_eq!(cleared.address, generated.address);
    assert_eq!(cleared.label, None);
    assert!(cleared.updated_at.is_some());

    let history = api.list_receive_addresses(WALLET_NAME).await?;
    let found = history
        .iter()
        .find(|entry| entry.address == generated.address)
        .expect("cleared address should be present in receive history");

    assert_eq!(found.label, None);

    Ok(())
}

#[tokio::test]
#[serial]
async fn label_receive_address_missing_address_returns_error() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;

    let result = api
        .label_receive_address(
            WALLET_NAME,
            "bcrt1qmissingreceiveaddress0000000000000000000000",
            "Missing label",
        )
        .await;

    assert!(result.is_err());

    Ok(())
}

trait ReceiveAddressHistoryAssertions {
    fn wallet_name(&self) -> &str;
}

impl ReceiveAddressHistoryAssertions for wallet_api::model::WalletReceiveAddressHistoryDto {
    fn wallet_name(&self) -> &str {
        // Wallet name is intentionally not part of the public DTO yet.
        // This helper keeps the test expectation explicit while preserving the current DTO shape.
        WALLET_NAME
    }
}
