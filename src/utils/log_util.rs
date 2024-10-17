use std::{env, fs};
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::exit;
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use {
    lazy_static::lazy_static,
    std::sync::{Arc, RwLock},
};

lazy_static! {
    static ref LOGGER: Arc<RwLock<env_logger::Logger>> =
        Arc::new(RwLock::new(env_logger::Logger::from_default_env()));
}

struct LoggerShim {}

impl log::Log for LoggerShim {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        LOGGER.read().unwrap().enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        LOGGER.read().unwrap().log(record);
    }

    fn flush(&self) {}
}

fn replace_logger(logger: env_logger::Logger) {
    log::set_max_level(logger.filter());
    *LOGGER.write().unwrap() = logger;
    let _ = log::set_boxed_logger(Box::new(LoggerShim {}));
}

// Configures logging with a specific filter overriding RUST_LOG.  _RUST_LOG is used instead
// so if set it takes precedence.
// May be called at any time to re-configure the log filter
pub fn setup_with(filter: &str) {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::new().filter_or("_RUST_LOG", filter))
            .format_timestamp_nanos()
            .build();
    replace_logger(logger);
}

// Configures logging with a default filter if RUST_LOG is not set
pub fn setup_with_default(filter: &str) {
    let logger = env_logger::Builder::from_env(env_logger::Env::new().default_filter_or(filter))
        .format_timestamp_nanos()
        .build();
    replace_logger(logger);
}

// Configures logging with the default filter "error" if RUST_LOG is not set
pub fn setup() {
    setup_with_default("error");
}

// Configures file logging with a default filter if RUST_LOG is not set
//
// NOTE: This does not work at the moment, pending the resolution of https://github.com/env-logger-rs/env_logger/issues/208
pub fn setup_file_with_default(logfile: &str, filter: &str) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(logfile)
        .unwrap();
    let logger = env_logger::Builder::from_env(env_logger::Env::new().default_filter_or(filter))
        .format_timestamp_nanos()
        .target(env_logger::Target::Pipe(Box::new(file)))
        .build();
    replace_logger(logger);
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogOutput {
    None,
    Log,
}


pub fn init_logger(output: LogOutput) {
    let mut log_dir = PathBuf::new();
    log_dir.push("log");
    if !log_dir.exists() {
        fs::create_dir(&log_dir).unwrap_or_else(|err| {
            println!(
                "Error: Unable to create directory {}: {}",
                log_dir.display(),
                err
            );
            exit(1);
        });
    }

    let log_symlink = log_dir.join("fraud-proof.log");
    let logfile = if output != LogOutput::Log {
        let log_with_timestamp = format!(
            "fraud-proof-{}.log",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        let _ = fs::remove_file(&log_symlink);
        symlink::symlink_file(&log_with_timestamp, &log_symlink).unwrap();

        Some(
            log_dir
                .join(log_with_timestamp)
                .into_os_string()
                .into_string()
                .unwrap(),
        )
    } else {
        None
    };
    let _logger_thread = redirect_stderr_to_file(logfile);
}

fn redirect_stderr(filename: &str) {
    use std::os::unix::io::AsRawFd;
    match OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(filename)
    {
        Ok(file) => unsafe {
            libc::dup2(file.as_raw_fd(), libc::STDERR_FILENO);
        },
        Err(err) => eprintln!("Unable to open {}: {}", filename, err),
    }
}

// Redirect stderr to a file with support for logrotate by sending a SIGUSR1 to the process.
//
// Upon success, future `log` macros and `eprintln!()` can be found in the specified log file.
pub fn redirect_stderr_to_file(logfile: Option<String>) -> Option<JoinHandle<()>> {
    // Default to RUST_BACKTRACE=1 for more informative validator logs
    if env::var_os("RUST_BACKTRACE").is_none() {
        env::set_var("RUST_BACKTRACE", "1")
    }

    let filter = "info";
    match logfile {
        None => {
            setup_with_default(filter);
            None
        }
        Some(log_file) => {
            {
                use log::info;
                let mut signals =
                    signal_hook::iterator::Signals::new([signal_hook::consts::SIGUSR1])
                        .unwrap_or_else(|err| {
                            eprintln!("Unable to register SIGUSR1 handler: {:?}", err);
                            exit(1);
                        });

                setup_with_default(filter);
                redirect_stderr(log_file.as_str());
                Some(
                    std::thread::Builder::new()
                        .name(String::from("solSigUsr1"))
                        .spawn(move || {
                            for signal in signals.forever() {
                                info!(
                                    "received SIGUSR1 ({}), reopening log file: {:?}",
                                    signal, log_file
                                );
                                redirect_stderr(log_file.as_str());
                            }
                        })
                        .unwrap(),
                )
            }
        }
    }
}

