use holochain_core::signal::Signal;
use holochain_json_api::{error::JsonError, json::JsonString};

/// This struct wraps a Signal from core before serializing and sending over
/// an interface to the UI or other client.
#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct SignalWrapper {
    pub signal: Signal,
    pub instance_id: String,
}
