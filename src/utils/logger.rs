use anyhow::Result;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::{Encode, Write};
use std::env;

#[derive(Debug)]
struct CustomEncoder {
    pattern: String,
}

impl CustomEncoder {
    fn new(pattern: &str) -> Self {
        CustomEncoder {
            pattern: pattern.to_string(),
        }
    }
}

impl Encode for CustomEncoder {
    fn encode(&self, w: &mut dyn Write, record: &log::Record) -> Result<()> {
        let file_name = record
            .file()
            .map(|path| {
                // フルパスからファイル名を抽出し、15文字固定（左寄せ）
                let file = path.rsplit('/').next().unwrap_or(path);
                format!("{:<15.15}", file)
            })
            .unwrap_or_else(|| format!("{:<15.15}", "unknown"));

        let line = record
            .line()
            .map_or("   ".to_string(), |line| format!("{:>3}", line)); // 3桁固定（右寄せ）

        let mut output = self.pattern.clone();
        output = output.replace("{file_name}", &file_name);
        output = output.replace("{line}", &line);
        output = output.replace("{message}", &record.args().to_string());
        output = output.replace("{level}", &format!("{:<5}", record.level().to_string()));
        output = output.replace(
            "{time}",
            &chrono::Local::now()
                .format("%Y-%m-%dT%H:%M:%S%.3f")
                .to_string(),
        );

        w.write_all(output.as_bytes())?;
        Ok(())
    }
}

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
        .encoder(Box::new(CustomEncoder::new(
            "[{time} {level}][{file_name}:{line}] {message}{n}",
        )))
        .build(log_file_path, Box::new(policy))?;

    // 標準出力用のAppender
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(CustomEncoder::new(
            "[{time} {level}][{file_name}:{line}] {message}{n}",
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
