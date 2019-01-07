//! This logger is the logger that's attached to each Holochain application
//! which is separate from standard logging via the log crate warn! info! debug! logging that
//! gets emitted globaly from the container.
use chrono::Local;
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
        let date = Local::now();
        println!("{}:{}", date.format("%Y-%m-%d %H:%M:%S"), msg);
    }
    // fn new() -> SimpleLogger {
    //      SimpleLogger {}
    // }
}
#[cfg(test)]
pub mod tests {
    use crate::logger::{Logger, SimpleLogger};
    #[test]
    fn test_logger() {
        let mut logger_test = SimpleLogger {};
        logger_test.log("Example Log".to_string());
    }
}
