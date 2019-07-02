use chrono;
use colored::*;
use log::{Level, Metadata, Record, SetLoggerError};
use serde_derive::Deserialize;
// use toml;

pub mod color;
pub mod tag;

use color::{pick_color, ColoredLevelConfig};
use crossbeam_channel::{self, Receiver, Sender};
use std::{boxed::Box, default::Default, io::Write, str::FromStr, thread};
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
    pub fn level(&self) -> Level {
        self.level
    }

    pub fn tag_filters(&self) -> &Vec<TagFilter> {
        &self.tag_filters
    }

    /// Returns the color of a log message if the logger should log it, and None other wise.
    pub fn should_log_in(&self, args: &str) -> Option<String> {
        if self.tag_filters.len() < 1 {
            return Some(String::default());
        } else {
            let mut color = String::default();
            for tag_filter in self.tag_filters.iter() {
                let is_match = tag_filter.is_match(args);

                if is_match {
                    if tag_filter.exclude() {
                        return None;
                    } else {
                        color = tag_filter.tag_color();
                        // return Some(tag_filter.tag_color())
                    }
                }
            }
            return Some(color);
        }
    }

    /// Add a tag filter to the list of existing filter.
    pub fn add_tag_filter(&mut self, tag_filter: TagFilter) {
        self.tag_filters.push(tag_filter);
    }

    /// Flush all filter from the logger.
    pub fn flush_filters(&mut self) {
        self.tag_filters.clear();
    }
}

impl log::Log for FastLogger {
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
/// let logger = FastLoggerBuilder::new()
///     .set_level_from_str("Debug")
///     .set_channel_size(512)
///     .build();
/// ```
impl FastLoggerBuilder {
    pub fn new() -> Self {
        FastLoggerBuilder::default()
    }

    pub fn set_level(&mut self, level: Level) -> &mut Self {
        self.level = level;
        self
    }

    pub fn set_level_from_str(&mut self, level: &str) -> &mut Self {
        self.level = Level::from_str(level).unwrap_or_else(|_| {
            eprintln!("Fail to parse the logging level from string: '{}'.", level);
            Level::Info
        });
        self
    }

    pub fn set_channel_size(&mut self, channel_size: usize) -> &mut Self {
        self.channel_size = channel_size;
        self
    }

    pub fn add_tag_filter(&mut self, tag_filter: TagFilter) -> &mut Self {
        self.tag_filters.push(tag_filter);
        self
    }

    pub fn build(&self) -> Result<FastLogger, SetLoggerError> {
        // Let's create the logging thread that will be responsable for all the heavy work of
        // building and printing the log messages
        let (s, r): (Sender<MsgT>, Receiver<MsgT>) = crossbeam_channel::bounded(self.channel_size);

        thread::spawn(move || {
            while let Ok(msg) = r.recv() {
                // eprintln!("{}", msg.build());
                writeln!(&mut std::io::stderr(), "{}", msg.build())
                    .expect("Fail to write to output.");
            }
        });

        let logger = FastLogger {
            level: self.level,
            tag_filters: self.tag_filters.to_owned(),
            level_colors: self.level_colors,
            sender: s,
        };

        log::set_boxed_logger(Box::new(logger.clone()))
            .map(|_| log::set_max_level(self.level.to_level_filter()))?;

        Ok(logger)
    }
}

impl Default for FastLoggerBuilder {
    fn default() -> Self {
        FastLoggerBuilder {
            level: Level::Info,
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
                if color.len() == 0 {
                    pick_color(&self.module)
                } else { color }
            },
            None => pick_color(&self.module),
        };

        let msg = format!(
            "{timestamp} | {thread_name}: {module} @ l.{line} - {level} - {args}",
            args = self.args.color(msg_color),
            module = self.module,
            line = self.line,
            // We might considere retrieving the timestamp once and proceed logging
            // in batch in the future, if this ends up being performance critical
            timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
            level = self.level,
            thread_name = self.thread_name,
        );
        msg.to_string()
    }
}

#[derive(Deserialize)]
struct Logger {
    level: String,
    rules: Option<Vec<Rule>>,
}

#[derive(Deserialize)]
struct Rule {
    pub pattern: String,
    pub exclude: Option<bool>,
    pub color: Option<String>,
}

impl From<Logger> for FastLogger {
    fn from(logger: Logger) -> Self {
        let _tag_filters: Vec<Rule> = Vec::with_capacity(logger.rules.unwrap_or(vec![]).len());
        FastLogger {
            level: Level::from_str(&logger.level).unwrap_or(Level::Info),
            ..FastLoggerBuilder::default().build().unwrap()
        }
    }
}

impl From<Rule> for TagFilter {
    fn from(rule: Rule) -> Self {
        let tf = TagFilter::default();
        TagFilter::new(
            &rule.pattern,
            rule.exclude.unwrap_or(tf.exclude()),
            &rule.color.unwrap_or(tf.tag_color()),
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
fn test_rules_serialization() {
    let _toml = r#"
        [logger]
        level = "debug"
            [[logger.rules.rules]]
            pattern = ".*"
            color = "red"
        "#;

    // let logger: Logger = toml::from_str(toml).expect("Fail to deserialize logger from toml.");
    // let fl: FastLogger = logger.into();
}
