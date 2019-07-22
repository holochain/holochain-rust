use holochain_core_types::{crud_status::CrudStatus, entry::EntryWithMetaAndHeader,chain_header::ChainHeader};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::{cas::content::Address,eav::Value};

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum GetLinksNetworkQuery {
    Count,
    Links(bool)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub struct GetLinkData
{
    pub address : Address,
    pub crud_status : CrudStatus,
    pub target : Value,
    pub headers : Option<Vec<ChainHeader>>
}

impl GetLinkData
{
    pub fn new(address:Address,crud_status:CrudStatus,target:Value,headers:Option<Vec<ChainHeader>>) -> GetLinkData
    {
        GetLinkData
        {
            address,
            crud_status,
            target,
            headers
        }
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum GetLinksNetworkResult {
    Count(usize),
    Links(Vec<GetLinkData>)
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
