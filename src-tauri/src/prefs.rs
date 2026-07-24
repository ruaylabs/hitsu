use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

use crate::error::{HitsuError, HitsuResult};
use crate::vault::atomic_write;

const LEGACY_IDENTIFIER: &str = "com.ruaylabs.kagi";
const PREFS_FILE: &str = "prefs.json";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

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
    /// Color scheme override. System follows the operating-system preference.
    #[serde(default)]
    pub theme: ThemePreference,
    /// Exposes the optional KDBX folder tree and entry-move controls.
    #[serde(default)]
    pub folders_enabled: bool,
    /// Starts the browser-extension IPC listener. Off by default: the
    /// integration is a developer preview, and the socket widens the local
    /// attack surface (see browser_ipc.rs).
    #[serde(default)]
    pub browser_integration_enabled: bool,
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
            theme: ThemePreference::default(),
            folders_enabled: false,
            browser_integration_enabled: false,
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
        data_dir.join(PREFS_FILE)
    }

    pub fn migrate_legacy(app: &tauri::AppHandle) -> HitsuResult<()> {
        let destination = Self::path(app);
        let Some(data_root) = destination.parent().and_then(Path::parent) else {
            return Ok(());
        };
        let source = data_root.join(LEGACY_IDENTIFIER).join(PREFS_FILE);
        Self::move_legacy_file(&source, &destination)
    }

    fn move_legacy_file(source: &Path, destination: &Path) -> HitsuResult<()> {
        if destination.exists() || !source.is_file() {
            return Ok(());
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(source, destination)?;
        Ok(())
    }

    pub fn load(app: &tauri::AppHandle) -> Self {
        Self::load_from(&Self::path(app))
    }

    pub(crate) fn load_from(path: &Path) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, app: &tauri::AppHandle) -> HitsuResult<()> {
        self.save_to(&Self::path(app))
    }

    pub(crate) fn save_to(&self, path: &Path) -> HitsuResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|_| HitsuError::Custom("Could not serialize preferences".to_string()))?;
        atomic_write(path, content.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Preferences;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("hitsu-prefs-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn moves_legacy_preferences_when_new_file_is_missing() {
        let root = temp_dir();
        let source = root.join("com.ruaylabs.kagi/prefs.json");
        let destination = root.join("com.ruaylabs.hitsu/prefs.json");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, r#"{"idleLockMinutes":10}"#).unwrap();

        Preferences::move_legacy_file(&source, &destination).unwrap();

        assert!(!source.exists());
        assert_eq!(
            fs::read_to_string(&destination).unwrap(),
            r#"{"idleLockMinutes":10}"#
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keeps_existing_preferences_instead_of_overwriting_them() {
        let root = temp_dir();
        let source = root.join("com.ruaylabs.kagi/prefs.json");
        let destination = root.join("com.ruaylabs.hitsu/prefs.json");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::write(&source, "legacy").unwrap();
        fs::write(&destination, "current").unwrap();

        Preferences::move_legacy_file(&source, &destination).unwrap();

        assert!(source.exists());
        assert_eq!(fs::read_to_string(&destination).unwrap(), "current");
        fs::remove_dir_all(root).unwrap();
    }
}
