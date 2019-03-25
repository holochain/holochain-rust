use holochain_core_types::{error::HolochainError, json::*};

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct KeystoreListResult(Vec<String>);
impl KeystoreListResult {
    pub fn new(string_list: Vec<String>) -> Self {
        KeystoreListResult(string_list)
    }
    pub fn list(&self) -> Vec<String> {
        self.0.clone()
    }
}

// NOTE: These properties must match the attributes in the conductor
// agent/keystore/* callback functions because the json encoding is just
// passed directly on

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct KeystoreNewRandomArgs {
    pub dst_id: String,
    pub size: usize,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct KeystoreDeriveSeedArgs {
    pub src_id: String,
    pub dst_id: String,
    pub context: String,
    pub index: u64,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub enum KeyType {
    Signing,
    Encrypting,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct KeystoreDeriveKeyArgs {
    pub src_id: String,
    pub dst_id: String,
    pub key_type: KeyType,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct KeystoreSignArgs {
    pub src_id: String,
    pub payload: String,
}
