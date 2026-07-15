//! Backend idle-lock watchdog.
//!
//! The webview has its own user-activity timer for responsive locking. This
//! watchdog is an independent backstop: every backend IPC command refreshes
//! its deadline, and the decrypted vault is dropped even if the webview hangs.

use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

pub fn start(app: AppHandle) {
    std::thread::Builder::new()
        .name("idle-lock-watchdog".into())
        .spawn(move || loop {
            let state = app.state::<AppState>();
            state.wait_for_idle_timeout_and_lock();

            // Match every other lock path by clearing copied secrets too.
            crate::commands::clipboard::clear_clipboard_sync();
            let _ = app.emit("vault://idle-locked", ());
        })
        .expect("failed to start idle-lock watchdog");
}
