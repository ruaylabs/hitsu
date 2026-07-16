use tauri::{AppHandle, State};

use crate::error::KagiResult;
use crate::prefs::Preferences;
use crate::state::AppState;

fn update_preferences(app: &AppHandle, update: impl FnOnce(&mut Preferences)) -> KagiResult<()> {
    let mut prefs = Preferences::load(app);
    update(&mut prefs);
    prefs.save(app)
}

#[tauri::command]
pub async fn prefs_get(app: AppHandle) -> KagiResult<Preferences> {
    Ok(Preferences::load(&app))
}

#[tauri::command]
pub async fn prefs_set_last_vault(app: AppHandle, path: String) -> KagiResult<()> {
    update_preferences(&app, |prefs| {
        prefs.last_vault = Some(path.clone());
        // Add to recent vaults, dedup
        prefs.recent_vaults.retain(|vault| vault != &path);
        prefs.recent_vaults.insert(0, path);
        prefs.recent_vaults.truncate(10);
    })
}

#[tauri::command]
pub async fn prefs_set_security(
    app: AppHandle,
    state: State<'_, AppState>,
    idle_lock_minutes: u32,
    clipboard_clear_seconds: u32,
) -> KagiResult<()> {
    update_preferences(&app, |prefs| {
        prefs.idle_lock_minutes = idle_lock_minutes;
        prefs.clipboard_clear_seconds = clipboard_clear_seconds;
    })?;
    state.configure_idle_lock(idle_lock_minutes);
    Ok(())
}

#[tauri::command]
pub async fn prefs_set_kdf_dismissed(
    app: AppHandle,
    path: String,
    dismissed: bool,
) -> KagiResult<()> {
    update_preferences(&app, |prefs| {
        prefs
            .kdf_upgrade_dismissed_vaults
            .retain(|vault| vault != &path);
        if dismissed {
            prefs.kdf_upgrade_dismissed_vaults.push(path);
        }
    })
}
