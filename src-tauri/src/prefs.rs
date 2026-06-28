use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub last_vault: Option<String>,
    pub recent_vaults: Vec<String>,
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

    pub fn save(&self, app: &tauri::AppHandle) {
        let path = Self::path(app);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, content);
        }
    }
}
