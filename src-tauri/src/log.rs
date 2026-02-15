use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use tokio::fs;

use crate::config::log_dir;

/// 初始化日志输出
///
/// 使用 RUST_LOG 控制日志程度, 默认为 debug.
pub(crate) async fn init() -> crate::Result<WorkerGuard> {
    let log_dir = log_dir()?;
    if !log_dir.exists() {
        fs::create_dir(&log_dir).await.map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::Io,
                format!("failed to create log directory: {}", log_dir.display()),
                Box::new(e),
            )
        })?;
    }
    let file_appender = RollingFileAppender::builder()
        .filename_prefix("app")
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .build(log_dir)
        .map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::Io,
                "failed to init rolling file appender",
                Box::new(e),
            )
        })?;
    let (logging_appender, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(logging_appender)
        .with_ansi(false);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let stdout_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();
    Ok(guard)
}
