

use sqlx::SqlitePool;

use crate::{models::AddressBookEntryRecord, WalletStorageError, WalletStorageResult};

fn map_address_book_error(err: sqlx::Error) -> WalletStorageError {
    if let sqlx::Error::Database(db_err) = &err {
        let message = db_err.message().to_ascii_lowercase();

        if message.contains("uq_address_book_wallet_label") {
            return WalletStorageError::DuplicateAddressBookLabel(db_err.message().to_string());
        }

        if message.contains("uq_address_book_wallet_address") {
            return WalletStorageError::DuplicateAddressBookAddress(db_err.message().to_string());
        }
    }

    WalletStorageError::from(err)
}

#[derive(Clone)]
pub struct AddressBookRepository {
    pool: SqlitePool,
}

impl AddressBookRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_entry(
        &self,
        wallet_name: &str,
        network: &str,
        label: &str,
        address: &str,
        notes: Option<&str>,
    ) -> WalletStorageResult<AddressBookEntryRecord> {
        let record = sqlx::query_as::<_, AddressBookEntryRecord>(
            r#"
            INSERT INTO address_book_entries (
                wallet_name,
                network,
                label,
                address,
                notes
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            RETURNING
                id,
                wallet_name,
                network,
                label,
                address,
                notes,
                created_at,
                updated_at
            "#,
        )
        .bind(wallet_name)
        .bind(network)
        .bind(label)
        .bind(address)
        .bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(map_address_book_error)?;

        Ok(record)
    }

    pub async fn list_entries(
        &self,
        wallet_name: &str,
    ) -> WalletStorageResult<Vec<AddressBookEntryRecord>> {
        let records = sqlx::query_as::<_, AddressBookEntryRecord>(
            r#"
            SELECT
                id,
                wallet_name,
                network,
                label,
                address,
                notes,
                created_at,
                updated_at
            FROM address_book_entries
            WHERE wallet_name = ?1
            ORDER BY label ASC
            "#,
        )
        .bind(wallet_name)
        .fetch_all(&self.pool)
        .await
        .map_err(WalletStorageError::from)?;

        Ok(records)
    }

    pub async fn get_entry_by_address(
        &self,
        wallet_name: &str,
        address: &str,
    ) -> WalletStorageResult<Option<AddressBookEntryRecord>> {
        let record = sqlx::query_as::<_, AddressBookEntryRecord>(
            r#"
            SELECT
                id,
                wallet_name,
                network,
                label,
                address,
                notes,
                created_at,
                updated_at
            FROM address_book_entries
            WHERE wallet_name = ?1
              AND address = ?2
            LIMIT 1
            "#,
        )
        .bind(wallet_name)
        .bind(address)
        .fetch_optional(&self.pool)
        .await
        .map_err(WalletStorageError::from)?;

        Ok(record)
    }

    pub async fn delete_entry(
        &self,
        wallet_name: &str,
        address: &str,
    ) -> WalletStorageResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM address_book_entries
            WHERE wallet_name = ?1
              AND address = ?2
            "#,
        )
        .bind(wallet_name)
        .bind(address)
        .execute(&self.pool)
        .await
        .map_err(WalletStorageError::from)?;

        Ok(result.rows_affected() > 0)
    }
}

pub async fn create_entry(
    pool: &SqlitePool,
    wallet_name: &str,
    network: &str,
    label: &str,
    address: &str,
    notes: Option<&str>,
) -> WalletStorageResult<AddressBookEntryRecord> {
    AddressBookRepository::new(pool.clone())
        .create_entry(wallet_name, network, label, address, notes)
        .await
}

pub async fn list_entries(
    pool: &SqlitePool,
    wallet_name: &str,
) -> WalletStorageResult<Vec<AddressBookEntryRecord>> {
    AddressBookRepository::new(pool.clone())
        .list_entries(wallet_name)
        .await
}

pub async fn get_entry_by_address(
    pool: &SqlitePool,
    wallet_name: &str,
    address: &str,
) -> WalletStorageResult<Option<AddressBookEntryRecord>> {
    AddressBookRepository::new(pool.clone())
        .get_entry_by_address(wallet_name, address)
        .await
}

pub async fn delete_entry(
    pool: &SqlitePool,
    wallet_name: &str,
    address: &str,
) -> WalletStorageResult<bool> {
    AddressBookRepository::new(pool.clone())
        .delete_entry(wallet_name, address)
        .await
}