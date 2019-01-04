use holochain_core::{logger::{ChannelLogger, Sender}};
use std::thread;
use chrono::{Local, DateTime};
use colored::*;
use regex;

pub type LogRule = regex::Regex;
pub struct LogRules {
    rules:  Vec<LogRule>
}
impl LogRules {
    pub fn new()->Self {
        LogRules{rules: Vec::new()}
    }
    pub fn add_rule(&mut self,rule: &str) {
        let x = regex::Regex::new(rule).unwrap();
        self.rules.push(x);
    }
}
pub struct DebugLogger {
    sender: Sender,
    //    thread: thread::JoinHandle<()>
}

impl DebugLogger {
    pub fn new() -> Self {
        let(tx,rx) = ChannelLogger::setup();
        let logger = DebugLogger{
            sender: tx.clone(),
        };

        let r = LogRules::new();
        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok((id,msg)) => {
                        debug(&r,id,msg)
                    },
                    Err(_) => break,
                }
            }
        });
        logger
    }
    pub fn get_sender(&self) -> Sender {
        self.sender.clone()
    }
}

pub fn debug(rules: &LogRules, id: String,msg: String) {
    match filter(rules, id, msg) {
        Some(message) => render(message),
        None=>(),
    }
}

pub fn render(msg: LogMessage) {
    let x = format!("{}:{}:{}", msg.date.format("%Y-%m-%d %H:%M:%S"),msg.id,msg.msg);
    println!("{}",x);
}

pub struct LogMessage {
    date: DateTime<Local>,
    id: String,
    msg: String,
}

pub fn filter(rules: &LogRules, id: String, msg: String) -> Option<LogMessage> {
    let mut message = LogMessage{
        date: Local::now(),
        id: "".to_string(),
        msg: msg.clone(),
    };
    if rules.rules.len() == 0 {
        message.id = id.green().to_string();
        Some(message)
    } else {
        for r in &rules.rules {
            if r.is_match(&msg) {
                message.id = id.red().to_string();
                return Some(message);
            }
        }
        None
    }
}
