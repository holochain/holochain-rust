use holochain_core_types::{error::HolochainError, json::*};

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct KeystoreListResult(Vec<String>);
impl KeystoreListResult {
    pub fn new(string_list: Vec<String>) -> Self {
        KeystoreListResult(string_list)
    }
}
