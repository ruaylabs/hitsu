use tauri::menu::{MenuBuilder, SubmenuBuilder};
use tauri::Emitter;

mod commands;
mod error;
mod models;
mod prefs;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let settings = tauri::menu::MenuItemBuilder::with_id("settings", "Settings…")
                .accelerator("CmdOrCtrl,+")
                .build(app)?;

            let app_menu = SubmenuBuilder::new(app, "Kagi")
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
            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id() == "settings" {
                let _ = app.emit("menu://settings", ());
            }
        })
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::vault::vault_open,
            commands::vault::vault_create,
            commands::vault::vault_change_password,
            commands::prefs::prefs_get,
            commands::prefs::prefs_set_last_vault,
            commands::entries::entries_list,
            commands::entries::entry_get,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
