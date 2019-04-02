//! This logger is the logger that's attached to each Holochain application
//! which is separate from standard logging via the log crate warn! info! debug! logging that
//! gets emitted globaly from the conductor.
use chrono::Local;
use holochain_common::env_vars::EnvVar;
use std::sync::{mpsc, Arc, Mutex};

/// trait that defines the logging functionality that holochain_core requires
pub trait Logger: Send {
    // Add log message to logger
    fn log(&mut self, msg: String);
    // Dump all held logs
    fn dump(&self) -> String {
        String::new()
    }
}
#[derive(Clone)]
pub struct SimpleLogger {
    // log: Vec<String>,
}
// ignore this in test coverage as it is only side effects
#[cfg_attr(tarpaulin, skip)]
impl Logger for SimpleLogger {
    fn log(&mut self, msg: String) {
        // note that this behaviour is documented within
        // holochain_common::env_vars module and should be updated
        // if this logic changes
        if EnvVar::SimpleLoggerMute.value().is_err() {
            let date = Local::now();
            println!("{}:{}", date.format("%Y-%m-%d %H:%M:%S"), msg);
        }
    }
}

/// create a test logger
pub fn test_logger() -> Arc<Mutex<TestLogger>> {
    Arc::new(Mutex::new(TestLogger { log: Vec::new() }))
}

#[derive(Clone, Debug)]
pub struct TestLogger {
    pub log: Vec<String>,
}

impl Logger for TestLogger {
    fn log(&mut self, msg: String) {
        // Debugging code (do not remove)
        // println!("{}", msg);
        self.log.push(msg);
    }
    fn dump(&self) -> String {
        format!("{:?}", self.log)
    }
}

pub type Receiver = mpsc::Receiver<(String, String)>;
pub type Sender = mpsc::Sender<(String, String)>;

#[derive(Clone)]
pub struct ChannelLogger {
    id: String,
    sender: Sender,
}

impl Logger for ChannelLogger {
    fn log(&mut self, msg: String) {
        self.sender.send((self.id.clone(), msg)).unwrap();
    }
}

impl ChannelLogger {
    pub fn new(id: String, sender: Sender) -> ChannelLogger {
        ChannelLogger { id, sender }
    }
    pub fn setup() -> (Sender, Receiver) {
        mpsc::channel()
    }
}
pub fn default_handler(msg: String) {
    let date = Local::now();
    println!("{}:{}", date.format("%Y-%m-%d %H:%M:%S"), msg);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn simple_logger_smoke_test() {
        let mut logger_test = SimpleLogger {};
        logger_test.log("Example Log".to_string());
    }

    #[test]
    fn test_test_logger() {
        let mut logger = TestLogger { log: Vec::new() };
        logger.log("test".to_string());
        assert_eq!(logger.dump(), "[\"test\"]".to_string());
    }

    #[test]
    fn test_channel_logger() {
        let (tx, rx) = ChannelLogger::setup();
        let mut logger = ChannelLogger::new("Me".to_string(), tx.clone());
        logger.log("fish".to_string());
        match rx.recv() {
            Ok((id, msg)) => {
                assert_eq!(id, "Me");
                assert_eq!(msg, "fish");
            }
            Err(_) => assert!(false),
        }
    }
}
