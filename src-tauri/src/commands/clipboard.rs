use std::time::Duration;

use zeroize::Zeroizing;

use crate::error::KagiResult;

/// Shared helper: copy `value` to the system clipboard with platform-specific
/// exclusion hints so clipboard managers / history / cloud sync don't capture
/// the secret.
fn set_clipboard(value: &str) -> KagiResult<()> {
    let mut cb = arboard::Clipboard::new()
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;

    let mut set = cb.set();

    #[cfg(windows)]
    {
        use arboard::SetExtWindows;
        set = set
            .exclude_from_monitoring()
            .exclude_from_cloud()
            .exclude_from_history();
    }

    #[cfg(target_os = "macos")]
    {
        use arboard::SetExtApple;
        set = set.exclude_from_history();
    }

    // Unix (non-macOS): excludes from history via x-kde-passwordManagerHint
    // MIME type on Wayland — honored by Klipper, GNOME Clipboard History,
    // and others. Not universal, but the best available signal.
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        use arboard::SetExtLinux;
        set = set.exclude_from_history();
    }

    set.text(value)
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub async fn clipboard_copy(value: String) -> KagiResult<()> {
    set_clipboard(&value)
}

/// Copy a secret with exclusion hints and (when `timeout_secs > 0`) a
/// spawned auto-clear task. Shared by the IPC command below and
/// `entry_copy_field`, whose values are read backend-side and never cross IPC.
pub(crate) fn copy_secret(secret: Zeroizing<String>, timeout_secs: u64) -> KagiResult<()> {
    set_clipboard(&secret)?;

    if timeout_secs > 0 {
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(timeout_secs)).await;
            if let Ok(mut cb) = arboard::Clipboard::new() {
                // Only clear if the clipboard still contains our secret.
                // If the user copied something else in the meantime, leave it alone
                // so we don't clobber their later copy.
                let current = cb.get_text().ok();
                if current.as_deref() == Some(secret.as_str()) {
                    let _ = cb.clear();
                }
            }
            // secret is dropped and zeroized here
        });
    }

    Ok(())
}

#[tauri::command]
pub async fn clipboard_copy_with_timeout(value: String, timeout_secs: u64) -> KagiResult<()> {
    copy_secret(Zeroizing::new(value), timeout_secs)
}

/// Synchronous clipboard clear — usable from sync contexts (exit handler, …).
/// Swallows errors: when the app is shutting down there's nothing to report to.
pub fn clear_clipboard_sync() {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.clear();
    }
}

/// Async clipboard clear with proper error reporting for frontend IPC.
#[tauri::command]
pub async fn clipboard_clear() -> KagiResult<()> {
    let mut cb = arboard::Clipboard::new()
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;
    cb.clear()
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;
    Ok(())
}
