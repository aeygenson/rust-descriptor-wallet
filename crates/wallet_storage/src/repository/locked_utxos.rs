

use sqlx::SqlitePool;

use crate::{models::LockedUtxoRecord, WalletStorageError, WalletStorageResult};

fn map_locked_utxo_error(err: sqlx::Error) -> WalletStorageError {
    if let sqlx::Error::Database(db_err) = &err {
        let message = db_err.message().to_ascii_lowercase();

        if message.contains("uq_locked_utxos_wallet_outpoint") {
            return WalletStorageError::DuplicateLockedUtxo(db_err.message().to_string());
        }
    }

    WalletStorageError::from(err)
}

#[derive(Clone)]
pub struct LockedUtxoRepository {
    pool: SqlitePool,
}

impl LockedUtxoRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn lock_utxo(
        &self,
        wallet_name: &str,
        outpoint: &str,
        reason: Option<&str>,
    ) -> WalletStorageResult<LockedUtxoRecord> {
        let record = sqlx::query_as::<_, LockedUtxoRecord>(
            r#"
            INSERT INTO locked_utxos (
                wallet_name,
                outpoint,
                reason
            )
            VALUES (?1, ?2, ?3)
            RETURNING
                id,
                wallet_name,
                outpoint,
                reason,
                created_at,
                updated_at
            "#,
        )
        .bind(wallet_name)
        .bind(outpoint)
        .bind(reason)
        .fetch_one(&self.pool)
        .await
        .map_err(map_locked_utxo_error)?;

        Ok(record)
    }

    pub async fn list_locked_utxos(
        &self,
        wallet_name: &str,
    ) -> WalletStorageResult<Vec<LockedUtxoRecord>> {
        let records = sqlx::query_as::<_, LockedUtxoRecord>(
            r#"
            SELECT
                id,
                wallet_name,
                outpoint,
                reason,
                created_at,
                updated_at
            FROM locked_utxos
            WHERE wallet_name = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(wallet_name)
        .fetch_all(&self.pool)
        .await
        .map_err(WalletStorageError::from)?;

        Ok(records)
    }

    pub async fn get_locked_utxo(
        &self,
        wallet_name: &str,
        outpoint: &str,
    ) -> WalletStorageResult<Option<LockedUtxoRecord>> {
        let record = sqlx::query_as::<_, LockedUtxoRecord>(
            r#"
            SELECT
                id,
                wallet_name,
                outpoint,
                reason,
                created_at,
                updated_at
            FROM locked_utxos
            WHERE wallet_name = ?1
              AND outpoint = ?2
            LIMIT 1
            "#,
        )
        .bind(wallet_name)
        .bind(outpoint)
        .fetch_optional(&self.pool)
        .await
        .map_err(WalletStorageError::from)?;

        Ok(record)
    }

    pub async fn is_locked(
        &self,
        wallet_name: &str,
        outpoint: &str,
    ) -> WalletStorageResult<bool> {
        let record = self.get_locked_utxo(wallet_name, outpoint).await?;
        Ok(record.is_some())
    }

    pub async fn unlock_utxo(
        &self,
        wallet_name: &str,
        outpoint: &str,
    ) -> WalletStorageResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM locked_utxos
            WHERE wallet_name = ?1
              AND outpoint = ?2
            "#,
        )
        .bind(wallet_name)
        .bind(outpoint)
        .execute(&self.pool)
        .await
        .map_err(WalletStorageError::from)?;

        Ok(result.rows_affected() > 0)
    }
}

pub async fn lock_utxo(
    pool: &SqlitePool,
    wallet_name: &str,
    outpoint: &str,
    reason: Option<&str>,
) -> WalletStorageResult<LockedUtxoRecord> {
    LockedUtxoRepository::new(pool.clone())
        .lock_utxo(wallet_name, outpoint, reason)
        .await
}

pub async fn list_locked_utxos(
    pool: &SqlitePool,
    wallet_name: &str,
) -> WalletStorageResult<Vec<LockedUtxoRecord>> {
    LockedUtxoRepository::new(pool.clone())
        .list_locked_utxos(wallet_name)
        .await
}

pub async fn get_locked_utxo(
    pool: &SqlitePool,
    wallet_name: &str,
    outpoint: &str,
) -> WalletStorageResult<Option<LockedUtxoRecord>> {
    LockedUtxoRepository::new(pool.clone())
        .get_locked_utxo(wallet_name, outpoint)
        .await
}

pub async fn is_locked(
    pool: &SqlitePool,
    wallet_name: &str,
    outpoint: &str,
) -> WalletStorageResult<bool> {
    LockedUtxoRepository::new(pool.clone())
        .is_locked(wallet_name, outpoint)
        .await
}

pub async fn unlock_utxo(
    pool: &SqlitePool,
    wallet_name: &str,
    outpoint: &str,
) -> WalletStorageResult<bool> {
    LockedUtxoRepository::new(pool.clone())
        .unlock_utxo(wallet_name, outpoint)
        .await
}