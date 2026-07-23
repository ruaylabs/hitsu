use tauri::menu::{MenuBuilder, SubmenuBuilder};
use tauri::{Emitter, Manager};

mod auto_lock;
#[cfg(unix)]
pub mod browser_ipc;
pub mod prefs;
pub mod state;

pub mod commands;
mod error;
mod hardening;
mod logging;
pub mod models;
mod session_lock;
mod vault;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let command_handler: Box<tauri::ipc::InvokeHandler<tauri::Wry>> =
        Box::new(tauri::generate_handler![
            commands::vault::vault_open,
            commands::vault::vault_create,
            commands::vault::vault_change_password,
            commands::vault::vault_lock,
            commands::vault::vault_refresh_if_changed,
            commands::vault::vault_empty_recycle_bin,
            commands::vault::vault_upgrade_kdf,
            commands::import::vault_import_1pif,
            commands::prefs::prefs_get,
            commands::prefs::prefs_set_last_vault,
            commands::prefs::prefs_set_security,
            commands::prefs::prefs_set_folders_enabled,
            commands::prefs::prefs_set_browser_integration_enabled,
            commands::prefs::prefs_set_kdf_dismissed,
            commands::entries::entry_get,
            commands::entries::entries_search,
            commands::entries::entry_edit_payload,
            commands::entries::entry_create,
            commands::entries::entry_update,
            commands::entries::entry_move,
            commands::entries::folder_create,
            commands::entries::folder_rename,
            commands::entries::entry_delete,
            commands::entries::entry_restore,
            commands::entries::entry_delete_permanent,
            commands::entries::entry_discard,
            commands::entries::entry_reveal_field,
            commands::entries::entry_copy_field,
            commands::entries::entry_reveal_custom_field,
            commands::entries::entry_copy_custom_field,
            commands::entries::entry_history_list,
            commands::entries::entry_history_get,
            commands::entries::entry_attachment_save,
            commands::entries::entry_attachment_add,
            commands::entries::entry_attachment_remove,
            commands::clipboard::clipboard_copy,
            commands::clipboard::clipboard_copy_with_timeout,
            commands::clipboard::clipboard_clear,
            commands::generator::generate_password,
            commands::idle::idle_activity,
            commands::totp::totp_compute,
        ]);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            logging::init(app.handle());
            hardening::apply();
            prefs::Preferences::migrate_legacy(app.handle())?;

            let settings = tauri::menu::MenuItemBuilder::with_id("settings", "Settings…")
                .accelerator("CmdOrCtrl,+")
                .build(app)?;

            let app_menu = SubmenuBuilder::new(app, "Hitsu")
                .about(None)
                .separator()
                .item(&settings)
                .separator()
                .quit()
                .build()?;

            let file_menu = SubmenuBuilder::new(app, "File").close_window().build()?;

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;

            let window_menu = SubmenuBuilder::new(app, "Window").minimize().build()?;

            let menu = MenuBuilder::new(app)
                .item(&app_menu)
                .item(&file_menu)
                .item(&edit_menu)
                .item(&window_menu)
                .build()?;

            app.set_menu(menu)?;

            let preferences = prefs::Preferences::load(app.handle());
            let state = app.state::<AppState>();
            state.configure_idle_lock(preferences.idle_lock_minutes);
            auto_lock::start(app.handle().clone());
            session_lock::start(app.handle().clone());
            // Browser integration is opt-in (developer preview); the managed
            // handle lets the Settings toggle start/stop it without a restart.
            // Native-host registration happens inside set_enabled, so nothing
            // is advertised to browsers until the user opts in.
            #[cfg(unix)]
            {
                app.manage(browser_ipc::BrowserIpc(parking_lot::Mutex::new(None)));
                if preferences.browser_integration_enabled {
                    if let Err(error) = browser_ipc::set_enabled(app.handle(), true) {
                        tracing::warn!("browser integration unavailable at startup");
                        tracing::debug!(error = %error, "browser integration startup failure detail");
                    }
                }
            }
            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id() == "settings" {
                let _ = app.emit("menu://settings", ());
            }
        })
        .manage(AppState::new())
        .invoke_handler(move |invoke: tauri::ipc::Invoke<tauri::Wry>| {
            // Refresh before dispatch so every frontend command counts as
            // backend activity, including commands that ultimately fail.
            invoke
                .message
                .webview_ref()
                .state::<AppState>()
                .reset_idle_lock();
            command_handler(invoke)
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_handle, event| {
        if let tauri::RunEvent::Exit = event {
            // Clear clipboard on exit so secrets don't linger after the app quits.
            commands::clipboard::clear_clipboard_sync();
        }
    });
}
