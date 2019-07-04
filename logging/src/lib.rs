use chrono;
use colored::*;
use log::{Level, Metadata, Record, SetLoggerError};
use serde_derive::Deserialize;
use toml;

pub mod color;
pub mod tag;

use color::{pick_color, ColoredLevelConfig};
use crossbeam_channel::{self, Receiver, Sender};
use std::{
    boxed::Box,
    default::Default,
    io::{self, Write},
    str::FromStr,
    thread,
    env,
};
use tag::TagFilter;

type MsgT = Box<dyn LogMessageTrait>;

/// The logging struct where we store everything we need in order to correctly log stuff.
#[derive(Clone)]
pub struct FastLogger {
    level: Level,
    tag_filters: Vec<TagFilter>,
    level_colors: ColoredLevelConfig,
    sender: Sender<MsgT>,
}

impl FastLogger {
    /// Returns the current log level defined. Can be one of: Trace, Debug, Info, Warn or Error.
    pub fn level(&self) -> Level {
        self.level
    }

    /// Returns all the *rules* / *tag filters* defined.
    pub fn tag_filters(&self) -> &Vec<TagFilter> {
        &self.tag_filters
    }

    /// Returns the color of a log message if the logger should log it, and None other wise.
    pub fn should_log_in(&self, args: &str) -> Option<String> {
        if self.tag_filters.is_empty() {
            Some(String::default())
        } else {
            let mut color = String::default();
            for tag_filter in self.tag_filters.iter() {
                if tag_filter.is_match(args) {
                    if tag_filter.exclude() {
                        return None;
                    } else {
                        color = tag_filter.tag_color();
                        // Do we want to return the fist match or the last one?
                        // return Some(tag_filter.tag_color())
                    }
                }
            }
            Some(color)
        }
    }

    /// Add a tag filter to the list of existing filter. This function has to be called be
    /// registering the logger or it will do nothing because the logger becomes static.
    pub fn add_tag_filter(&mut self, tag_filter: TagFilter) {
        self.tag_filters.push(tag_filter);
    }

    /// Flush all filter from the logger. Once a logger has been registered, it becomes static and
    /// so this function doesn't do anything.
    pub fn flush_filters(&mut self) {
        self.tag_filters.clear();
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
        let should_log = self.should_log_in(&args);

        if self.enabled(record.metadata()) && should_log != None {
            let msg = LogMessage {
                args,
                module: record.module_path().unwrap_or("module-name").to_string(),
                line: record.line().unwrap_or(000),
                level: self.level_colors.color(record.level()).to_string(),
                thread_name: std::thread::current()
                    .name()
                    // .unwrap_or("Thread-name")
                    .unwrap_or("Anonymous thread")
                    .to_string(),
                color: should_log,
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
    level: Level,
    tag_filters: Vec<TagFilter>,
    level_colors: ColoredLevelConfig,
    channel_size: usize,
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
impl FastLoggerBuilder {
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

        let flb: FastLoggerBuilder = logger.unwrap().into();
        Ok(flb)
    }

    /// Returns the verbosity level of logger to build. Can be one of: Trace, Debug, Info, Warn or Error.
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
            eprintln!(
                "Fail to parse the logging level from string: '{}'.",
                level
            );
            self.level
        });
        self
    }

    /// Sets the capacity of our bounded message queue (i.e. there is a limit to how many messages
    /// it can hold at a time.). By default we use a queue of 512.
    pub fn set_channel_size(&mut self, channel_size: usize) -> &mut Self {
        self.channel_size = channel_size;
        self
    }

    /// Add filtering [rules (or tag filters)](tag::TagFilter) to the logging facility.
    pub fn add_tag_filter(&mut self, tag_filter: TagFilter) -> &mut Self {
        self.tag_filters.push(tag_filter);
        self
    }

    /// Registers a [FastLogger] as the comsumer of [log] facade so it becomes static and any further
    /// mutation are discarded.
    pub fn build(&self) -> Result<FastLogger, SetLoggerError> {
        // Let's create the logging thread that will be responsable for all the heavy work of
        // building and printing the log messages
        let (s, r): (Sender<MsgT>, Receiver<MsgT>) = crossbeam_channel::bounded(self.channel_size);

        let logger = FastLogger {
            level: self.level,
            tag_filters: self.tag_filters.to_owned(),
            level_colors: self.level_colors,
            sender: s,
        };


        // This is where I should impl the output

        match log::set_boxed_logger(Box::new(logger.clone()))
            .map(|_| log::set_max_level(self.level.to_level_filter()))
        {
            Ok(_v) => {
                thread::spawn(move || {
                    while let Ok(msg) = r.recv() {
                        // Here we use `writeln!` instead of println! in order to avoid
                        // unnecessary flush.
                        // Currently we use `BufWriter` which has a sized buffer of about
                        // 8kb by default
                        writeln!(&mut io::BufWriter::new(io::stderr()), "{}", msg.build())
                            .expect("Fail to write to output.");
                    }
                });
            }
            Err(e) => {
                eprintln!("Attempt to initialize the Logger more than once. '{}'.", e);
            }
        }

        Ok(logger)
    }
}

impl Default for FastLoggerBuilder {
    fn default() -> Self {
        // Get the log verbosity from the command line
        let level = env::var("RUST_LOG").unwrap_or("Info".to_string());

        FastLoggerBuilder {
            level: Level::from_str(&level).unwrap_or(Level::Info),
            tag_filters: Vec::new(),
            level_colors: ColoredLevelConfig::new(),
            channel_size: 512,
        }
    }
}

/// Initialize a simple logging instance with Debug level and no tag filtering.
pub fn init_simple() -> Result<(), SetLoggerError> {
    FastLoggerBuilder::new().set_level(Level::Debug).build()?;
    Ok(())
}

/// This is our log message data structure. Usefull especially for performance reasons.
#[derive(Clone, Debug)]
struct LogMessage {
    args: String,
    module: String,
    line: u32,
    level: String,
    thread_name: String,
    color: Option<String>,
}

trait LogMessageTrait: Send {
    fn build(&self) -> String;
}

/// For performance purpose, we build the logging message in the logging thread instead of the
/// calling one. It's primarily to deal with the potential slowness of retreaving the timestamp
/// from the OS.
impl LogMessageTrait for LogMessage {
    fn build(&self) -> String {
        // Let's colorize our logging messages
        let msg_color = match &self.color {
            Some(color) => {
                if color.is_empty() {
                    pick_color(&self.module)
                } else {
                    color
                }
            }
            None => pick_color(&self.module),
        };

        let msg = format!(
            "{timestamp} | {thread_name}: {module} @ l.{line} - {level} - {args}",
            args = self.args.color(msg_color),
            module = self.module,
            line = self.line,
            // We might considere retrieving the timestamp once and proceed logging
            // in batch in the future, if this ends up being performance critical
            // timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
            timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            level = self.level,
            thread_name = self.thread_name,
        );
        msg.to_string()
    }
}

/// This is a helper for TOML deserialization.
#[derive(Debug, Deserialize)]
struct LoggerConfig {
    pub logger: Option<Logger>,
}

/// This structure is a helper for the TOML logging configuration.
#[derive(Clone, Debug, Deserialize)]
struct Logger {
    level: String,
    rules: Option<Vec<Rule>>,
}

#[derive(Clone, Debug, Deserialize)]
struct Rule {
    pub pattern: String,
    pub exclude: Option<bool>,
    pub color: Option<String>,
}

impl From<Logger> for FastLoggerBuilder {
    fn from(logger: Logger) -> Self {
        let tag_filters: Vec<TagFilter> = logger
            .rules
            .unwrap_or_else(|| vec![])
            .into_iter()
            .map(|rule| rule.into())
            .collect();

        FastLoggerBuilder {
            level: Level::from_str(&logger.level).unwrap_or(Level::Info),
            tag_filters,
            ..FastLoggerBuilder::default()
        }
    }
}

impl From<Rule> for TagFilter {
    fn from(rule: Rule) -> Self {
        let tf = TagFilter::default();
        TagFilter::new(
            &rule.pattern,
            rule.exclude.unwrap_or_else(|| tf.exclude()),
            &rule.color.unwrap_or_else(|| tf.tag_color()),
        )
    }
}

#[test]
fn should_log_test() {
    use tag::TagFilterBuilder;

    let mut logger = FastLoggerBuilder::new()
        .set_level_from_str("Debug")
        .add_tag_filter(
            TagFilterBuilder::new()
                .set_pattern("foo")
                .set_exclusion(false)
                .set_color("Blue")
                .build(),
        )
        .build()
        .unwrap();

    assert_eq!(logger.should_log_in("bar"), Some(String::from("")));

    assert_eq!(logger.should_log_in("xfooy"), Some(String::from("Blue")));

    // rule to reject anything with baz
    logger.add_tag_filter(TagFilter::new("baz", true, "White"));
    // rule to accept anything with b
    logger.add_tag_filter(TagFilter::new("b", false, "Green"));

    assert_eq!(logger.should_log_in("baz"), None);
    assert_eq!(logger.should_log_in("xboy"), Some(String::from("Green")));
}

#[test]
fn test_rules_deserialization() {
    let toml = r#"
        [logger]
        level = "debug"
            [[logger.rules]]
            pattern = ".*"
            color = "red"
    "#;

    let logger_conf: LoggerConfig =
        toml::from_str(toml).expect("Fail to deserialize logger from toml.");
    let logger: Option<Logger> = logger_conf.logger;
    assert!(logger.is_some());

    let _flb: FastLoggerBuilder = logger.unwrap().into();
    assert!(_flb.build().is_ok());
}

#[test]
fn configure_log_level_from_env_test() {
    env::set_var("RUST_LOG", "warn");
    let flb = FastLoggerBuilder::new();

    assert_eq!(flb.level(), Level::Warn)
}
