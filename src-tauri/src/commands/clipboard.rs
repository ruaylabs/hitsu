use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use parking_lot::Mutex;
use zeroize::Zeroizing;

use crate::error::KagiResult;

/// Monotonic token for clipboard writes performed by this app.
///
/// Every successful app-side clipboard write receives a generation. Delayed
/// auto-clear tasks only clear when their generation is still current, so a
/// later copy of the same secret is not cleared by an older timer.
static CLIPBOARD_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Serializes app-side clipboard writes with delayed clear checks. Without
/// this, an old timer could observe the clipboard after a newer write but
/// before that write records its new generation.
static CLIPBOARD_LOCK: Mutex<()> = Mutex::new(());

fn mark_clipboard_write() -> u64 {
    CLIPBOARD_GENERATION.fetch_add(1, Ordering::SeqCst) + 1
}

fn should_clear_secret(
    timer_generation: u64,
    current_clipboard: Option<&str>,
    secret: &str,
) -> bool {
    CLIPBOARD_GENERATION.load(Ordering::SeqCst) == timer_generation
        && current_clipboard == Some(secret)
}

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
    let _guard = CLIPBOARD_LOCK.lock();
    set_clipboard(&value)?;
    mark_clipboard_write();
    Ok(())
}

/// Copy a secret with exclusion hints and (when `timeout_secs > 0`) a
/// spawned auto-clear task. Shared by the IPC command below and
/// `entry_copy_field`, whose values are read backend-side and never cross IPC.
pub(crate) fn copy_secret(secret: Zeroizing<String>, timeout_secs: u64) -> KagiResult<()> {
    let generation = {
        let _guard = CLIPBOARD_LOCK.lock();
        set_clipboard(&secret)?;
        mark_clipboard_write()
    };

    if timeout_secs > 0 {
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(timeout_secs)).await;

            let _guard = CLIPBOARD_LOCK.lock();
            if let Ok(mut cb) = arboard::Clipboard::new() {
                // Only clear if this timer is still the latest app-side write
                // and the clipboard still contains our secret. If the user
                // copied something else in the meantime, leave it alone so we
                // don't clobber their later copy.
                let current = cb.get_text().ok();
                if should_clear_secret(generation, current.as_deref(), secret.as_str()) {
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
    let _guard = CLIPBOARD_LOCK.lock();
    if let Ok(mut cb) = arboard::Clipboard::new() {
        if cb.clear().is_ok() {
            mark_clipboard_write();
        }
    }
}

/// Async clipboard clear with proper error reporting for frontend IPC.
#[tauri::command]
pub async fn clipboard_clear() -> KagiResult<()> {
    let _guard = CLIPBOARD_LOCK.lock();
    let mut cb = arboard::Clipboard::new()
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;
    cb.clear()
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;
    mark_clipboard_write();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_timer_does_not_clear_newer_copy_of_same_secret() {
        let _guard = CLIPBOARD_LOCK.lock();

        let old_timer = mark_clipboard_write();
        let _newer_copy = mark_clipboard_write();

        assert!(!should_clear_secret(
            old_timer,
            Some("same-secret"),
            "same-secret"
        ));
    }

    #[test]
    fn current_timer_clears_matching_secret() {
        let _guard = CLIPBOARD_LOCK.lock();

        let timer = mark_clipboard_write();

        assert!(should_clear_secret(timer, Some("secret"), "secret"));
    }

    #[test]
    fn current_timer_leaves_user_replaced_clipboard_alone() {
        let _guard = CLIPBOARD_LOCK.lock();

        let timer = mark_clipboard_write();

        assert!(!should_clear_secret(timer, Some("user-copy"), "secret"));
    }
}
