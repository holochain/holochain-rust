/// trait that defines the logging functionality that hc_core requires
pub trait Logger {
    fn log(&mut self, msg: String);
    fn read(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct SimpleLogger {
    //    log: Vec<String>,
}

extern crate chrono;

use self::chrono::Local;

impl Logger for SimpleLogger {
    fn log(&mut self, msg: String) {
        let date = Local::now();
        println!("{}:{}", date.format("%Y-%m-%d %H:%M:%S"), msg);
    }
    fn read(&self) -> String {
        "".to_string()
    }
}
