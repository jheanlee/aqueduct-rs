use crate::LOG_CONFIG;
use log::LevelFilter;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct LogConfig {
    pub stdout_filter: u8,
    pub system_filter: u8,
    pub stdout_enabled: bool,
    pub syslog_enabled: bool,
    pub oslog_enabled: bool,
}

#[derive(Debug)]
pub enum Error {
    UnsupportedPlatform,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnsupportedPlatform => write!(f, "unsupported platform"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Level {
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
    Trace,
    Always,
}

impl Level {
    fn as_u8(&self) -> u8 {
        match self {
            Level::Critical => 60,
            Level::Error => 50,
            Level::Warning => 40,
            Level::Notice => 30,
            Level::Info => 20,
            Level::Debug => 10,
            Level::Trace => 0,
            Level::Always => u8::MAX,
        }
    }
}

impl From<Level> for u8 {
    fn from(value: Level) -> Self {
        Level::as_u8(&value)
    }
}

mod color_code {
    pub const RESET: &str = "\x1b[0m";
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const FAINT_GRAY: &str = "\x1b[2;90m";
    pub const CYAN: &str = "\x1b[36m";
}

fn get_config(
    stdout_filter: u8,
    system_filter: u8,
    stdout_enabled: bool,
    system_logging_enabled: bool,
) -> LogConfig {
    LogConfig {
        stdout_filter,
        system_filter,
        stdout_enabled,
        syslog_enabled: system_logging_enabled && cfg!(target_os = "linux"),
        oslog_enabled: system_logging_enabled && cfg!(target_os = "macos"),
    }
}

pub fn init(
    verbose_level: u8,
    system_filter: u8,
    stdout_enabled: bool,
    system_logging_enabled: bool,
) -> Result<LogConfig, Error> {
    if cfg!(target_os = "macos") {
        #[cfg(target_os = "macos")]
        {
            let level_filter = match system_filter {
                0..=19 => LevelFilter::Trace,  //  OS_LOG_TYPE_DEBUG
                20..=29 => LevelFilter::Debug, //  OS_LOG_TYPE_INFO
                30..=49 => LevelFilter::Info,  //  OS_LOG_TYPE_DEFAULT
                50..=59 => LevelFilter::Warn,  //  OS_LOG_TYPE_ERROR
                60.. => LevelFilter::Error,    //  OS_LOG_TYPE_FAULT
            };
            oslog::OsLogger::new("cloud.silvergrass.rivulet")
                .level_filter(level_filter)
                .init()
                .expect("unable to initialise os_log");
        }

        let stdout_filter = match verbose_level {
            0 => 20,
            1 => 10,
            2.. => 0,
        };

        Ok(get_config(
            stdout_filter,
            system_filter,
            stdout_enabled,
            system_logging_enabled,
        ))
    } else if cfg!(target_os = "linux") {
        #[cfg(target_os = "linux")]
        {
            let level_filter = match system_filter {
                0..=9 => LevelFilter::Trace,
                10..=19 => LevelFilter::Debug,
                20..=29 => LevelFilter::Info,
                30..=49 => LevelFilter::Warn,
                50.. => LevelFilter::Error,
            };
            let formatter = syslog::Formatter3164 {
                facility: syslog::Facility::LOG_DAEMON,
                hostname: None,
                process: "rivulet".into(),
                pid: 0,
            };
            let logger = syslog::unix(formatter).expect("unable to connect to syslog");
            log::set_boxed_logger(Box::new(syslog::BasicLogger::new(logger)))
                .map(|()| log::set_max_level(level_filter))
                .expect("unable to register logger");
        }
        let stdout_filter = match verbose_level {
            0 => 20,
            1 => 10,
            2.. => 0,
        };

        Ok(get_config(
            stdout_filter,
            system_filter,
            stdout_enabled,
            system_logging_enabled,
        ))
    } else {
        Err(Error::UnsupportedPlatform)
    }
}

pub async fn log(level: Level, message: &str, category: &str) {
    let log_config = LOG_CONFIG.lock().await;

    //  stdout
    if log_config.stdout_enabled && level.as_u8() >= log_config.stdout_filter {
        let timestamp = chrono::Utc::now().format("%F %T").to_string();

        let level_code = match level {
            Level::Critical => color_code::RED.to_owned() + "[Critical]",
            Level::Error => color_code::RED.to_owned() + "[Error]",
            Level::Warning => color_code::YELLOW.to_owned() + "[Warning]",
            Level::Notice => "[Notice]".to_owned(),
            Level::Info => "[Info]".to_owned(),
            Level::Debug => color_code::FAINT_GRAY.to_owned() + "[Debug]",
            Level::Trace => color_code::FAINT_GRAY.to_owned() + "[Trace]",
            Level::Always => "".to_owned(),
        };

        let color_reset = color_code::RESET.to_owned();

        let console_message = format!(
            "({timestamp}) {level_code}{color_reset} {message} {}",
            if log_config.stdout_filter < Level::Info.as_u8() {
                format!("{}({category}){color_reset}", color_code::FAINT_GRAY)
            } else {
                String::new()
            }
        );
        println!("{console_message}");
    }

    //  macos os_log (only available if compiled with Apple clang)
    #[cfg(target_os = "macos")]
    if log_config.oslog_enabled && level.as_u8() != Level::Always.as_u8() {
        let log_level = match level.as_u8() {
            0..=19 => log::Level::Trace,  //  OS_LOG_TYPE_DEBUG
            20..=29 => log::Level::Debug, //  OS_LOG_TYPE_INFO
            30..=49 => log::Level::Info,  //  OS_LOG_TYPE_DEFAULT
            50..=59 => log::Level::Warn,  //  OS_LOG_TYPE_ERROR
            60.. => log::Level::Error,    //  OS_LOG_TYPE_FAULT
        };

        log::log!(
          target: category,
          log_level,
          "{message}"
        );
    }

    //  linux syslog
    #[cfg(target_os = "linux")]
    if log_config.syslog_enabled && level.as_u8() != Level::Always.as_u8() {
        let log_level = match level.as_u8() {
            0..=9 => log::Level::Trace,
            10..=19 => log::Level::Debug,
            20..=29 => log::Level::Info,
            30..=49 => log::Level::Warn,
            50.. => log::Level::Error,
        };

        log::log!(
          target: category,
          log_level,
          "{message}"
        );
    }
}
