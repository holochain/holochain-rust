use holochain_core::signal::Signal;
use holochain_core_types::{error::HolochainError, json::JsonString};

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct SignalWrapper {
    pub signal: Signal,
    pub instance_id: String,
}
