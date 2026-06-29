use std::time::Duration;

use crate::error::KagiResult;

#[tauri::command]
pub async fn clipboard_copy(value: String) -> KagiResult<()> {
    let mut cb = arboard::Clipboard::new()
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;
    cb.set_text(value)
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn clipboard_copy_with_timeout(value: String, timeout_secs: u64) -> KagiResult<()> {
    let mut cb = arboard::Clipboard::new()
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;

    cb.set_text(&value)
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;
    drop(cb);

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(timeout_secs)).await;
        if let Ok(mut cb) = arboard::Clipboard::new() {
            // Only clear if the clipboard still contains our secret.
            // If the user copied something else in the meantime, leave it alone
            // so we don't clobber their later copy.
            let current = cb.get_text().ok();
            if current.as_deref() == Some(&value) {
                let _ = cb.clear();
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn clipboard_clear() -> KagiResult<()> {
    let mut cb = arboard::Clipboard::new()
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;
    cb.clear()
        .map_err(|e| crate::error::KagiError::Custom(format!("Clipboard error: {}", e)))?;
    Ok(())
}
