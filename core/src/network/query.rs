use holochain_core_types::{crud_status::CrudStatus, entry::EntryWithMetaAndHeader};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::cas::content::Address;

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum NetworkQuery {
    GetEntry,
    GetLinks(String, String),
    GetLinksCount(String, String, Option<CrudStatus>),
    GetLinksByTag(String,Option<CrudStatus>)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum NetworkQueryResult {
    Entry(Option<EntryWithMetaAndHeader>),
    Links(Vec<(Address, CrudStatus)>, String, String),
    LinksCount(usize, String, String),
    LinksCountByTag(usize,String)
}
