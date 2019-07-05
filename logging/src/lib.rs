use chrono;
use colored::*;
use log::{Level, Metadata, Record, SetLoggerError};
use serde_derive::Deserialize;
use toml;

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

/// Helper type pointing to a trait object in order to send around a log message.
type MsgT = Box<dyn LogMessageTrait>;

/// The logging struct where we store everything we need in order to correctly log stuff.
#[derive(Clone)]
pub struct FastLogger {
    level: Level,
    rule_filters: Vec<RuleFilter>,
    level_colors: ColoredLevelConfig,
    sender: Sender<MsgT>,
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
    pub fn should_log_in(&self, args: &str) -> Option<String> {
        if self.rule_filters.is_empty() {
            Some(String::default())
        } else {
            let mut color = String::default();
            for rule_filter in self.rule_filters.iter() {
                if rule_filter.is_match(args) {
                    if rule_filter.exclude() {
                        return None;
                    } else {
                        color = rule_filter.get_color();
                        // Do we want to return the fist match or the last one?
                        // return Some(rule_filter.get_color())
                    }
                }
            }
            Some(color)
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
        let should_log = self.should_log_in(&args);

        if self.enabled(record.metadata()) && should_log != None {
            let msg = LogMessage {
                args,
                module: record.module_path().unwrap_or("module-name").to_string(),
                line: record.line().unwrap_or(000),
                level: self.level_colors.color(record.level()).to_string(),
                thread_name: std::thread::current()
                    .name()
                    .unwrap_or("Anonymous thread")
                    .to_string(),
                color: should_log,
                target: Some(String::from(record.target())),
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
            eprintln!("Fail to parse the logging level from string: '{}'.", level);
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
                        // ends up empty most of the time, so we don't use a buffer at all
                        // io::BufWriter::new(file_stream)
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
                    thread::spawn(move || {
                        while let Ok(msg) = r.recv() {
                            // Here we use `writeln!` instead of println! in order to avoid
                            // unnecessary flush.
                            // Currently we use `BufWriter` which has a sized buffer of about
                            // 8kb by default
                            writeln!(&mut io::BufWriter::new(io::stderr()), "{}", msg.build())
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
}

impl Default for FastLoggerBuilder {
    fn default() -> Self {
        // Get the log verbosity from the command line
        let level = env::var("RUST_LOG").unwrap_or_else(|_| "Info".to_string());

        FastLoggerBuilder {
            level: Level::from_str(&level).unwrap_or(Level::Info),
            rule_filters: Vec::new(),
            level_colors: ColoredLevelConfig::new(),
            channel_size: 512,
            file_path: None,
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
    /// Log verbosity level.
    level: String,
    /// Thread name of the log message issuer. Default to `Anonymous Thread`.
    thread_name: String,
    /// The color of the log message defined by the user using [RuleFilter]. Default to color based
    /// on the thread name and the module name if not present.
    color: Option<String>,
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
        let module_name = self.target.to_owned().unwrap_or(self.module.to_owned());
        let base_color_on = format!("{}", &module_name).to_owned();
        // let base_color_on = format!("{}{}", &self.thread_name, &self.module).to_owned();

        // Let's colorize our logging messages
        let msg_color = match &self.color {
            Some(color) => {
                if color.is_empty() {
                    pick_color(&base_color_on)
                } else {
                    color
                }
            }
            None => pick_color(&base_color_on),
        };

        let msg = format!(
            "{timestamp} | {thread_name}: {module} @ l.{line} - {level} - {args}",
            args = self.args.color(msg_color),
            module = module_name,
            line = self.line,
            // We might consider retrieving the timestamp once and proceed logging
            // in batch in the future, if this ends up being performance critical
            // timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
            timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            level = self.level,
            thread_name = self.thread_name,
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
            ..FastLoggerBuilder::default()
        }
    }
}

// impl From<Rule> for RuleFilter {
//     fn from(rule: Rule) -> Self {
//         let tf = RuleFilter::default();
//         RuleFilter::new(
//             &rule.pattern,
//             rule.exclude.unwrap_or_else(|| tf.exclude()),
//             &rule.color.unwrap_or_else(|| tf.get_color()),
//         )
//     }
// }

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
        .build()
        .unwrap();

    assert_eq!(logger.should_log_in("bar"), Some(String::from("")));

    assert_eq!(logger.should_log_in("xfooy"), Some(String::from("Blue")));

    // rule to reject anything with baz
    logger.add_rule_filter(RuleFilter::new("baz", true, "White"));
    // rule to accept anything with b
    logger.add_rule_filter(RuleFilter::new("b", false, "Green"));

    assert_eq!(logger.should_log_in("baz"), None);
    assert_eq!(logger.should_log_in("xboy"), Some(String::from("Green")));
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

    assert_eq!(flb.level(), Level::Warn)
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
