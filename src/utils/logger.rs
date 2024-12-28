use anyhow::Result;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::env;

pub fn setup_logging(log_file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_level = match env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase()
        .as_str()
    {
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    let size_limit = 5 * 1024 * 1024;
    let pattern = format!("{}.{{}}.gz", log_file_path);

    let policy = CompoundPolicy::new(
        Box::new(SizeTrigger::new(size_limit)),
        Box::new(FixedWindowRoller::builder().build(&pattern, 3)?),
    );

    // ログファイル用のAppender
    let logfile = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%dT%H:%M:%S.%3f)} {h({l:5.5})}][{M:10.10}:{line:3.3}] {m}{n}",
        )))
        .build(log_file_path, Box::new(policy))?;

    // 標準出力用のAppender
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%dT%H:%M:%S.%3f)} {h({l:5.5})}][{M:10.10}:{line:3.3}] {m}{n}",
        )))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(log_level),
        )?;

    log4rs::init_config(config)?;
    Ok(())
}
