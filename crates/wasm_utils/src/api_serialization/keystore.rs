use holochain_json_api::{error::JsonError, json::*};

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct KeystoreListResult {
    pub ids: Vec<String>,
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
    pub seed_type: SeedType,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub enum KeyType {
    Signing,
    Encrypting,
}
/// Enum of all the types of seeds
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SeedType {
    /// Root / Master seed
    Root,
    /// Revocation seed
    Revocation,
    /// Auth seed
    Auth,
    /// Device seed
    Device,
    /// Derivative of a Device seed with a PIN
    DevicePin,
    /// DNA specific seed
    DNA,
    /// Seed for a one use only key
    OneShot,
    /// Seed used only in tests or mocks
    Mock,
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

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct KeystoreGetPublicKeyArgs {
    pub src_id: String,
}
