//! OS session-lock monitoring.
//!
//! Linux watches systemd-logind's `LockedHint`; macOS observes both workspace
//! session resignation and the distributed screen-lock notification. In both cases the backend
//! drops decrypted vault state immediately, then tells the webview to switch
//! to its locked UI.

#[cfg(target_os = "linux")]
mod linux {
    use tauri::{AppHandle, Emitter, Manager};
    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::OwnedObjectPath;

    use crate::state::AppState;

    pub fn start(app: AppHandle) {
        std::thread::Builder::new()
            .name("session-lock-monitor".into())
            .spawn(move || {
                if let Err(error) = monitor(&app) {
                    tracing::warn!("session lock monitor stopped");
                    tracing::debug!(error = %error, "session lock monitor failure detail");
                }
            })
            .expect("failed to start session-lock monitor");
    }

    fn monitor(app: &AppHandle) -> zbus::Result<()> {
        let connection = Connection::system()?;
        let manager = Proxy::new(
            &connection,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )?;
        let session_path: OwnedObjectPath =
            manager.call("GetSessionByPID", &(std::process::id()))?;
        let session = Proxy::new(
            &connection,
            "org.freedesktop.login1",
            session_path,
            "org.freedesktop.login1.Session",
        )?;

        for changed in session.receive_property_changed::<bool>("LockedHint") {
            if changed.get().unwrap_or(false) {
                let state = app.state::<AppState>();
                crate::commands::vault::lock_open_vaults(&state);
                let _ = app.emit("vault://session-locked", ());
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub use linux::start;

#[cfg(target_os = "macos")]
mod macos {
    use std::ptr::NonNull;

    use objc2_app_kit::{NSWorkspace, NSWorkspaceSessionDidResignActiveNotification};
    use objc2_foundation::{NSDistributedNotificationCenter, NSNotification, NSString};
    use tauri::{AppHandle, Emitter, Manager};

    use crate::state::AppState;

    fn lock(app: &AppHandle) {
        let state = app.state::<AppState>();
        crate::commands::vault::lock_open_vaults(&state);
        let _ = app.emit("vault://session-locked", ());
    }

    pub fn start(app: AppHandle) {
        // Notification registration must happen on the main thread; Tauri's
        // setup callback (our caller) satisfies that requirement.
        let workspace_center = NSWorkspace::sharedWorkspace().notificationCenter();
        let session_app = app.clone();
        let session_block = block2::RcBlock::new(move |_notification: NonNull<NSNotification>| {
            lock(&session_app);
        });

        // NSWorkspace's session notification covers fast-user switching and
        // session resignation, but macOS screen locking uses a distributed
        // `com.apple.screenIsLocked` notification instead.
        let distributed_center = NSDistributedNotificationCenter::defaultCenter();
        let screen_lock_name = NSString::from_str("com.apple.screenIsLocked");
        let screen_block = block2::RcBlock::new(move |_notification: NonNull<NSNotification>| {
            lock(&app);
        });

        // SAFETY: both notification names are valid NSString values, object
        // filters are intentionally unrestricted, no operation queue is used,
        // and AppHandle is Send + Sync. The centers copy and retain the blocks.
        // Keep both opaque tokens alive because these monitors are process-long.
        let session_observer = unsafe {
            workspace_center.addObserverForName_object_queue_usingBlock(
                Some(NSWorkspaceSessionDidResignActiveNotification),
                None,
                None,
                &session_block,
            )
        };
        let screen_observer = unsafe {
            distributed_center.addObserverForName_object_queue_usingBlock(
                Some(&screen_lock_name),
                None,
                None,
                &screen_block,
            )
        };
        std::mem::forget(session_observer);
        std::mem::forget(screen_observer);
    }
}

#[cfg(target_os = "macos")]
pub use macos::start;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn start(_app: tauri::AppHandle) {
    // The existing visibility/sleep monitor remains the fallback where no
    // native session-lock integration is installed yet.
}
