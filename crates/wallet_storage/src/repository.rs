use crate::db::default_wallet_db_path;
use crate::{WalletRecord, WalletStorageError, WalletStorageResult};
use sqlx::{SqlitePool};
use crate::models::ImportWalletFile;
use std::fs;
/// Fetch wallet by name
pub async fn get_wallet_by_name(
    pool: &SqlitePool,
    name: &str,
) -> WalletStorageResult<WalletRecord> {
    let wallet = sqlx::query_as::<_, WalletRecord>(
        r#"
        SELECT id, name, network, external_descriptor, internal_descriptor,
               db_path, esplora_url, is_watch_only
        FROM wallets
        WHERE name = ?1
        "#,
    )
    .bind(name)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| WalletStorageError::NotFound(name.to_string()))?;

    Ok(wallet)
}

/// List all wallets
pub async fn list_wallets(pool: &SqlitePool) -> WalletStorageResult<Vec<WalletRecord>> {
    let wallets = sqlx::query_as::<_, WalletRecord>(
        r#"
        SELECT id, name, network, external_descriptor, internal_descriptor,
               db_path, esplora_url, is_watch_only
        FROM wallets
        ORDER BY id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(wallets)
}

/// Create wallet
pub async fn create_wallet(
    pool: &SqlitePool,
    name: &str,
    network: &str,
    external_descriptor: &str,
    internal_descriptor: &str,
    esplora_url: &str,
    is_watch_only: bool,
) -> WalletStorageResult<()> {
    let db_path = default_wallet_db_path(name)?;
    let db_path_str = db_path.to_string_lossy().to_string();

    sqlx::query(
        r#"
        INSERT INTO wallets (
            name,
            network,
            external_descriptor,
            internal_descriptor,
            db_path,
            esplora_url,
            is_watch_only
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(name)
    .bind(network)
    .bind(external_descriptor)
    .bind(internal_descriptor)
    .bind(db_path_str)
    .bind(esplora_url)
    .bind(if is_watch_only { 1_i64 } else { 0_i64 })
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete wallet
pub async fn delete_wallet(pool: &SqlitePool, name: &str) -> WalletStorageResult<()> {
    let result = sqlx::query("DELETE FROM wallets WHERE name = ?1")
        .bind(name)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(WalletStorageError::NotFound(name.to_string()));
    }

    Ok(())
}



pub async fn import_wallet_from_file(
    pool: &SqlitePool,
    file_path: &str,
) -> WalletStorageResult<()> {
    let content = fs::read_to_string(file_path)?;
    
    let wallet: ImportWalletFile =
        serde_json::from_str(&content)?;
    
    create_wallet(
        pool,
        &wallet.name,
        &wallet.network,
        &wallet.external_descriptor,
        &wallet.internal_descriptor,
        &wallet.esplora_url,
        wallet.is_watch_only,
    )
        .await?;
    
    Ok(())
}