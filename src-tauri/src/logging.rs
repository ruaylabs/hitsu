use std::sync::OnceLock;

use tauri::Manager;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;

static FILE_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

/// Initialize leveled JSON logging. Production defaults to Hitsu warnings and
/// info while `RUST_LOG` can opt into debug details, which may include paths.
pub fn init(app: &tauri::AppHandle) {
    if let Err(error) = init_file_logging(app) {
        let fallback = tracing_subscriber::registry()
            .with(default_filter())
            .with(tracing_subscriber::fmt::layer().with_ansi(false));
        let _ = fallback.try_init();
        tracing::error!("file logging unavailable; using stderr");
        tracing::debug!(error = %error, "file logging initialization failure detail");
    }
}

fn init_file_logging(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = app.path().app_data_dir()?.join("logs");
    std::fs::create_dir_all(&log_dir)?;
    restrict_log_directory(&log_dir)?;

    let appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("hitsu")
        .filename_suffix("jsonl")
        .max_log_files(7)
        .build(&log_dir)?;
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let subscriber = tracing_subscriber::registry().with(default_filter()).with(
        tracing_subscriber::fmt::layer()
            .json()
            .with_ansi(false)
            .with_writer(writer),
    );
    subscriber.try_init()?;
    let _ = FILE_GUARD.set(guard);

    tracing::info!("file logging initialized");
    tracing::debug!(path = ?log_dir, "log directory configured");
    Ok(())
}

fn default_filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            EnvFilter::new("hitsu=debug,hitsu_lib=debug")
        } else {
            EnvFilter::new("hitsu=info,hitsu_lib=info")
        }
    })
}

#[cfg(unix)]
fn restrict_log_directory(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn restrict_log_directory(_path: &std::path::Path) -> std::io::Result<()> {
    Ok(())
}
