extern crate serde_json;
use chain_header::ChainHeader;
use hash::HashString;

#[derive(Serialize, Deserialize)]
pub struct ValidationData {
    pub chain_header: Option<ChainHeader>,
    pub sources: Vec<HashString>,
    pub source_chain_entries: Option<Vec<serde_json::Value>>,
    pub source_chain_headers: Option<Vec<ChainHeader>>,
    pub custom: Option<serde_json::Value>,
    pub lifecycle: HcEntryLifecycle,
    pub action: HcEntryAction,
}

#[derive(Serialize, Deserialize)]
pub enum HcEntryLifecycle {
    Chain,
    Dht,
    Meta,
}

#[derive(Serialize, Deserialize)]
pub enum HcEntryAction {
    Commit,
    Modify,
    Delete,
}

#[derive(Serialize, Deserialize)]
pub enum HcLinkAction {
    Commit,
    Delete,
}

//#[derive(Serialize, Deserialize)]
//#[serde(remote = "Either")]
//pub enum EitherDef<L, R> {
//    Left(L),
//    Right(R),
//}
