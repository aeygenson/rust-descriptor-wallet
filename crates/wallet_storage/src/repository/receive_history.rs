use crate::{ReceiveAddressHistoryRecord, WalletStorageError, WalletStorageResult};
use sqlx::SqlitePool;

pub async fn record_receive_address(
    pool: &SqlitePool,
    wallet_name: &str,
    address: &str,
    keychain: &str,
    address_index: Option<i64>,
    bitcoin_uri: &str,
) -> WalletStorageResult<ReceiveAddressHistoryRecord> {
    let insert_result = sqlx::query(
        r#"
        INSERT INTO receive_address_history (
            wallet_name,
            address,
            keychain,
            address_index,
            bitcoin_uri
        )
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(wallet_name)
    .bind(address)
    .bind(keychain)
    .bind(address_index)
    .bind(bitcoin_uri)
    .execute(pool)
    .await;

    match insert_result {
        Ok(_) => {}
        Err(sqlx::Error::Database(db_err)) => {
            if db_err
                .message()
                .to_ascii_lowercase()
                .contains("unique")
            {
                return get_receive_address_by_wallet_and_address(pool, wallet_name, address)
                    .await?
                    .ok_or_else(|| {
                        WalletStorageError::AlreadyExists(format!(
                            "receive address already exists: {wallet_name}:{address}"
                        ))
                    });
            }

            return Err(WalletStorageError::Database(sqlx::Error::Database(
                db_err,
            )));
        }
        Err(err) => {
            return Err(WalletStorageError::Database(err));
        }
    }

    get_receive_address_by_wallet_and_address(pool, wallet_name, address)
        .await?
        .ok_or_else(|| {
            WalletStorageError::NotFound(format!(
                "receive address not found after insert: {wallet_name}:{address}"
            ))
        })
}


pub async fn list_receive_addresses(
    pool: &SqlitePool,
    wallet_name: &str,
) -> WalletStorageResult<Vec<ReceiveAddressHistoryRecord>> {
    let records = sqlx::query_as::<_, ReceiveAddressHistoryRecord>(
        r#"
        SELECT
            id,
            wallet_name,
            address,
            keychain,
            address_index,
            bitcoin_uri,
            label,
            created_at,
            updated_at
        FROM receive_address_history
        WHERE wallet_name = ?1
        ORDER BY created_at DESC
        "#,
    )
    .bind(wallet_name)
    .fetch_all(pool)
    .await?;

    Ok(records)
}

pub async fn label_receive_address(
    pool: &SqlitePool,
    wallet_name: &str,
    address: &str,
    label: &str,
) -> WalletStorageResult<ReceiveAddressHistoryRecord> {
    let result = sqlx::query(
        r#"
        UPDATE receive_address_history
        SET
            label = ?3,
            updated_at = CURRENT_TIMESTAMP
        WHERE wallet_name = ?1
          AND address = ?2
        "#,
    )
    .bind(wallet_name)
    .bind(address)
    .bind(label)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(WalletStorageError::NotFound(format!(
            "receive address not found: {wallet_name}:{address}"
        )));
    }

    get_receive_address_by_wallet_and_address(pool, wallet_name, address)
        .await?
        .ok_or_else(|| {
            WalletStorageError::NotFound(format!(
                "receive address not found after label update: {wallet_name}:{address}"
            ))
        })
}

pub async fn clear_receive_address_label(
    pool: &SqlitePool,
    wallet_name: &str,
    address: &str,
) -> WalletStorageResult<ReceiveAddressHistoryRecord> {
    let result = sqlx::query(
        r#"
        UPDATE receive_address_history
        SET
            label = NULL,
            updated_at = CURRENT_TIMESTAMP
        WHERE wallet_name = ?1
          AND address = ?2
        "#,
    )
    .bind(wallet_name)
    .bind(address)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(WalletStorageError::NotFound(format!(
            "receive address not found: {wallet_name}:{address}"
        )));
    }

    get_receive_address_by_wallet_and_address(pool, wallet_name, address)
        .await?
        .ok_or_else(|| {
            WalletStorageError::NotFound(format!(
                "receive address not found after label clear: {wallet_name}:{address}"
            ))
        })
}

pub async fn get_receive_address_by_wallet_and_address(
    pool: &SqlitePool,
    wallet_name: &str,
    address: &str,
) -> WalletStorageResult<Option<ReceiveAddressHistoryRecord>> {
    let record = sqlx::query_as::<_, ReceiveAddressHistoryRecord>(
        r#"
        SELECT
            id,
            wallet_name,
            address,
            keychain,
            address_index,
            bitcoin_uri,
            label,
            created_at,
            updated_at
        FROM receive_address_history
        WHERE wallet_name = ?1
          AND address = ?2
        LIMIT 1
        "#,
    )
    .bind(wallet_name)
    .bind(address)
    .fetch_optional(pool)
    .await?;

    Ok(record)
}
