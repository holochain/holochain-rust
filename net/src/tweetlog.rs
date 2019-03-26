use std::{
    collections::{HashMap, HashSet},
    string::*,
    sync::RwLock,
};

//--------------------------------------------------------------------------------------------------
// MACROS
//--------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! log_t {
    ($($arg:tt)+) => { {
        let msg = format!($($arg)+);
        TWEETLOG.read().unwrap().t(&msg);
      } };
}
#[macro_export]
macro_rules! log_d {
    ($($arg:tt)+) => { {
        let msg = format!($($arg)+);
        TWEETLOG.read().unwrap().d(&msg);
      } };
}
#[macro_export]
macro_rules! log_i {
    ($($arg:tt)+) => { {
        let msg = format!($($arg)+);
        TWEETLOG.read().unwrap().i(&msg);
      } };
}
#[macro_export]
macro_rules! log_w {
    ($($arg:tt)+) => { {
        let msg = format!($($arg)+);
        TWEETLOG.read().unwrap().w(&msg);
      } };
}
#[macro_export]
macro_rules! log_e {
    ($($arg:tt)+) => { {
        let msg = format!($($arg)+);
        TWEETLOG.read().unwrap().e(&msg);
      } };
}

#[macro_export]
macro_rules! log_tt {
    ($tag:expr, $($arg:tt)+) => {
        let msg = format!($($arg)+);
        TWEETLOG.read().unwrap().tt($tag, &msg);
    };
}
#[macro_export]
macro_rules! log_dd {
    ($tag:expr, $($arg:tt)+) => {
        let msg = format!($($arg)+);
        TWEETLOG.read().unwrap().dd($tag, &msg);
    };
}
#[macro_export]
macro_rules! log_ii {
    ($tag:expr, $($arg:tt)+) => {
        let msg = format!($($arg)+);
        TWEETLOG.read().unwrap().ii($tag, &msg);
    };
}
#[macro_export]
macro_rules! log_ww {
    ($tag:expr, $($arg:tt)+) => {
        let msg = format!($($arg)+);
        TWEETLOG.read().unwrap().ww($tag, &msg);
    };
}
#[macro_export]
macro_rules! log_ee {
    ($tag:expr, $($arg:tt)+) => {
        let msg = format!($($arg)+);
        TWEETLOG.read().unwrap().ee($tag, &msg);
    };
}

//--------------------------------------------------------------------------------------------------
// LOGLEVEL
//--------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum LogLevel {
    Trace = 1,
    Debug,
    Info,
    Warning,
    Error,
}

impl From<char> for LogLevel {
    fn from(l: char) -> Self {
        match l {
            't' => LogLevel::Trace,
            'd' => LogLevel::Debug,
            'i' => LogLevel::Info,
            'w' => LogLevel::Warning,
            'e' => LogLevel::Error,
            _ => unreachable!(),
        }
    }
}
impl LogLevel {
    pub fn to_char(level: &LogLevel) -> char {
        match level {
            LogLevel::Trace => 't',
            LogLevel::Debug => 'd',
            LogLevel::Info => 'i',
            LogLevel::Warning => 'w',
            LogLevel::Error => 'e',
        }
    }

    pub fn as_char(&self) -> char {
        LogLevel::to_char(self)
    }
}

//--------------------------------------------------------------------------------------------------
// TWEETLOGGER
//--------------------------------------------------------------------------------------------------

/// TweetLog has a TweetLogger per Tag
/// which has its own loglevel and callbacks
#[derive(Debug)]
struct TweetLogger {
    pub level: LogLevel,
    pub callbacks: HashSet<listenerCallback>,
}

impl TweetLogger {
    pub fn new() -> Self {
        TweetLogger::with_level(LogLevel::Info)
    }

    pub fn with_level(level: LogLevel) -> Self {
        TweetLogger {
            level,
            callbacks: HashSet::new(),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// TWEETLOG
//--------------------------------------------------------------------------------------------------

#[allow(non_camel_case_types)]
type listenerCallback = fn(LogLevel, Option<&str>, &str);

/// this is the actual memory space for our Tweetlog singleton
lazy_static! {
    pub static ref TWEETLOG: RwLock<Tweetlog> = RwLock::new(Tweetlog::new());
}

/// Tweetlog singleton
/// Holds a TweetLogger per Tag
/// Creates a global Tag: "_"
pub struct Tweetlog {
    log_by_tag: HashMap<String, TweetLogger>,
}

impl Tweetlog {
    pub fn new() -> Self {
        let mut tlog = Tweetlog {
            log_by_tag: HashMap::new(),
        };
        tlog.log_by_tag.insert("_".to_string(), TweetLogger::new());
        tlog
    }
}

impl Tweetlog {
    /// Create internal TweetLogger for a Tag
    /// as well as a TweetProxy
    pub fn add(&mut self, tag: &str) -> TweetProxy {
        self.log_by_tag.insert(tag.to_string(), TweetLogger::new());
        TweetProxy::new(tag)
    }

    /// Set the logging level, either globally, or for a tag
    pub fn set(&mut self, level: LogLevel, maybe_tag: Option<String>) {
        let tag = match maybe_tag {
            None => "_".to_string(),
            Some(tag) => tag,
        };
        // update existing logger
        {
            let maybe_logger = self.log_by_tag.get_mut(&tag);
            if let Some(logger) = maybe_logger {
                logger.level = level;
                return;
            };
        }
        // otherwise create new one
        self.log_by_tag.insert(tag, TweetLogger::with_level(level));
    }

    /// Clear any registered log levels
    pub fn reset_levels(self) {
        for (_, mut logger) in self.log_by_tag {
            logger.level = LogLevel::Info;
        }
    }

    pub fn listen(&mut self, cb: listenerCallback) {
        self.listen_to_tag("_", cb);
    }

    pub fn listen_to_tag(&mut self, tag: &str, cb: listenerCallback) {
        let logger = self
            .log_by_tag
            .get_mut(tag)
            .expect("TweetLogger for Tag does not exist");
        logger.callbacks.insert(cb);
    }

    /// Clear any registered listener
    pub fn unlisten_all(self) {
        for (_, mut logger) in self.log_by_tag {
            logger.callbacks.clear();
        }
    }

    /// Clear any registered log listeners or levels
    pub fn unlisten(&mut self, tag: &str) {
        let maybe_logger = self.log_by_tag.get_mut(tag);
        if let Some(logger) = maybe_logger {
            logger.callbacks.clear();
        }
    }

    /// Check if a given level and tag would be logged
    pub fn should(&self, level: LogLevel, maybe_tag: Option<String>) -> bool {
        let tag = match maybe_tag {
            None => "_".to_string(),
            Some(tag) => tag,
        };
        let maybe_logger = self.log_by_tag.get(&tag);
        match maybe_logger {
            None => false,
            Some(logger) => (logger.level.clone() as usize) <= (level as usize),
        }
    }

    /// callback according to level and tag
    fn tweet(&self, level: LogLevel, maybe_tag: Option<&str>, msg: &str) {
        // replace None to "_"
        let tag = match maybe_tag {
            None => "_",
            Some(tag) => tag,
        };
        // Find logger, if unknown tag use general
        let maybe_logger = self.log_by_tag.get(tag);
        let logger = match maybe_logger {
            None => self.log_by_tag.get("_").unwrap(),
            Some(logger) => logger,
        };
        // print if logger can
        if (logger.level.clone() as usize) <= (level.clone() as usize) {
            for cb in logger.callbacks.clone() {
                cb(level.clone(), Some(tag), msg);
            }
        }
    }

    // -- sugar -- //

    pub fn t(&self, msg: &str) {
        self.tweet(LogLevel::Trace, None, msg);
    }
    pub fn tt(&self, tag: &str, msg: &str) {
        self.tweet(LogLevel::Trace, Some(tag), msg);
    }

    pub fn d(&self, msg: &str) {
        self.tweet(LogLevel::Debug, None, msg);
    }
    pub fn dd(&self, tag: &str, msg: &str) {
        self.tweet(LogLevel::Debug, Some(tag), msg);
    }

    pub fn i(&self, msg: &str) {
        self.tweet(LogLevel::Info, None, msg);
    }
    pub fn ii(&self, tag: &str, msg: &str) {
        self.tweet(LogLevel::Info, Some(tag), msg);
    }

    pub fn w(&self, msg: &str) {
        self.tweet(LogLevel::Warning, None, msg);
    }
    pub fn ww(&self, tag: &str, msg: &str) {
        self.tweet(LogLevel::Warning, Some(tag), msg);
    }

    pub fn e(&self, msg: &str) {
        self.tweet(LogLevel::Error, None, msg);
    }
    pub fn ee(&self, tag: &str, msg: &str) {
        self.tweet(LogLevel::Error, Some(tag), msg);
    }

    // -- provided listeners -- //

    /// println without displaying tag
    pub fn console(level: LogLevel, maybe_tag: Option<&str>, msg: &str) {
        match maybe_tag {
            None => println!("[{}] {}\n", level.as_char(), msg),
            Some(_tag) => println!("[{}] {}\n", level.as_char(), msg),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// TWEETPROXY
//--------------------------------------------------------------------------------------------------

/// A facade for calling the singleton with a specific tag
#[derive(Debug)]
pub struct TweetProxy {
    tag: String,
}

impl TweetProxy {
    pub fn new(tag: &str) -> Self {
        TweetProxy {
            tag: tag.to_owned(),
        }
    }

    pub fn t(&self, msg: &str) {
        TWEETLOG
            .read()
            .unwrap()
            .tweet(LogLevel::Trace, Some(&self.tag), msg);
    }
    pub fn d(&self, msg: &str) {
        TWEETLOG
            .read()
            .unwrap()
            .tweet(LogLevel::Debug, Some(&self.tag), msg);
    }
    pub fn i(&self, msg: &str) {
        TWEETLOG
            .read()
            .unwrap()
            .tweet(LogLevel::Info, Some(&self.tag), msg);
    }
    pub fn w(&self, msg: &str) {
        TWEETLOG
            .read()
            .unwrap()
            .tweet(LogLevel::Warning, Some(&self.tag), msg);
    }
    pub fn e(&self, msg: &str) {
        TWEETLOG
            .read()
            .unwrap()
            .tweet(LogLevel::Error, Some(&self.tag), msg);
    }
}

//--------------------------------------------------------------------------------------------------
// TESTS
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_default_should() {
        let tweetlog = Tweetlog::new();

        assert!(!tweetlog.should(LogLevel::Trace, None));
        assert!(!tweetlog.should(LogLevel::Debug, None));
        assert!(tweetlog.should(LogLevel::Info, None));
        assert!(tweetlog.should(LogLevel::Warning, None));
        assert!(tweetlog.should(LogLevel::Error, None));
    }

    #[test]
    fn log_should() {
        let mut tweetlog = Tweetlog::new();
        tweetlog.set(LogLevel::Error, None);

        assert!(!tweetlog.should(LogLevel::Trace, None));
        assert!(!tweetlog.should(LogLevel::Debug, None));
        assert!(!tweetlog.should(LogLevel::Info, None));
        assert!(!tweetlog.should(LogLevel::Warning, None));
        assert!(tweetlog.should(LogLevel::Error, None));

        tweetlog.set(LogLevel::Trace, None);

        assert!(tweetlog.should(LogLevel::Trace, None));
        assert!(tweetlog.should(LogLevel::Debug, None));
        assert!(tweetlog.should(LogLevel::Info, None));
        assert!(tweetlog.should(LogLevel::Warning, None));
        assert!(tweetlog.should(LogLevel::Error, None));
    }

    #[test]
    fn log_should_tag() {
        let mut tweetlog = Tweetlog::new();
        tweetlog.set(LogLevel::Error, Some("toto".to_string()));

        assert!(!tweetlog.should(LogLevel::Trace, None));
        assert!(!tweetlog.should(LogLevel::Debug, None));
        assert!(tweetlog.should(LogLevel::Info, None));
        assert!(tweetlog.should(LogLevel::Warning, None));
        assert!(tweetlog.should(LogLevel::Error, None));
    }

    #[test]
    fn log_println_hello() {
        let mut tweetlog = Tweetlog::new();
        tweetlog.add("errorlog");

        // set general logging to error only
        tweetlog.set(LogLevel::Warning, None);
        tweetlog.listen(Tweetlog::console);

        // set testlogger output to trace level
        tweetlog.add("tracelog");
        tweetlog.set(LogLevel::Trace, Some("tracelog".to_string()));
        tweetlog.listen_to_tag("tracelog", Tweetlog::console);

        tweetlog.t("hello trace");
        tweetlog.d("hello debug");
        tweetlog.i("hello info");
        tweetlog.w("hello warning");
        tweetlog.e("hello error");
    }
}
