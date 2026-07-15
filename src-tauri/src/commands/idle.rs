/// Lightweight heartbeat sent when the webview observes user input. The
/// invoke-handler middleware refreshes the backend watchdog before this runs.
#[tauri::command]
pub async fn idle_activity() {}
