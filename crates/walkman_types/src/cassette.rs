use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    time::Instant,
};

pub type WalkmanNodeId = usize;
pub type WalkmanNodeName = String;

pub struct WalkmanData(String);

const LOG_PREFIX: &'static str = "(walkman) ";

pub struct WalkmanLogItem {
    time: Instant,
    event: WalkmanEvent,
}

impl WalkmanLogItem {
    pub fn from_data(data: WalkmanData) -> Self {
        Self {
            timestamp: Instant::now(),
            data,
        }
    }
}

pub struct WalkmanEvent {
    node_id: WalkmanNodeId,
    timestamp: u32,
    data: WalkmanData,
}

pub struct WalkmanCassette {
    node_ids: Vec<WalkmanNodeName>,
    events: Vec<WalkmanEvent>,
}

type WalkmanError = String;

impl WalkmanCassette {
    fn try_from_logs(
        logs: Vec<(WalkmanNodeName, PathBuf)>,
    ) -> Result<WalkmanCassette, WalkmanError> {
        let mut events = Vec::new();
        for (node_id, (name, file)) in logs.iter().enumerate() {
            let mut f = File::open(file).map_err(|e| e.to_string())?;
            let mut contents = String::new();
            for line in BufReader::new(f).lines() {
                if line.expect("IO error reading log").starts_with(LOG_PREFIX) {
                    events.push(WalkmanEvent { node_id })
                }
            }
        }
        Ok(Self {
            node_ids: logs.keys(),
            events: Vec::new(), // TODO
        })
    }
}
