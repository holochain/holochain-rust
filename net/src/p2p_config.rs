use holochain_core_types::json::JsonString;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum P2pBackendKind {
    MOCK,
    IPC,
}

#[derive(Clone, Debug)]
pub struct P2pConfig {
    pub backend_kind: P2pBackendKind,
    pub backend_config: JsonString,
}