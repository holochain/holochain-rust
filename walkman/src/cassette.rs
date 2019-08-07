use std::{
    fs::File,
    io::{BufRead, BufReader, Result},
    path::PathBuf,
};

pub type WalkmanNodeId = usize;
pub type WalkmanNodeName = String;

pub trait WalkmanData {
    fn to_string(&self) -> String;
}

const LOG_PREFIX: &'static str = "🖭 ";

pub struct WalkmanLog {
    timestamp: u32,
    data: Box<dyn WalkmanData>,
}

impl WalkmanLog {
    pub fn from_data(data: dyn WalkmanData) -> Self {
        Self {
            timestamp: 0, // TODO
            data: data,
        }
    }
}

pub struct WalkmanEvent {
    node_id: WalkmanNodeId,
    timestamp: u32,
    data: Box<dyn WalkmanData>,
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
                if line.starts_with(LOG_PREFIX) {
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
