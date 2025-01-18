use super::configuration::AppConfigItem;
use std::fs::{self, DirBuilder};
use std::{env, panic, path::PathBuf};
use time::{macros::format_description, UtcOffset};
use tracing::{self, error};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*};

const LOG_PREFIX: &str = "tts_server.log";

pub fn get_abs_path(path: String) -> PathBuf {
    let tgt_path = if PathBuf::from(path.clone()).is_absolute() {
        let tmp_path = PathBuf::from(path);
        if !tmp_path.exists() {
            DirBuilder::new()
                .recursive(true)
                .create(tmp_path.clone())
                .unwrap();
        }
        tmp_path
    } else {
        let mut cur_path = std::env::current_dir().unwrap();
        cur_path.push(path);
        if !cur_path.exists() {
            DirBuilder::new()
                .recursive(true)
                .create(cur_path.clone())
                .unwrap();
        }
        let abs_log_path = fs::canonicalize(&cur_path);
        abs_log_path.unwrap()
    };
    tgt_path
}

pub fn get_workspace_path() -> PathBuf {
    let workspace_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    workspace_path.parent().unwrap().to_path_buf()
}

pub fn get_default_log_path() -> anyhow::Result<PathBuf> {
    let log_path = get_workspace_path().join("logs");
    if !log_path.exists() {
        std::fs::create_dir(&log_path)?;
    }
    anyhow::Ok(log_path)
}

pub fn init(config_data: &AppConfigItem) -> anyhow::Result<Vec<WorkerGuard>> {
    let log_path = get_abs_path(config_data.log_path.clone().unwrap());
    let file_appender = tracing_appender::rolling::daily(log_path.clone(), LOG_PREFIX);
    let (filelog, file_log_guard) = tracing_appender::non_blocking(file_appender);
    let (stdoutlog, std_out_guard) = tracing_appender::non_blocking(std::io::stdout());
    let local_time = tracing_subscriber::fmt::time::OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).unwrap(),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]"),
    );

    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(stdoutlog.with_max_level(tracing::Level::DEBUG))
                .with_timer(local_time.clone())
                .with_ansi(true)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .pretty(),
        )
        .with(
            fmt::Layer::new()
                .with_writer(filelog.with_max_level(tracing::Level::INFO))
                .with_timer(local_time.clone())
                .with_ansi(false)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true),
        );
    tracing::subscriber::set_global_default(subscriber).unwrap();
    Ok(vec![file_log_guard, std_out_guard])
}

pub fn init_panic() {
    panic::set_hook(Box::new(|panic_info| {
        // 在 panic 发生时记录日志
        error!("Panic occurred: {:?}", panic_info);
    }));
}
