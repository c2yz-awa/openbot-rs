mod config;
mod tui;

use crate::config::{AppConfig, AppType};
use chrono::Local;
use std::sync::{mpsc, OnceLock};
use tokio::fs;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

// config 宏
// pub static CONFIG: OnceLock<AppConfig> = OnceLock::new();
// #[macro_export]
// macro_rules! config {
//     () => {
//         $crate::CONFIG.get().unwrap()
//     };
// } 过时且诗山
struct Timer;
impl fmt::time::FormatTime for Timer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("[%Y-%m-%d %H:%M:%S]"))
    }
}

struct TuiWriter {
    tx: mpsc::Sender<String>,
}

impl std::io::Write for TuiWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = String::from_utf8_lossy(buf).to_string();
        let _ = self.tx.send(s.trim_end().to_string());
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
static CONFIG: OnceLock<AppConfig> = OnceLock::new();
// 获取配置 
pub fn config() -> &'static AppConfig {
    CONFIG.get().expect("Config isn't initialized.")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fs::create_dir_all("./logs").await?;

    let (tx, rx) = mpsc::channel();
    let log_name = format!("{}.log", Local::now().format("%Y-%m-%d-%H-%M-%S"));
    let file_appender = std::fs::File::create(format!("./logs/{}", log_name))?;
    let (writer, _guard) = tracing_appender::non_blocking(file_appender);

    let _config = AppConfig::load().await?;
    CONFIG.set(_config).ok();
    match config().app_type {
        AppType::CLI => {
            tracing_subscriber::registry()
                .with(EnvFilter::new("info"))
                .with(
                    fmt::layer()
                        .with_writer(move || TuiWriter { tx: tx.clone() })
                        .with_ansi(true)
                        .with_level(true)
                        .with_target(false)
                        .with_timer(Timer),
                )
                .with(
                    fmt::layer()
                        .with_writer(writer)
                        .with_ansi(false)
                        .with_timer(Timer),
                )
                .init();

            tracing::info!(
                "OpenBot {0} launching in CLI mode...",
                env!("CARGO_PKG_VERSION")
            );
            tracing::info!("Loaded config.");
            let terminal = ratatui::init();
            let app = tui::TuiApp::new();
            let res = app.run(terminal, rx);
            ratatui::restore();
            res?;
        }
        AppType::Daemon => {}
    }
    Ok(())
}
