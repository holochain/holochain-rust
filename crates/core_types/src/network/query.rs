use crate::{chain_header::ChainHeader, crud_status::CrudStatus, entry::EntryWithMetaAndHeader};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::{cas::content::Address, eav::Value};

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub struct GetLinksQueryConfiguration {
    pub headers: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum GetLinksNetworkQuery {
    Count,
    Links(GetLinksQueryConfiguration),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub struct GetLinkData {
    pub address: Address,
    pub crud_status: CrudStatus,
    pub target: Value,
    pub tag: String,
    pub headers: Option<Vec<ChainHeader>>,
}

impl GetLinkData {
    pub fn new(
        address: Address,
        crud_status: CrudStatus,
        target: Value,
        tag: String,
        headers: Option<Vec<ChainHeader>>,
    ) -> GetLinkData {
        GetLinkData {
            address,
            crud_status,
            target,
            tag,
            headers,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum GetLinksNetworkResult {
    Count(usize),
    Links(Vec<GetLinkData>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum NetworkQuery {
    GetEntry,
    GetLinks(String, String, Option<CrudStatus>, GetLinksNetworkQuery),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum NetworkQueryResult {
    Entry(Option<EntryWithMetaAndHeader>),
    Links(GetLinksNetworkResult, String, String),
}
