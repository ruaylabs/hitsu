use std::sync::OnceLock;

use gtk::prelude::*;

use crate::prefs::ThemePreference;

/// Keep the native menu readable in both active and backdrop states without
/// hardcoding a dark palette or changing popup-menu/selection styling.
pub fn install_theme_css() -> Result<(), gtk::glib::Error> {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        b"menubar {
            background-image: none;
            background-color: @theme_bg_color;
        }
        menubar > menuitem > label {
            color: @theme_fg_color;
        }",
    )?;
    if let Some(screen) = gtk::gdk::Screen::default() {
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    Ok(())
}

pub fn apply_theme(app: &tauri::AppHandle, preference: ThemePreference) {
    // Capture GTK's detected preference before applying any explicit override.
    // On Linux, set_theme(None) disables dark mode instead of restoring it.
    static SYSTEM_THEME: OnceLock<tauri::Theme> = OnceLock::new();
    let system_theme = *SYSTEM_THEME.get_or_init(|| {
        if gtk::Settings::default()
            .is_some_and(|settings| settings.is_gtk_application_prefer_dark_theme())
        {
            tauri::Theme::Dark
        } else {
            tauri::Theme::Light
        }
    });

    app.set_theme(match preference {
        ThemePreference::System => Some(system_theme),
        ThemePreference::Light => Some(tauri::Theme::Light),
        ThemePreference::Dark => Some(tauri::Theme::Dark),
    });
}
