use holochain_json_api::{error::JsonError, json::*};

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct WasmString(String);

impl ToString for WasmString {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<String> for WasmString {
    fn from(s: String) -> WasmString {
        WasmString(s)
    }
}
