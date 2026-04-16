use crate::{WalletStorageError, WalletStorageResult};
use dirs::home_dir;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

pub fn default_app_path() -> WalletStorageResult<PathBuf> {
    let home = home_dir().ok_or(WalletStorageError::HomeDirNotFound)?;
    Ok(home.join(".rust-descriptor-wallet"))
}

pub fn default_db_path() -> WalletStorageResult<PathBuf> {
    Ok(default_app_path()?.join("app.db"))
}

pub fn default_wallet_dir(wallet_name: &str) -> WalletStorageResult<PathBuf> {
    Ok(default_app_path()?.join("wallets").join(wallet_name))
}

pub fn default_wallet_db_path(wallet_name: &str) -> WalletStorageResult<PathBuf> {
    Ok(default_wallet_dir(wallet_name)?.join("wallet.db"))
}

pub async fn connect() -> WalletStorageResult<SqlitePool> {
    let db_path = default_db_path()?;
    //println!("DB path: {}", db_path.display());

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if !db_path.exists() {
        File::create(&db_path)?;
    }

    let options = SqliteConnectOptions::from_str(&format!("sqlite:///{}", db_path.display()))?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn migrate(pool: &SqlitePool) -> WalletStorageResult<()> {
    let sql = include_str!("../migrations/0001_init.sql");
    sqlx::query(sql).execute(pool).await?;
    Ok(())
}
