use holochain_core::{context::InstanceStats, signal::Signal};
use holochain_json_api::{error::JsonError, json::JsonString};
use std::collections::HashMap;

/// This enum wraps a Signal from core before serializing and sending over
/// an interface to the UI or other client.
#[derive(Serialize, Deserialize, Debug, DefaultJson)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "type")]
pub enum SignalWrapper {
    InstanceSignal {
        signal: Signal,
        instance_id: String,
    },
    InstanceStats {
        instance_stats: HashMap<String, InstanceStats>,
    },
}
