//! This logger is the logger that's attached to each Holochain application
//! which is separate from standard logging via the log crate warn! info! debug! logging that
//! gets emitted globaly from the container.

use chrono::Local;
use std::fmt;

/// trait that defines the logging functionality that holochain_core requires
pub trait Logger: fmt::Debug {
    fn log(&mut self, msg: String);
}

#[derive(Clone)]
pub struct SimpleLogger {
    // log: Vec<String>,
}

impl Logger for SimpleLogger {
    fn log(&mut self, msg: String) {
        let date = Local::now();
        println!("{}:{}", date.format("%Y-%m-%d %H:%M:%S"), msg);
    }
    // fn new() -> SimpleLogger {
    //      SimpleLogger {}
    // }
}

impl fmt::Debug for SimpleLogger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<empty>")
    }
}
