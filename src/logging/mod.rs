use colored::Colorize;
use indicatif::{MultiProgress, ProgressStyle};
use indicatif_log_bridge::LogWrapper;
use std::io::Write;

pub fn setup_logging() -> MultiProgress {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format(|buf, record| {
                let level = match record.level() {
                    log::Level::Error => format!("{:>5}", "ERROR").red().bold().to_string(),
                    log::Level::Warn => format!("{:>5}", "WARN").yellow().bold().to_string(),
                    log::Level::Info => format!("{:>5}", "INFO").green().bold().to_string(),
                    log::Level::Debug => format!("{:>5}", "DEBUG").cyan().bold().to_string(),
                    log::Level::Trace => format!("{:>5}", "TRACE").magenta().bold().to_string(),
                };
                writeln!(buf, "{} -> {}", level, record.args())
            })
            .build();
    let level = logger.filter();
    let multi_progress = MultiProgress::new();
    LogWrapper::new(multi_progress.clone(), logger)
        .try_init()
        .unwrap();
    log::set_max_level(level);
    multi_progress
}

pub fn progress_bar_style() -> ProgressStyle {
    ProgressStyle::with_template("{bar:40} {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("━━╸")
}
