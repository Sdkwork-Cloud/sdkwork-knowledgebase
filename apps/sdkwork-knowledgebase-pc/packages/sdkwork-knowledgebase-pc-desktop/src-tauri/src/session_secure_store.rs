use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use keyring::Entry;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const KEYRING_SERVICE: &str = "sdkwork-knowledgebase-pc";
const LEGACY_SNAPSHOT_FILE: &str = "secure-session.json";
const KEY_INDEX_FILE: &str = "secure-session-keys.json";
const SESSION_STORAGE_KEY: &str = "sdkwork-knowledgebase-pc-session";
const WECHAT_CREDENTIAL_PREFIX: &str = "sdkwork.knowledgebase.pc.wechat.credentials.v1.";
const MAX_SECURE_KEY_BYTES: usize = 256;
const MAX_SECURE_VALUE_BYTES: usize = 256 * 1024;
const MAX_TRACKED_KEYS: usize = 512;
const MAX_KEY_INDEX_BYTES: u64 = 64 * 1024;

#[derive(Debug, Default, Serialize, Deserialize)]
struct SecureSessionSnapshot {
    values: HashMap<String, String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SecureSessionKeyIndex {
    keys: Vec<String>,
}

pub struct SecureSessionState {
    keys_path: PathBuf,
    keys: Mutex<Vec<String>>,
}

impl SecureSessionState {
    fn new(app_data_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;

        let keys_path = app_data_dir.join(KEY_INDEX_FILE);
        let legacy_path = app_data_dir.join(LEGACY_SNAPSHOT_FILE);
        let mut keys = load_key_index(&keys_path);

        if legacy_path.exists() {
            migrate_legacy_snapshot(&legacy_path, &mut keys)?;
            let _ = fs::remove_file(&legacy_path);
        }

        persist_key_index(&keys_path, &keys)?;
        Ok(Self {
            keys_path,
            keys: Mutex::new(keys),
        })
    }

    fn track_key(&self, key: &str) -> Result<(), String> {
        let mut keys = self
            .keys
            .lock()
            .map_err(|_| "secure session lock poisoned".to_string())?;
        if !keys.iter().any(|existing| existing == key) {
            if keys.len() >= MAX_TRACKED_KEYS {
                return Err("secure session key limit exceeded".to_string());
            }
            keys.push(key.to_string());
            persist_key_index(&self.keys_path, &keys)?;
        }
        Ok(())
    }

    fn untrack_key(&self, key: &str) -> Result<(), String> {
        let mut keys = self
            .keys
            .lock()
            .map_err(|_| "secure session lock poisoned".to_string())?;
        let original_len = keys.len();
        keys.retain(|existing| existing != key);
        if keys.len() != original_len {
            persist_key_index(&self.keys_path, &keys)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecureSessionKeyRequest {
    key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecureSessionWriteRequest {
    key: String,
    value: String,
}

fn keyring_entry(key: &str) -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, key).map_err(|error| error.to_string())
}

fn validate_secure_key(key: &str) -> Result<&str, String> {
    if key.is_empty() || key.len() > MAX_SECURE_KEY_BYTES || key.trim() != key {
        return Err("secure session key is invalid".to_string());
    }
    let is_allowed_namespace =
        key == SESSION_STORAGE_KEY || key.starts_with(WECHAT_CREDENTIAL_PREFIX);
    let has_safe_characters = key
        .bytes()
        .all(|value| value.is_ascii_alphanumeric() || matches!(value, b'.' | b'-' | b'_' | b':'));
    if !is_allowed_namespace || !has_safe_characters {
        return Err("secure session key is outside the allowed namespace".to_string());
    }
    Ok(key)
}

fn validate_secure_value(value: &str) -> Result<(), String> {
    if value.len() > MAX_SECURE_VALUE_BYTES {
        return Err("secure session value exceeds the maximum allowed size".to_string());
    }
    Ok(())
}

fn load_key_index(path: &Path) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }
    if fs::metadata(path)
        .map(|metadata| metadata.len() > MAX_KEY_INDEX_BYTES)
        .unwrap_or(true)
    {
        return Vec::new();
    }
    let raw = fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str::<SecureSessionKeyIndex>(&raw)
        .map(|index| {
            index
                .keys
                .into_iter()
                .filter(|key| validate_secure_key(key).is_ok())
                .take(MAX_TRACKED_KEYS)
                .collect()
        })
        .unwrap_or_default()
}

fn persist_key_index(path: &Path, keys: &[String]) -> Result<(), String> {
    let payload = SecureSessionKeyIndex {
        keys: keys.to_vec(),
    };
    let serialized = serde_json::to_string_pretty(&payload)
        .map_err(|error: serde_json::Error| error.to_string())?;
    fs::write(path, serialized).map_err(|error| error.to_string())
}

fn migrate_legacy_snapshot(legacy_path: &Path, keys: &mut Vec<String>) -> Result<(), String> {
    if fs::metadata(legacy_path)
        .map_err(|error| error.to_string())?
        .len()
        > MAX_SECURE_VALUE_BYTES as u64
    {
        return Err("legacy secure session snapshot exceeds the maximum allowed size".to_string());
    }
    let raw = fs::read_to_string(legacy_path).map_err(|error| error.to_string())?;
    let snapshot = serde_json::from_str::<SecureSessionSnapshot>(&raw).unwrap_or_default();
    for (key, value) in snapshot.values {
        if validate_secure_key(&key).is_err() || validate_secure_value(&value).is_err() {
            continue;
        }
        keyring_entry(&key)?
            .set_password(&value)
            .map_err(|error| error.to_string())?;
        if !keys.iter().any(|existing| existing == &key) {
            keys.push(key);
        }
    }
    Ok(())
}

pub fn init_secure_session_state(app: &AppHandle) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let state = SecureSessionState::new(app_data_dir)?;
    app.manage(state);
    Ok(())
}

#[tauri::command]
pub fn write_secure_session_value(
    state: tauri::State<'_, SecureSessionState>,
    request: SecureSessionWriteRequest,
) -> Result<(), String> {
    let key = validate_secure_key(&request.key)?;
    validate_secure_value(&request.value)?;
    keyring_entry(key)?
        .set_password(&request.value)
        .map_err(|error| error.to_string())?;
    state.track_key(key)
}

#[tauri::command]
pub fn remove_secure_session_value(
    state: tauri::State<'_, SecureSessionState>,
    request: SecureSessionKeyRequest,
) -> Result<(), String> {
    let key = validate_secure_key(&request.key)?;
    if let Ok(entry) = keyring_entry(key) {
        let _ = entry.delete_credential();
    }
    state.untrack_key(key)
}

#[tauri::command]
pub fn clear_secure_session_values(
    state: tauri::State<'_, SecureSessionState>,
) -> Result<(), String> {
    let keys = state
        .keys
        .lock()
        .map_err(|_| "secure session lock poisoned".to_string())?
        .clone();
    for key in keys {
        if let Ok(entry) = keyring_entry(&key) {
            let _ = entry.delete_credential();
        }
    }
    {
        let mut keys = state
            .keys
            .lock()
            .map_err(|_| "secure session lock poisoned".to_string())?;
        keys.clear();
        persist_key_index(&state.keys_path, &keys)?;
    }
    Ok(())
}

#[tauri::command]
pub fn read_secure_session_value(
    request: SecureSessionKeyRequest,
) -> Result<Option<String>, String> {
    let key = validate_secure_key(&request.key)?;
    match keyring_entry(key)?.get_password() {
        Ok(value) => {
            validate_secure_value(&value)?;
            Ok(Some(value))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secure_key_validation_allows_only_owned_namespaces() {
        assert!(validate_secure_key(SESSION_STORAGE_KEY).is_ok());
        assert!(validate_secure_key(
            "sdkwork.knowledgebase.pc.wechat.credentials.v1.applet.42.appSecret"
        )
        .is_ok());
        assert!(validate_secure_key("untrusted.secret").is_err());
        assert!(validate_secure_key("sdkwork-knowledgebase-pc-session\nother").is_err());
    }

    #[test]
    fn secure_value_validation_is_bounded() {
        assert!(validate_secure_value(&"x".repeat(MAX_SECURE_VALUE_BYTES)).is_ok());
        assert!(validate_secure_value(&"x".repeat(MAX_SECURE_VALUE_BYTES + 1)).is_err());
    }
}
