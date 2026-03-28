use crate::{WalletApiError, WalletApiResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use wallet_storage::{default_app_path, WalletStorage, WalletStorageError};

/// Markers are lightweight app-level metadata stored in a JSON file.
///
/// Examples:
/// - last selected wallet
/// - last sync timestamp
/// - onboarding flags
/// - one-off UI state
///
/// They are *not* a replacement for wallet state, transaction history,
/// or descriptor storage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AppMarkers {
    markers: HashMap<String, String>,
}

fn io_error(err: std::io::Error) -> WalletApiError {
    WalletStorageError::IO(err).into()
}

fn serde_error(err: serde_json::Error) -> WalletApiError {
    WalletStorageError::Serialization(err).into()
}

/// Resolve path to app_state.json.
fn markers_file_path() -> WalletApiResult<PathBuf> {
    let dir = default_app_path()?;
    Ok(dir.join("app_state.json"))
}

/// Load all markers from the JSON file.
fn load_all_markers() -> WalletApiResult<AppMarkers> {
    let path = markers_file_path()?;

    if !path.exists() {
        return Ok(AppMarkers::default());
    }

    let content = fs::read_to_string(&path).map_err(io_error)?;

    if content.trim().is_empty() {
        return Ok(AppMarkers::default());
    }

    let state = serde_json::from_str(&content).map_err(serde_error)?;
    Ok(state)
}

/// Save all markers to the JSON file.
fn save_all_markers(state: &AppMarkers) -> WalletApiResult<()> {
    let path = markers_file_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }

    let content = serde_json::to_string_pretty(state).map_err(serde_error)?;
    fs::write(path, content).map_err(io_error)?;

    Ok(())
}

/// Load a marker value by key.
pub async fn load_marker(_storage: &WalletStorage, key: &str) -> WalletApiResult<String> {
    let state = load_all_markers()?;

    state
        .markers
        .get(key)
        .cloned()
        .ok_or_else(|| WalletStorageError::NotFound(format!("marker not found: {key}")).into())
}

/// Save or update a marker value by key.
pub async fn save_marker(
    _storage: &WalletStorage,
    key: &str,
    value: &str,
) -> WalletApiResult<String> {
    let mut state = load_all_markers()?;
    state.markers.insert(key.to_string(), value.to_string());
    save_all_markers(&state)?;
    Ok(value.to_string())
}