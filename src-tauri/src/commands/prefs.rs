use std::path::Path;

use parking_lot::Mutex;
use tauri::{AppHandle, State};

use crate::error::KagiResult;
use crate::prefs::Preferences;
use crate::state::AppState;

fn update_preferences_at_path(
    path: &Path,
    preference_lock: &Mutex<()>,
    update: impl FnOnce(&mut Preferences),
) -> KagiResult<()> {
    let _guard = preference_lock.lock();
    let mut prefs = Preferences::load_from(path);
    update(&mut prefs);
    prefs.save_to(path)
}

fn update_preferences(
    app: &AppHandle,
    state: &AppState,
    update: impl FnOnce(&mut Preferences),
) -> KagiResult<()> {
    update_preferences_at_path(&Preferences::path(app), &state.preference_lock, update)
}

#[tauri::command]
pub async fn prefs_get(app: AppHandle, state: State<'_, AppState>) -> KagiResult<Preferences> {
    let _guard = state.preference_lock.lock();
    Ok(Preferences::load(&app))
}

#[tauri::command]
pub async fn prefs_set_last_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> KagiResult<()> {
    update_preferences(&app, &state, |prefs| {
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
    update_preferences(&app, &state, |prefs| {
        prefs.idle_lock_minutes = idle_lock_minutes;
        prefs.clipboard_clear_seconds = clipboard_clear_seconds;
    })?;
    state.configure_idle_lock(idle_lock_minutes);
    Ok(())
}

#[tauri::command]
pub async fn prefs_set_folders_enabled(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> KagiResult<()> {
    update_preferences(&app, &state, |prefs| {
        prefs.folders_enabled = enabled;
    })
}

#[tauri::command]
pub async fn prefs_set_kdf_dismissed(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    dismissed: bool,
) -> KagiResult<()> {
    update_preferences(&app, &state, |prefs| {
        prefs
            .kdf_upgrade_dismissed_vaults
            .retain(|vault| vault != &path);
        if dismissed {
            prefs.kdf_upgrade_dismissed_vaults.push(path);
        }
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    use parking_lot::Mutex;

    use super::update_preferences_at_path;
    use crate::prefs::Preferences;

    #[test]
    fn concurrent_preference_mutations_preserve_every_update() {
        const WRITERS: usize = 12;
        let dir = std::env::temp_dir().join(format!("kagi-prefs-{}", uuid::Uuid::new_v4()));
        let path = Arc::new(dir.join("prefs.json"));
        let lock = Arc::new(Mutex::new(()));
        let start = Arc::new(Barrier::new(WRITERS + 1));

        let writers = (0..WRITERS)
            .map(|index| {
                let path = Arc::clone(&path);
                let lock = Arc::clone(&lock);
                let start = Arc::clone(&start);
                thread::spawn(move || {
                    start.wait();
                    update_preferences_at_path(&path, &lock, |prefs| {
                        // Widen the read-modify-write race enough that removing
                        // the lock reliably loses updates in this regression test.
                        thread::sleep(Duration::from_millis(10));
                        prefs.recent_vaults.push(format!("vault-{index}"));
                    })
                    .unwrap();
                })
            })
            .collect::<Vec<_>>();

        start.wait();
        for writer in writers {
            writer.join().unwrap();
        }

        let mut actual = Preferences::load_from(&path).recent_vaults;
        actual.sort();
        let mut expected = (0..WRITERS)
            .map(|index| format!("vault-{index}"))
            .collect::<Vec<_>>();
        expected.sort();
        assert_eq!(actual, expected);
        assert!(!path.with_extension("kagi-tmp").exists());

        let _ = std::fs::remove_dir_all(dir);
    }
}
