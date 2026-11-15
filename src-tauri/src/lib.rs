use tauri::menu::{MenuBuilder, SubmenuBuilder};
use tauri::{Emitter, Manager};

mod auto_lock;
mod biometric;
#[cfg(unix)]
pub mod browser_ipc;
pub mod prefs;
pub mod state;

pub mod commands;
mod error;
mod hardening;
mod kdbx_fields;
mod logging;
pub mod models;
#[cfg(target_os = "linux")]
mod native_menu;
mod session_lock;
mod vault;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let command_handler: Box<tauri::ipc::InvokeHandler<tauri::Wry>> =
        Box::new(tauri::generate_handler![
            commands::biometric::biometric_status,
            commands::biometric::biometric_enable,
            commands::biometric::biometric_disable,
            commands::biometric::biometric_unlock,
            commands::vault::vault_open,
            commands::vault::vault_create,
            commands::vault::vault_change_password,
            commands::vault::vault_lock,
            commands::vault::vault_refresh_if_changed,
            commands::vault::vault_empty_recycle_bin,
            commands::vault::vault_upgrade_kdf,
            commands::import::vault_import_1pif,
            commands::import::import_report_export,
            commands::prefs::prefs_get,
            commands::prefs::prefs_set_last_vault,
            commands::prefs::prefs_set_security,
            commands::prefs::prefs_set_theme,
            commands::prefs::prefs_set_folders_enabled,
            commands::prefs::prefs_set_browser_integration_enabled,
            commands::prefs::prefs_set_kdf_dismissed,
            commands::entries::entry_get,
            commands::entries::entries_search,
            commands::entries::entries_health_report,
            commands::entries::entry_edit_payload,
            commands::entries::entry_create,
            commands::entries::entry_update,
            commands::entries::entry_convert_to_login,
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
            commands::entries::entry_download_favicon,
            commands::entries::entries_download_favicons,
            commands::entries::entry_get_custom_icon,
            commands::entries::tag_rename,
            commands::entries::tag_delete,
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
            let preferences = prefs::Preferences::load(app.handle());
            let window_config = &app.config().app.windows[0];
            // Create hidden so the native background is correct before the first frame.
            let theme = match preferences.theme {
                prefs::ThemePreference::System => None,
                prefs::ThemePreference::Light => Some(tauri::Theme::Light),
                prefs::ThemePreference::Dark => Some(tauri::Theme::Dark),
            };
            let saved_theme = serde_json::to_string(&preferences.theme)?;
            let window = tauri::WebviewWindowBuilder::from_config(app, window_config)?
                .visible(false)
                .initialization_script(format!(
                    r#"(() => {{
                        const theme = {saved_theme};
                        if (theme === "system") return;
                        const apply = () => {{
                            if (!document.documentElement) return false;
                            document.documentElement.dataset.theme = theme;
                            return true;
                        }};
                        if (!apply()) {{
                            const observer = new MutationObserver(() => {{
                                if (apply()) observer.disconnect();
                            }});
                            observer.observe(document, {{ childList: true }});
                        }}
                    }})();"#
                ))
                .build()?;
            let dark = theme.unwrap_or(window.theme()?) == tauri::Theme::Dark;
            let color = if dark { (32, 31, 29, 255) } else { (239, 237, 229, 255) };
            window.set_background_color(Some(tauri::window::Color::from(color)))?;
            if window_config.visible {
                window.show()?;
            }

            let settings = tauri::menu::MenuItemBuilder::with_id("settings", "Settings…")
                .accelerator("CmdOrCtrl,+")
                .build(app)?;

            let new_item = tauri::menu::MenuItemBuilder::with_id("new-item", "New Item…")
                .accelerator("CmdOrCtrl+N")
                .build(app)?;
            let lock_vault = tauri::menu::MenuItemBuilder::with_id("lock-vault", "Lock Vault")
                .accelerator("CmdOrCtrl+L")
                .build(app)?;
            let find = tauri::menu::MenuItemBuilder::with_id("find", "Find")
                .accelerator("CmdOrCtrl+F")
                .build(app)?;

            let app_menu = SubmenuBuilder::new(app, "Hitsu")
                .about(None)
                .separator()
                .item(&settings)
                .separator();
            #[cfg(target_os = "macos")]
            let app_menu = app_menu.hide().hide_others().show_all().separator();
            let app_menu = app_menu.quit().build()?;

            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&new_item)
                .item(&lock_vault)
                .separator()
                .close_window()
                .build()?;

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .separator()
                .item(&find)
                .build()?;

            let window_menu = SubmenuBuilder::new(app, "Window")
                .minimize()
                .maximize()
                .build()?;
            let help_menu = SubmenuBuilder::new(app, "Help")
                .text("keyboard-shortcuts", "Keyboard Shortcuts")
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&app_menu)
                .item(&file_menu)
                .item(&edit_menu)
                .item(&window_menu)
                .item(&help_menu)
                .build()?;

            app.set_menu(menu)?;

            #[cfg(target_os = "linux")]
            {
                native_menu::install_theme_css()?;
                native_menu::apply_theme(app.handle(), preferences.theme);
            }
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
            let event_name = match event.id().as_ref() {
                "settings" => "menu://settings",
                "new-item" => "menu://new-item",
                "lock-vault" => "menu://lock-vault",
                "find" => "menu://find",
                "keyboard-shortcuts" => "menu://keyboard-shortcuts",
                _ => return,
            };
            let _ = app.emit(event_name, ());
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
