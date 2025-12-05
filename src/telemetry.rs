use tracing::{info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling;
use tracing_appender::non_blocking;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use chrono::{Utc, NaiveDate};
use std::fs;
use std::io;
use tracing_subscriber::fmt::writer::MakeWriterExt;

static GUARD: OnceLock<non_blocking::WorkerGuard> = OnceLock::new();

fn split_path(path: &str) -> (PathBuf, String) {
    let p = Path::new(path);
    let dir = p.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
    let file = p.file_name().and_then(|s| s.to_str()).unwrap_or("mangabot.log").to_string();
    (dir, file)
}

pub fn init_telemetry(config: &crate::config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let (dir, file) = split_path(&config.server.log_path);
    fs::create_dir_all(&dir)?;

    let appender = rolling::daily(&dir, &file);
    let (non_blocking_appender, guard) = non_blocking(appender);
    let _ = GUARD.set(guard);

    let stdout = io::stdout.with_max_level(tracing::Level::TRACE);
    let fmt_layer = fmt::layer()
        .json()
        .with_writer(stdout.and(non_blocking_appender))
        .with_timer(fmt::time::SystemTime)
        .with_thread_names(true);

    let filter = EnvFilter::new(config.server.log_level.clone());

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    info!("Telemetry initialized");
    Ok(())
}