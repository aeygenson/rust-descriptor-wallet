use serde::{Deserialize,Serialize};
use sqlx::FromRow;
#[derive(Serialize, Deserialize, Debug,Clone,FromRow)]
pub  struct WalletRecord{
    pub id:i64,
    pub name:String,
    pub  network: String,
    pub  external_descriptor: String,
    pub  internal_descriptor: String,
    pub  db_path:String,
    pub  esplora_url:String,
    pub  is_watch_only:bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWalletFile {
    pub name: String,
    pub network: String,
    pub esplora_url: String,
    pub external_descriptor: String,
    pub internal_descriptor: String,
    pub is_watch_only: bool,
}