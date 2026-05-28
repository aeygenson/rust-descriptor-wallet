mod common;

use common::*;
use serial_test::file_serial;
use wallet_api::factory::build_default_api;

#[tokio::test]
#[file_serial]
async fn descriptor_info_returns_redacted_wallet_metadata() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-descriptor")
        .await
        .expect("clone wallet for descriptor test");
    let wallet_name = wallet_name.as_str();

    let info = api.descriptor_info(wallet_name).await?;

    assert_eq!(info.wallet_name, wallet_name);
    assert_eq!(info.network, "regtest");
    assert!(info.external.script_type.is_some());
    assert!(info.external.has_wildcards);
    assert!(info.contains_private_data);
    assert!(info.external.has_private_keys);
    assert!(
        info.external
            .descriptor_redacted
            .contains("<redacted-extended-private-key>"),
        "expected redacted descriptor, got {}",
        info.external.descriptor_redacted
    );
    assert!(
        !info.external.descriptor_redacted.contains("tprv"),
        "redacted descriptor must not expose private extended keys: {}",
        info.external.descriptor_redacted
    );

    let internal = info
        .internal
        .expect("descriptor info should expose internal/change branch");
    assert_eq!(internal.script_type, info.external.script_type);
    assert!(internal.has_private_keys);
    assert_eq!(internal.has_origin_info, info.external.has_origin_info);
    assert_eq!(internal.has_wildcards, info.external.has_wildcards);
    assert!(
        internal
            .descriptor_redacted
            .contains("<redacted-extended-private-key>"),
        "expected redacted internal descriptor, got {}",
        internal.descriptor_redacted
    );

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn descriptor_info_missing_wallet_returns_error() -> wallet_api::WalletApiResult<()> {
    let api = build_default_api().await?;

    let result = api.descriptor_info("wallet-that-does-not-exist").await;

    assert!(result.is_err());

    Ok(())
}
