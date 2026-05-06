use wallet_api::WalletApi;

#[tokio::test]
async fn wallet_api_status_smoke() {
    // Initialize API (same as app startup)
    let api = WalletApi::new()
        .await
        .expect("failed to initialize WalletApi");

    // List wallets
    let wallets = api.list_wallets().await.expect("failed to list wallets");

    // We expect at least one wallet to exist in dev environment
    assert!(!wallets.is_empty(), "no wallets found");

    // Take first wallet and fetch status
    let wallet = &wallets[0];

    let status = api
        .status(&wallet.name)
        .await
        .expect("failed to get wallet status");

    // Ensure we received a plausible status: either synced height is known,
    // or there are UTXOs, or balance is zero (fresh wallet)
    assert!(status.last_block_height.is_some() || status.utxo_count > 0 || status.balance == 0);
}
