

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use wallet_storage::{
    clear_receive_address_label, label_receive_address, list_receive_addresses,
    record_receive_address,
};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

async fn test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create in-memory sqlite pool");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("enable sqlite foreign keys");

    MIGRATOR.run(&pool).await.expect("run migrations");

    pool
}

async fn insert_wallet(pool: &SqlitePool, name: &str, network: &str) {
    sqlx::query(
        r#"
        INSERT INTO wallets (
            name,
            network,
            external_descriptor,
            internal_descriptor,
            db_path,
            sync_backend,
            broadcast_backend,
            is_watch_only
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(name)
    .bind(network)
    .bind("wpkh([00000000/84h/1h/0h]tpub-test/0/*)")
    .bind("wpkh([00000000/84h/1h/0h]tpub-test/1/*)")
    .bind(format!("/tmp/{name}/wallet.db"))
    .bind(r#"{"kind":"electrum","url":"tcp://127.0.0.1:50001"}"#)
    .bind(Option::<String>::None)
    .bind(false)
    .execute(pool)
    .await
    .expect("insert wallet fixture");
}

#[tokio::test]
async fn migrations_run_on_empty_database() {
    let pool = test_pool().await;

    let wallet_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wallets")
        .fetch_one(&pool)
        .await
        .expect("count wallets");

    assert_eq!(wallet_count, 0);
}

#[tokio::test]
async fn record_receive_address_persists_history() {
    let pool = test_pool().await;
    insert_wallet(&pool, "regtest-local", "regtest").await;

    let record = record_receive_address(
        &pool,
        "regtest-local",
        "bcrt1qexample000000000000000000000000000000000",
        "external",
        Some(0),
        "bitcoin:bcrt1qexample000000000000000000000000000000000",
    )
    .await
    .expect("record receive address");

    assert_eq!(record.wallet_name, "regtest-local");
    assert_eq!(record.keychain, "external");
    assert_eq!(record.address_index, Some(0));
    assert_eq!(record.label, None);

    let records = list_receive_addresses(&pool, "regtest-local")
        .await
        .expect("list receive addresses");

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].address, record.address);
}

#[tokio::test]
async fn list_receive_addresses_is_wallet_scoped() {
    let pool = test_pool().await;
    insert_wallet(&pool, "wallet-a", "regtest").await;
    insert_wallet(&pool, "wallet-b", "regtest").await;

    record_receive_address(
        &pool,
        "wallet-a",
        "bcrt1qwalleta00000000000000000000000000000000",
        "external",
        Some(0),
        "bitcoin:bcrt1qwalleta00000000000000000000000000000000",
    )
    .await
    .expect("record wallet-a receive address");

    record_receive_address(
        &pool,
        "wallet-b",
        "bcrt1qwalletb00000000000000000000000000000000",
        "external",
        Some(0),
        "bitcoin:bcrt1qwalletb00000000000000000000000000000000",
    )
    .await
    .expect("record wallet-b receive address");

    let wallet_a_records = list_receive_addresses(&pool, "wallet-a")
        .await
        .expect("list wallet-a receive addresses");

    assert_eq!(wallet_a_records.len(), 1);
    assert_eq!(wallet_a_records[0].wallet_name, "wallet-a");
    assert_eq!(wallet_a_records[0].address, "bcrt1qwalleta00000000000000000000000000000000");
}

#[tokio::test]
async fn record_receive_address_is_idempotent_for_duplicate_address() {
    let pool = test_pool().await;
    insert_wallet(&pool, "regtest-local", "regtest").await;

    let first = record_receive_address(
        &pool,
        "regtest-local",
        "bcrt1qduplicate000000000000000000000000000000",
        "external",
        Some(0),
        "bitcoin:bcrt1qduplicate000000000000000000000000000000",
    )
    .await
    .expect("record first receive address");

    let second = record_receive_address(
        &pool,
        "regtest-local",
        "bcrt1qduplicate000000000000000000000000000000",
        "external",
        Some(0),
        "bitcoin:bcrt1qduplicate000000000000000000000000000000",
    )
    .await
    .expect("record duplicate receive address");

    assert_eq!(first.id, second.id);

    let records = list_receive_addresses(&pool, "regtest-local")
        .await
        .expect("list receive addresses");

    assert_eq!(records.len(), 1);
}

#[tokio::test]
async fn label_receive_address_updates_label() {
    let pool = test_pool().await;
    insert_wallet(&pool, "regtest-local", "regtest").await;

    record_receive_address(
        &pool,
        "regtest-local",
        "bcrt1qlabeled0000000000000000000000000000000",
        "external",
        Some(0),
        "bitcoin:bcrt1qlabeled0000000000000000000000000000000",
    )
    .await
    .expect("record receive address");

    let updated = label_receive_address(
        &pool,
        "regtest-local",
        "bcrt1qlabeled0000000000000000000000000000000",
        "Savings top-up",
    )
    .await
    .expect("label receive address");

    assert_eq!(updated.label.as_deref(), Some("Savings top-up"));
    assert!(updated.updated_at.is_some());
}

#[tokio::test]
async fn clear_receive_address_label_removes_label() {
    let pool = test_pool().await;
    insert_wallet(&pool, "regtest-local", "regtest").await;

    record_receive_address(
        &pool,
        "regtest-local",
        "bcrt1qclearlabel0000000000000000000000000000",
        "external",
        Some(0),
        "bitcoin:bcrt1qclearlabel0000000000000000000000000000",
    )
    .await
    .expect("record receive address");

    label_receive_address(
        &pool,
        "regtest-local",
        "bcrt1qclearlabel0000000000000000000000000000",
        "Temporary label",
    )
    .await
    .expect("label receive address");

    let updated = clear_receive_address_label(
        &pool,
        "regtest-local",
        "bcrt1qclearlabel0000000000000000000000000000",
    )
    .await
    .expect("clear receive address label");

    assert_eq!(updated.label, None);
    assert!(updated.updated_at.is_some());
}

#[tokio::test]
async fn label_receive_address_returns_not_found_for_missing_address() {
    let pool = test_pool().await;
    insert_wallet(&pool, "regtest-local", "regtest").await;

    let result = label_receive_address(
        &pool,
        "regtest-local",
        "bcrt1qmissing0000000000000000000000000000000",
        "Missing label",
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn delete_wallet_cascades_receive_history() {
    let pool = test_pool().await;
    insert_wallet(&pool, "regtest-local", "regtest").await;

    record_receive_address(
        &pool,
        "regtest-local",
        "bcrt1qcascade0000000000000000000000000000000",
        "external",
        Some(0),
        "bitcoin:bcrt1qcascade0000000000000000000000000000000",
    )
    .await
    .expect("record receive address");

    sqlx::query("DELETE FROM wallets WHERE name = ?1")
        .bind("regtest-local")
        .execute(&pool)
        .await
        .expect("delete wallet");

    let records = list_receive_addresses(&pool, "regtest-local")
        .await
        .expect("list receive addresses");

    assert!(records.is_empty());
}

#[tokio::test]
async fn receive_history_storage_end_to_end_flow() {
    let pool = test_pool().await;
    insert_wallet(&pool, "wallet-e2e", "regtest").await;

    let first = record_receive_address(
        &pool,
        "wallet-e2e",
        "bcrt1qe2e00000000000000000000000000000000000",
        "external",
        Some(0),
        "bitcoin:bcrt1qe2e00000000000000000000000000000000000",
    )
    .await
    .expect("record first receive address");

    assert_eq!(first.wallet_name, "wallet-e2e");
    assert_eq!(first.label, None);

    let labeled = label_receive_address(
        &pool,
        "wallet-e2e",
        "bcrt1qe2e00000000000000000000000000000000000",
        "Primary receive",
    )
    .await
    .expect("label receive address");

    assert_eq!(labeled.label.as_deref(), Some("Primary receive"));

    let records = list_receive_addresses(&pool, "wallet-e2e")
        .await
        .expect("list receive addresses");

    assert_eq!(records.len(), 1);

    let cleared = clear_receive_address_label(
        &pool,
        "wallet-e2e",
        "bcrt1qe2e00000000000000000000000000000000000",
    )
    .await
    .expect("clear receive address label");

    assert_eq!(cleared.label, None);

    sqlx::query("DELETE FROM wallets WHERE name = ?1")
        .bind("wallet-e2e")
        .execute(&pool)
        .await
        .expect("delete wallet");

    let after_delete = list_receive_addresses(&pool, "wallet-e2e")
        .await
        .expect("list receive addresses after delete");

    assert!(after_delete.is_empty());
}