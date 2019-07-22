use holochain_core_types::{crud_status::CrudStatus, entry::EntryWithMetaAndHeader};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::{cas::content::Address,eav::Value};

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum GetLinksNetworkQuery {
    Count,
    Links,
}


#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum GetLinksNetworkResult {
    Count(usize),
    Links(Vec<(Address, CrudStatus,Value)>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum NetworkQuery {
    GetEntry,
    GetLinks(String, String, Option<CrudStatus>, GetLinksNetworkQuery),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum NetworkQueryResult {
    Entry(Option<EntryWithMetaAndHeader>),
    Links(GetLinksNetworkResult, String, String),
}
