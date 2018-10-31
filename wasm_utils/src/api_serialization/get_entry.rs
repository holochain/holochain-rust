use holochain_core_types::{
    entry::SerializedEntry,
    error::{HcResult, HolochainError},
    json::*,
};
use serde_json;
use std::convert::TryFrom;

// empty for now, need to implement get options
#[derive(Deserialize, Debug, Serialize)]
pub struct GetEntryOptions {}
