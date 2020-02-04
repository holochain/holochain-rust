use crate::{chain_header::ChainHeader, crud_status::CrudStatus, entry::EntryWithMetaAndHeader};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::{cas::content::Address, eav::Value};

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash, DefaultJson)]
pub struct Pagination {
    pub page_number: usize,
    pub page_size: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone, Default)]
pub struct GetLinksQueryConfiguration {
    pub headers: bool,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum GetLinksNetworkQuery {
    Count,
    Links(GetLinksQueryConfiguration),
}

// The data returned by a remote node on responding to a get_links query
#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub struct GetLinkFromRemoteData {
    pub link_add_address: Address,
    pub crud_status: CrudStatus,
    pub tag: String,
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
    Links(Vec<GetLinkFromRemoteData>),
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
