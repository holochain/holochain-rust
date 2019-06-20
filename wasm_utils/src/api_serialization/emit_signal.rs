use holochain_json_api::{error::JsonError, json::JsonString};

/// Struct for input data received when invoke_emit_signal is called
#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct EmitSignalArgs {
    pub name: String,
    pub arguments: JsonString,
}
