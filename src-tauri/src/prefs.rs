use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

use crate::error::{KagiError, KagiResult};
use crate::vault::atomic_write;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub last_vault: Option<String>,
    pub recent_vaults: Vec<String>,
    /// Idle lock timeout in minutes (0 = never). Default: 5.
    #[serde(default = "default_idle_lock")]
    pub idle_lock_minutes: u32,
    /// Clipboard auto-clear timeout in seconds. Default: 15.
    #[serde(default = "default_clipboard_clear")]
    pub clipboard_clear_seconds: u32,
    /// Vault paths for which the user dismissed the KDF-upgrade prompt.
    /// Per-vault so dismissing on one weak vault doesn't hide the prompt on
    /// a different weak vault.
    #[serde(default)]
    pub kdf_upgrade_dismissed_vaults: Vec<String>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            last_vault: None,
            recent_vaults: Vec::new(),
            idle_lock_minutes: default_idle_lock(),
            clipboard_clear_seconds: default_clipboard_clear(),
            kdf_upgrade_dismissed_vaults: Vec::new(),
        }
    }
}

fn default_idle_lock() -> u32 {
    5
}

fn default_clipboard_clear() -> u32 {
    15
}

impl Preferences {
    pub fn path(app: &tauri::AppHandle) -> PathBuf {
        let data_dir = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        data_dir.join("prefs.json")
    }

    pub fn load(app: &tauri::AppHandle) -> Self {
        let path = Self::path(app);
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, app: &tauri::AppHandle) -> KagiResult<()> {
        let path = Self::path(app);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|_| KagiError::Custom("Could not serialize preferences".to_string()))?;
        atomic_write(&path, content.as_bytes())?;
        Ok(())
    }
}
