use crate::WireMessage;
use chrono::{DateTime, Utc};
use holochain_tracing_macros::newrelic_autotrace;
use lib3h_protocol::{types::AgentPubKey, uri::Lib3hUri};
use log::error;
use parking_lot::Mutex;
use std::{collections::LinkedList, fs::OpenOptions, io::Write, path::PathBuf};

#[derive(Serialize, Debug)]
enum Direction {
    In,
    Out,
}

#[derive(Serialize)]
struct MessageLog {
    time: String,
    uri: Lib3hUri,
    agent: AgentPubKey,
    direction: Direction,
    message: WireMessage,
}

lazy_static! {
    pub static ref MESSAGE_LOGGER: Mutex<MessageLogger> = Mutex::new(MessageLogger::new());
}

/// Logger for wire messages with a buffer.
/// Each call to `log_in` and `log_out` will just add a `MessageLog` instance to the buffer.
/// `write_thread()` starts a thread that clears the buffer and writes new lines to the log file
/// once every second.
pub struct MessageLogger {
    buffer: LinkedList<MessageLog>,
    file_path: PathBuf,
    running: bool,
}

#[newrelic_autotrace(SIM2H)]
impl MessageLogger {
    pub fn new() -> Self {
        MessageLogger {
            buffer: LinkedList::new(),
            file_path: PathBuf::from("sim2h_messages.log"),
            running: false,
        }
    }

    pub fn start(&mut self) {
        if !self.running {
            self.running = true;
            Self::write_thread()
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    /// The thread started by this function is doing the actual work of taking all items
    /// from the buffer and adding lines to the log file.
    /// Stops if Self::running was set to false.
    fn write_thread() {
        std::thread::Builder::new()
            .name("MessageLogger".into())
            .spawn(|| loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let mut logger = MESSAGE_LOGGER.lock();
                if !logger.is_running() {
                    return;
                }
                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(logger.file_path())
                {
                    let to_append = logger
                        .buffer
                        .split_off(0)
                        .into_iter()
                        .map(|log| Self::log_to_line(&log))
                        .collect::<Vec<String>>()
                        .join("\n");
                    if let Err(e) = file.write(to_append.as_bytes()) {
                        error!("Error writing log file: {:?}", e);
                    }
                } else {
                    error!("Could not open log file!")
                }
            })
            .expect("Could not spawn logger thread");
    }

    /// Serializes a `MessageLog` item to a line that gets added to the log file.
    /// Creates a tab-separated concatenation of the logs elements.
    fn log_to_line(log: &MessageLog) -> String {
        format!(
            "{}\t{:?}\t{}\t{}\t{}",
            log.time,
            log.direction,
            log.agent,
            log.uri,
            serde_json::to_string(&log.message).expect("Message must be serializable")
        )
    }

    fn time() -> String {
        let now: DateTime<Utc> = Utc::now();
        format!("{}", now)
    }

    pub fn log_in(&mut self, agent: AgentPubKey, uri: Lib3hUri, message: WireMessage) {
        if self.running {
            self.buffer.push_back(MessageLog {
                time: Self::time(),
                uri,
                agent,
                direction: Direction::In,
                message,
            });
        }
    }

    pub fn log_out(&mut self, agent: AgentPubKey, uri: Lib3hUri, message: WireMessage) {
        if self.running {
            self.buffer.push_back(MessageLog {
                time: Self::time(),
                uri,
                agent,
                direction: Direction::Out,
                message,
            });
        }
    }

    pub fn set_logfile(&mut self, path: PathBuf) {
        self.file_path = path;
    }

    pub fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }
}
