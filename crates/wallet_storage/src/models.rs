use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// SQLite row model for a stored wallet definition.
///
/// Backend fields are stored as JSON strings so the database schema can stay
/// flexible while Rust uses strongly typed backend config models elsewhere.
#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct WalletRecord {
    pub id: i64,
    pub name: String,
    pub network: String,
    pub external_descriptor: String,
    pub internal_descriptor: String,
    pub db_path: String,
    pub sync_backend: String,
    pub broadcast_backend: Option<String>,
    pub is_watch_only: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl WalletRecord {
    pub fn parse_sync_backend(&self) -> Result<SyncBackendFile, serde_json::Error> {
        serde_json::from_str(&self.sync_backend)
    }

    pub fn parse_broadcast_backend(&self) -> Result<Option<BroadcastBackendFile>, serde_json::Error> {
        match &self.broadcast_backend {
            Some(b) => Ok(Some(serde_json::from_str(b)?)),
            None => Ok(None),
        }
    }
}

/// Descriptor pair used by imported/exported wallet config files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDescriptorsFile {
    pub external: String,
    pub internal: String,
}

/// Sync backend configuration stored in wallet JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyncBackendFile {
    Esplora { url: String },
    Electrum { url: String },
}

/// Broadcast backend configuration stored in wallet JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BroadcastBackendFile {
    Esplora { url: String },
    Rpc {
        url: String,
        rpc_user: String,
        rpc_pass: String,
    },
}

/// Top-level backend configuration for imported/exported wallet JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBackendFile {
    pub sync: SyncBackendFile,
    pub broadcast: Option<BroadcastBackendFile>,
}

/// New wallet JSON import/export format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWalletFile {
    pub name: String,
    pub network: String,
    pub descriptors: WalletDescriptorsFile,
    pub backend: WalletBackendFile,
    pub is_watch_only: bool,
}

impl ImportWalletFile {
    pub fn serialize_backends(
        &self,
    ) -> Result<(String, Option<String>), serde_json::Error> {
        let sync = serde_json::to_string(&self.backend.sync)?;
        let broadcast = match &self.backend.broadcast {
            Some(b) => Some(serde_json::to_string(b)?),
            None => None,
        };
        Ok((sync, broadcast))
    }
}