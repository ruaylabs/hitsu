use tauri::AppHandle;

use crate::error::KagiResult;
use crate::prefs::Preferences;

#[tauri::command]
pub async fn prefs_get(app: AppHandle) -> KagiResult<Preferences> {
    Ok(Preferences::load(&app))
}

#[tauri::command]
pub async fn prefs_set_last_vault(app: AppHandle, path: String) -> KagiResult<()> {
    let mut prefs = Preferences::load(&app);
    prefs.last_vault = Some(path.clone());
    // Add to recent vaults, dedup
    prefs.recent_vaults.retain(|v| v != &path);
    prefs.recent_vaults.insert(0, path);
    if prefs.recent_vaults.len() > 10 {
        prefs.recent_vaults.truncate(10);
    }
    prefs.save(&app);
    Ok(())
}

#[tauri::command]
pub async fn prefs_set_security(
    app: AppHandle,
    idle_lock_minutes: u32,
    clipboard_clear_seconds: u32,
) -> KagiResult<()> {
    let mut prefs = Preferences::load(&app);
    prefs.idle_lock_minutes = idle_lock_minutes;
    prefs.clipboard_clear_seconds = clipboard_clear_seconds;
    prefs.save(&app);
    Ok(())
}
