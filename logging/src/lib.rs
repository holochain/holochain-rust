//! Holochain's log implementation.
//!
//! This logger implementation is designed to be fast and to provide useful filtering combination capabilities.
//!
//! # Use
//!
//! The basic use of the log crate is through the five logging macros: [`error!`],
//! [`warn!`], [`info!`], [`debug!`] and [`trace!`]
//! where `error!` represents the highest-priority log messages
//! and `trace!` the lowest. The log messages are filtered by configuring
//! the log level to exclude messages with a lower priority.
//! Each of these macros accept format strings similarly to [`println!`].
//!
//! [`error!`]: ../log/macro.error.html
//! [`warn!`]: ../log/macro.warn.html
//! [`info!`]: ../log/macro.info.html
//! [`debug!`]: ..log/macro.debug.html
//! [`trace!`]: ../log/macro.trace.html
//! [`println!`]: https://doc.rust-lang.org/stable/std/macro.println.html
//!
//! # Quick Start
//!
//! To get you started quickly, the easiest and highest-level way to get a working logger (with the
//! [`Info`](Level::Info) log verbosity level) is to use [`init_simple`].
//!
//! ```edition2018
//! use logging::prelude::*;
//!
//! logging::init_simple().unwrap();
//! info!("Here you go champ!");
//!
//! // Warning and Error log message have their own color
//! warn!("You've been warned Sir!");
//! error!("Oh... something wrong pal.");
//! ```
//!
//! ### Examples
//!
//! #### Simple log with Trace verbosity level.
//!
//! In order to log everything with at least the [`Debug`](Level::Debug) verbosity level:
//!
//! ```edition2018
//! use logging::prelude::*;
//!
//! FastLoggerBuilder::new()
//!     // The timestamp format is customizable as well
//!     .timestamp_format("%Y-%m-%d %H:%M:%S%.6f")
//!     .set_level_from_str("Debug")
//!     .build()
//!     .expect("Fail to init the logging factory.");
//!
//! debug!("Let's trace what that program is doing.");
//! ```
//!
//! #### Building the logging factory from TOML configuration.
//!
//! The logger can be built from a [TOML](https://github.com/toml-lang/toml) configuration file:
//!
//! ```edition2018
//! use logging::prelude::*;
//! let toml = r#"
//!    [logger]
//!    level = "debug"
//!
//!         [[logger.rules]]
//!         pattern = "info"
//!         exclude = false
//!         color = "Blue"
//!     "#;
//!
//! FastLoggerBuilder::from_toml(toml)
//!     .expect("Fail to instantiate the logger from toml.")
//!      .build()
//!      .expect("Fail to build logger from toml.");
//!
//! // Should NOT be logged because of the verbosity level set to Debug
//! trace!("Track me if you can.");
//! debug!("What's bugging you today?");
//!
//! // This one is colored in blue because of our rule on 'info' pattern
//! info!("Some interesting info here");
//!
//! ```
//!
//! #### Dependency filtering
//!
//! Filtering out every log from dependencies and putting back in everything related to a
//! particular [`target`](../log/struct.Record.html#method.target)
//!
//! ```edition2018
//! use logging::prelude::*;
//!
//! let toml = r#"
//!     [logger]
//!     level = "debug"
//!
//!         [[logger.rules]]
//!         pattern = ".*"
//!         exclude = true
//!
//!         [[logger.rules]]
//!         pattern = "^holochain"
//!         exclude = false
//!     "#;
//!
//! FastLoggerBuilder::from_toml(toml)
//!     .expect("Fail to instantiate the logger from toml.")
//!     .build()
//!     .expect("Fail to build logger from toml.");
//!
//! // Should NOT be logged
//! debug!(target: "rpc", "This is our dependency log filtering.");
//!
//! // Should be logged each in different color. We avoid filtering by prefixing using the 'target'
//! // argument.
//! info!(target: "holochain", "Log message from Holochain Core.");
//! info!(target: "holochain-app-2", "Log message from Holochain Core with instance ID 2");
//! ```

use log;
use chrono;
use colored::*;
use log::{Level, Metadata, Record, SetLoggerError};
use serde_derive::Deserialize;
use toml;

pub mod prelude;
pub mod color;
pub mod rule;

use color::{pick_color, ColoredLevelConfig};
use crossbeam_channel::{self, Receiver, Sender};
use rule::{Rule, RuleFilter};
use std::{
    boxed::Box,
    default::Default,
    env,
    io::{self, Write},
    str::FromStr,
    thread,
};

/// Default format of the log's timestamp.
const DEFAULT_TIMESTAMP_FMT: &str = "%Y-%m-%d %H:%M:%S";
/// Default channel size value.
const DEFAULT_CHANNEL_SIZE: usize = 1024;
/// Default log verbosity level.
const DEFAULT_LOG_LEVEL: Level = Level::Info;
/// Default log verbosity level as a String.
const DEFAULT_LOG_LEVEL_STR: &str = "Info";

/// Helper type pointing to a trait object in order to send around a log message.
type MsgT = Box<dyn LogMessageTrait>;

/// The logging struct where we store everything we need in order to correctly log stuff.
#[derive(Clone)]
pub struct FastLogger {
    /// Log verbosity level.
    level: Level,
    /// List of filtering [rules](RuleFilter).
    rule_filters: Vec<RuleFilter>,
    /// Color of the verbosity levels.
    level_colors: ColoredLevelConfig,
    /// Thread producer used to send log message to the log consumer.
    sender: Sender<MsgT>,
    /// Timestamp format of each log.
    timestamp_format: String,
}

impl FastLogger {
    /// Returns the current log level defined. Can be one of: Trace, Debug, Info, Warn or Error.
    pub fn level(&self) -> Level {
        self.level
    }

    /// Returns all the *rules* defined.
    pub fn rule_filters(&self) -> &Vec<RuleFilter> {
        &self.rule_filters
    }

    /// Returns the color of a log message if the logger should log it, and None other wise.
    /// We want to return the last match so we can combine rules and filters.
    pub fn should_log_in(&self, args: &str) -> Option<String> {
        if self.rule_filters.is_empty() {
            Some(String::default())
        } else {
            let mut color = Some(String::default());
            for rule_filter in self.rule_filters.iter() {
                if rule_filter.is_match(args) {
                    if rule_filter.exclude() {
                        color = None;
                    } else {
                        color = Some(rule_filter.get_color());
                    }
                }
            }
            color
        }
    }

    /// Add a rule filter to the list of existing filter. This function has to be called before
    /// registering the logger or it will do nothing because the logger becomes static.
    pub fn add_rule_filter(&mut self, rule_filter: RuleFilter) {
        self.rule_filters.push(rule_filter);
    }

    /// Flush all filter from the logger. Once a logger has been registered, it becomes static and
    /// so this function doesn't do anything.
    pub fn flush_filters(&mut self) {
        self.rule_filters.clear();
    }
}

impl log::Log for FastLogger {
    /// Determines if a log message with the specified metadata would be logged.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level()
    }

    /// This is where we build the log message and send it to the logging thread that is in charge
    /// of formatting and printing the log message.
    fn log(&self, record: &Record) {
        let args = record.args().to_string();

        // If it happens to slow the performances, maybe we should combine those two operations in
        // order to ovoid walking twice the same rule filter loop
        let should_log_args = self.should_log_in(&args);
        let should_log_target = self.should_log_in(&record.target());

        // Prioritizing the target color rule (if any) over the args one
        let should_log_in = should_log_target.clone();
        let should_log_in = match (should_log_args, should_log_target) {
            (Some(_), Some(target_color)) | (None, Some(target_color)) => Some(target_color),
            (Some(args_color), None) => Some(args_color),
            _ => should_log_in,
        };


        if self.enabled(record.metadata()) && should_log_in != None {
            let msg = LogMessage {
                args,
                module: record.module_path().unwrap_or("module-name").to_string(),
                line: record.line().unwrap_or(000),
                file: record.file().unwrap_or("").to_string(),
                level: record.level(),
                level_to_print: self.level_colors.color(record.level()).to_string(),
                thread_name: std::thread::current()
                    .name()
                    // .unwrap_or("Anonymous-thread")
                    .unwrap_or(&String::default())
                    .to_string(),
                color: should_log_in,
                target: Some(String::from(record.target())),
                timestamp_format: self.timestamp_format.to_owned(),
            };

            self.sender
                .send(Box::new(msg))
                .expect("Fail to send message to the logging thread.");
        }
    }

    /// Flushes any buffered records.
    fn flush(&self) {}
}

/// Logger Builder used to correctly set up our logging capability.
pub struct FastLoggerBuilder {
    /// Verbosity level of the logger.
    level: Level,
    /// List of filtering [rules](RuleFilter).
    rule_filters: Vec<RuleFilter>,
    /// Color of the verbosity levels.
    level_colors: ColoredLevelConfig,
    /// Size of the channel used to send log message. Currently defaulting to 512.
    channel_size: usize,
    /// The path of the file where the log will be dump in the optional case we redirect logs to a file.
    file_path: Option<String>,
    /// Timestamp format of each log.
    timestamp_format: String,
}

///
/// ```rust
/// use logging::FastLoggerBuilder;
///
/// let logger = FastLoggerBuilder::new()
///     .set_level_from_str("Debug")
///     .set_channel_size(512)
///     .build();
///
/// assert!(logger.is_ok());
/// ```

impl<'a> FastLoggerBuilder {
    /// It will init a [FastLogger] with the default argument (log level set to
    /// [Info](log::Level::Info) by default).
    pub fn new() -> Self {
        FastLoggerBuilder::default()
    }

    /// Instantiate a logger builder from a config file in TOML format.
    /// ```rust
    /// use logging::FastLoggerBuilder;
    ///
    /// let toml = r#"
    /// [logger]
    /// level = "debug"
    ///     [[logger.rules]]
    ///     pattern = ".*"
    ///     color = "red"
    /// "#;
    ///
    /// let logger = FastLoggerBuilder::from_toml(toml)
    ///                 .expect("Fail to instantiate the logger from toml.");
    /// assert!(logger.build().is_ok());
    /// ```
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        let logger_conf: LoggerConfig = toml::from_str(toml_str)?;
        let logger: Option<Logger> = logger_conf.logger;
        assert!(
            logger.is_some(),
            "The 'logger' part might be missing in the toml."
        );

        Ok(logger.unwrap().into())
    }

    /// Returns the [verbosity level](log::Level) of logger to build. Can be one of:
    /// [Trace](Level::Trace), [Debug](Level::Debug), [Info](Level::Info),
    /// [Warn](Level::Warn) or [Error](Level::Error).
    pub fn level(&self) -> Level {
        self.level
    }
    /// Set the [verbosity level](log::Level) of the logger.
    /// Value can be one of: [Trace](Level::Trace), [Debug](Level::Debug), [Info](Level::Info),
    /// [Warn](Level::Warn) or [Error](Level::Error).
    pub fn set_level(&mut self, level: Level) -> &mut Self {
        self.level = level;
        self
    }

    /// Set the [verbosity level](log::Level) of the logger from a string value: Trace, Debug,
    /// Info, Warn or Error.
    pub fn set_level_from_str(&mut self, level: &str) -> &mut Self {
        self.level = Level::from_str(level).unwrap_or_else(|_| {
            eprintln!("Fail to parse the logging level from string: '{}'.", level);
            self.level
        });
        self
    }

    /// Sets the capacity of our bounded message queue (i.e. there is a limit to how many messages
    /// it can hold at a time.). By default we use a queue of 1024.
    pub fn set_channel_size(&mut self, channel_size: usize) -> &mut Self {
        self.channel_size = channel_size;
        self
    }

    /// Customize our logging timestamp.
    pub fn timestamp_format(&mut self, timestamp_fmt: &str) -> &mut Self {
        self.timestamp_format = timestamp_fmt.to_owned();
        self
    }

    /// Add filtering [rules](rule::RuleFilter) to the logging facility.
    pub fn add_rule_filter(&mut self, rule_filter: RuleFilter) -> &mut Self {
        self.rule_filters.push(rule_filter);
        self
    }

    /// Returns all the [rules](Rule) that will be applied to the logger.
    pub fn rule_filters(&self) -> &[RuleFilter] {
        &self.rule_filters
    }

    /// Redirect log message to the provided file path.
    pub fn redirect_to_file(&mut self, file_path: &str) -> &mut Self {
        self.file_path = Some(String::from(file_path));
        self
    }

    /// Returns the file path of the logs in the case we want to redirect them to a file.
    pub fn file_path(&self) -> Option<String> {
        self.file_path.clone()
    }

    /// Registers a [FastLogger] as the comsumer of [log] facade so it becomes static and any further
    /// mutation are discarded.
    #[allow(clippy::let_and_return)]
    pub fn build(&self) -> Result<FastLogger, SetLoggerError> {
        // Let's create the logging thread that will be responsable for all the heavy work of
        // building and printing the log messages
        let (s, r): (Sender<MsgT>, Receiver<MsgT>) = crossbeam_channel::bounded(self.channel_size);

        let logger = FastLogger {
            level: self.level,
            rule_filters: self.rule_filters.to_owned(),
            level_colors: self.level_colors,
            sender: s,
            timestamp_format: self.timestamp_format.to_owned(),
        };

        match log::set_boxed_logger(Box::new(logger.clone()))
            .map(|_| log::set_max_level(self.level.to_level_filter()))
        {
            Ok(_v) => {
                // This is a hacky way to do it, because it cannot work using the Write trait object:
                // `dyn std::io::Write` cannot be sent between threads safely
                if self.file_path.is_some() {
                    let mut file_stream = {
                        let fp = match &self.file_path {
                            Some(fp) => fp.to_string(),
                            None => String::from("dummy.log"),
                        };
                        let file_path = std::path::PathBuf::from(&fp);
                        let file_stream = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&file_path)
                            .unwrap_or_else(|_| panic!("Fail to log to {:?}.", &file_path));
                        // We have some strange behavior with BufWriter and file writing: The file
                        // ends up empty most of the time during testing, so we don't use a buffer
                        // at all like : `io::BufWriter::new(file_stream)`
                        file_stream
                    };
                    thread::spawn(move || {
                        while let Ok(msg) = r.recv() {
                            // Here we use `writeln!` instead of println! in order to avoid
                            // unnecessary flush.
                            writeln!(&mut file_stream, "{}", msg.build())
                                .expect("Fail to log to file.")
                        }
                    });
                } else {
                    let mut buffer = io::BufWriter::new(io::stderr());
                    thread::spawn(move || {
                        while let Ok(msg) = r.recv() {
                            // Here we use `writeln!` instead of println! in order to avoid
                            // unnecessary flush.
                            // Currently we use `BufWriter` which has a sized buffer of about
                            // 8kb by default
                            writeln!(&mut buffer, "{}", msg.build())
                                .expect("Fail to log to file.")
                        }
                    });
                }
            }
            Err(e) => {
                eprintln!("Attempt to initialize the Logger more than once. '{}'.", e);
            }
        }

        Ok(logger)
    }

    /// Dull log build, only used for test purposes because it actually doesn't log anything by not
    /// registering the logger.
    #[allow(dead_code)]
    pub fn build_test(&self) -> Result<FastLogger, SetLoggerError> {
        // Let's create the logging thread that will be responsable for all the heavy work of
        // building and printing the log messages
        let (s, _): (Sender<MsgT>, Receiver<MsgT>) = crossbeam_channel::bounded(self.channel_size);

        let logger = FastLogger {
            level: self.level,
            rule_filters: self.rule_filters.to_owned(),
            level_colors: self.level_colors,
            sender: s,
            timestamp_format: self.timestamp_format.to_owned(),
        };

        Ok(logger)
    }
}

impl Default for FastLoggerBuilder {
    fn default() -> Self {
        // Get the log verbosity from the command line
        let level = env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_LOG_LEVEL_STR.to_string());

        FastLoggerBuilder {
            level: Level::from_str(&level).unwrap_or(DEFAULT_LOG_LEVEL),
            rule_filters: Vec::new(),
            level_colors: ColoredLevelConfig::new(),
            channel_size: DEFAULT_CHANNEL_SIZE,
            file_path: None,
            timestamp_format: String::from(DEFAULT_TIMESTAMP_FMT)
        }
    }
}

/// Initialize a simple logging instance with [Info](Level::Info) log level verbosity or retrieve
/// the level from the *RUST_LOG* environment variable and no rule filtering.
pub fn init_simple() -> Result<(), SetLoggerError> {
    FastLoggerBuilder::new().build()?;
    Ok(())
}

/// This is our log message data structure. Useful especially for performance reasons.
#[derive(Clone, Debug)]
struct LogMessage {
    /// The actual log message.
    args: String,
    /// The module name of the caller log message.
    module: String,
    /// Additional information provided by the log caller. In HC Core it correspond to the instance
    /// id of the Conductor.
    target: Option<String>,
    /// Line number of the issued log message.
    line: u32,
    /// The source file containing the message.
    file: String,
    /// Log verbosity level.
    level: Level,
    /// Log verbosity level to print with color.
    level_to_print: String,
    /// Thread name of the log message issuer. Default to `Anonymous Thread`.
    thread_name: String,
    /// The color of the log message defined by the user using [RuleFilter]. Default to color based
    /// on the thread name and the module name if not present.
    color: Option<String>,
    /// Timestamp format.
    timestamp_format: String,
}

/// For performance purpose, we build the logging message in the logging thread instead of the
/// calling one. It's primarily to deal with the potential slowness of retrieving the timestamp
/// from the OS.
trait LogMessageTrait: Send {
    fn build(&self) -> String;
}

impl LogMessageTrait for LogMessage {
    /// Build the log message as a string. Applying custom color if needed.
    fn build(&self) -> String {
        // Prioritizing `target` as a tag name and falling back to the module name if missing.
        let tag_name = self
            .target
            .to_owned()
            .unwrap_or_else(|| format!("{}{}", &self.thread_name, &self.module).to_owned());
        let base_color_on = &tag_name.to_owned();
        let pseudo_rng_color = pick_color(&base_color_on);

        // Let's colorize our logging messages
        let msg_color = match &self.color {
            Some(color) => {
                if color.is_empty() {
                    pseudo_rng_color
                } else {
                    color
                }
            },
            None => pseudo_rng_color,
        };

        // Force color on "special" log level
        let msg_color = match self.level {
            Level::Error => "Red",
            Level::Warn => "Yellow",
            _ => msg_color,
        };

        let msg = format!(
            "{level} {timestamp} [{tag}] {thread_name} {line} {args}",
            args = self.args.color(msg_color),
            tag = tag_name.bold().color(pseudo_rng_color),
            line = format!("{}:{}", self.file, self.line).italic(),
            // We might consider retrieving the timestamp once and proceed logging
            // in batch in the future, if this ends up being performance critical
            timestamp = chrono::Local::now().format(&self.timestamp_format),
            level = self.level_to_print.bold(),
            thread_name = self.thread_name.underline(),
        );
        msg.to_string()
    }
}

/// This is a helper for [TOML](toml) deserialization into [Logger].
#[derive(Debug, Deserialize)]
struct LoggerConfig {
    pub logger: Option<Logger>,
}

/// This structure is a helper for the [TOML](toml) logging configuration.
#[derive(Clone, Debug, Deserialize)]
struct Logger {
    /// Verbosity level of the logger.
    level: String,
    /// The path to the file in the case where we want to redirect our log to a file.
    file: Option<String>,
    /// List of filtering [rules](RuleFilter).
    rules: Option<Vec<Rule>>,
    /// Timestamp format.
    timestamp_format: Option<String>,
}

impl From<Logger> for FastLoggerBuilder {
    fn from(logger: Logger) -> Self {
        let rule_filters: Vec<RuleFilter> = logger
            .rules
            .unwrap_or_else(|| vec![])
            .into_iter()
            .map(|rule| rule.into())
            .collect();

        FastLoggerBuilder {
            level: Level::from_str(&logger.level).unwrap_or(Level::Info),
            rule_filters,
            file_path: logger.file,
            timestamp_format: logger.timestamp_format.unwrap_or_else(|| String::from(DEFAULT_TIMESTAMP_FMT)),
            ..FastLoggerBuilder::default()
        }
    }
}

#[test]
fn should_log_test() {
    use rule::RuleFilterBuilder;

    let mut logger = FastLoggerBuilder::new()
        .set_level_from_str("Debug")
        .add_rule_filter(
            RuleFilterBuilder::new()
                .set_pattern("foo")
                .set_exclusion(false)
                .set_color("Blue")
                .build(),
        )
        .build_test()
        .unwrap();

    assert_eq!(logger.should_log_in("bar"), Some(String::from("")));

    assert_eq!(logger.should_log_in("xfooy"), Some(String::from("Blue")));

    // rule to reject anything with baz
    logger.add_rule_filter(RuleFilter::new("baz", true, "White"));
    assert_eq!(logger.should_log_in("baz"), None);

    // rule to accept anything with b
    logger.add_rule_filter(RuleFilter::new("b", false, "Green"));
    assert_eq!(logger.should_log_in("xboy"), Some(String::from("Green")));
}

#[test]
fn filtering_back_log_test() {
    let toml = r#"
        [logger]
        level = "debug"

            [[logger.rules]]
            pattern = ".*"
            exclude = true

            [[logger.rules]]
            pattern = "^holochain"
            exclude = false

            [[logger.rules]]
            pattern = "Cyan"
            exclude = false
            color = "Cyan"

            [[logger.rules]]
            pattern = "app-6"
            exclude = false
            color = "Green"
    "#;

    let logger_conf: LoggerConfig =
        toml::from_str(toml).expect("Fail to deserialize logger from toml.");
    let logger: Option<Logger> = logger_conf.logger;
    assert!(logger.is_some());

    let flb: FastLoggerBuilder = logger.unwrap().into();
    let logger = flb.build_test().unwrap();

    // This log entry should be filtered: 'debug!(target: "rpc", "...")'
    assert_eq!(logger.should_log_in("rpc"), None);

    // This one should be logged: 'info!(target: "holochain-app-2", "...")' because of 2nd rule
    assert_ne!(logger.should_log_in("holochain"), None);

    // This next one should be logged in red: 'debug!(target: "holochain-app-6", "...'Red'...")'
    assert_eq!(logger.should_log_in("app-6"), Some(String::from("Green")));

}

#[test]
fn logger_conf_deserialization_test() {
    let toml = r#"
        [logger]
        level = "debug"
        file = "humpty_dumpty.log"

            [[logger.rules]]
            pattern = ".*"
            color = "red"
    "#;

    let logger_conf: LoggerConfig =
        toml::from_str(toml).expect("Fail to deserialize logger from toml.");
    let logger: Option<Logger> = logger_conf.logger;
    assert!(logger.is_some());

    let flb: FastLoggerBuilder = logger.unwrap().into();

    // Log verbosity check
    assert_eq!(flb.level(), Level::Debug);

    // File dump check
    assert_eq!(flb.file_path(), Some(String::from("humpty_dumpty.log")));
}

#[test]
fn fastloggerbuilder_conf_deserialization_test() {
    let toml = r#"
        [logger]
        level = "debug"
        file = "humpty_dumpty.log"

            [[logger.rules]]
            pattern = ".*"
            color = "red"
    "#;

    let flb =
        FastLoggerBuilder::from_toml(&toml).expect("Fail to init `FastLoggerBuilder` from toml.");

    // Log verbosity check
    assert_eq!(flb.level(), Level::Debug);

    // File dump check
    assert_eq!(flb.file_path(), Some(String::from("humpty_dumpty.log")));
}

#[test]
fn log_rules_deserialization_test() {
    use rule;

    let toml = r#"
        [logger]
        level = "debug"

            [[logger.rules]]
            pattern = ".*"
            color = "red"
            [[logger.rules]]
            pattern = "twice"
            exclude = true
            color = "magenta"
    "#;

    let logger_conf: LoggerConfig =
        toml::from_str(toml).expect("Fail to deserialize logger from toml.");
    let logger: Option<Logger> = logger_conf.logger;
    assert!(logger.is_some());

    let rule0 = rule::Rule {
        pattern: String::from(".*"),
        exclude: Some(false),
        color: Some(String::from("red")),
    };
    let rule1 = rule::Rule {
        pattern: String::from("twice"),
        exclude: Some(true),
        color: Some(String::from("magenta")),
    };

    let flb: FastLoggerBuilder = logger.unwrap().into();

    let rule0_from_toml: Rule = flb.rule_filters()[0].clone().into();
    let rule1_from_toml: Rule = flb.rule_filters()[1].clone().into();
    // File dump check
    assert_eq!(rule0_from_toml, rule0);
    assert_eq!(rule1_from_toml, rule1);
}

#[test]
fn configure_log_level_from_env_test() {
    env::set_var("RUST_LOG", "warn");
    let flb = FastLoggerBuilder::new();

    assert_eq!(flb.level(), Level::Warn);

    let logger = flb.build_test().unwrap();
    assert_eq!(logger.level(), Level::Warn)
}

#[test]
fn log_to_file_test() {
    let toml = r#"
        [logger]
        level = "debug"
        file = "logger_dump.log"

            [[logger.rules]]
            pattern = ".*"
            color = "Yellow"
    "#;

    FastLoggerBuilder::from_toml(toml).expect("Fail to load logging conf from toml.");
}
