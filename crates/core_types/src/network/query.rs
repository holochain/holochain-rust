use crate::{
    chain_header::ChainHeader, crud_status::CrudStatus, entry::EntryWithMetaAndHeader,
    time::Iso8601,
};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::{cas::content::Address, eav::Value};

//makes more sense semantically to have this as an enum instead of a boolean.
//it adds more meaning to what sorting mechanism it is
#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone, PartialEq, Eq, Hash)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Descending
    }
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq, Eq, Hash, DefaultJson)]
pub struct TimePagination {
    pub from_time: Iso8601,
    pub limit: usize,
}
#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash, DefaultJson)]
pub struct SizePagination {
    pub page_number: usize,
    pub page_size: usize,
}
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq, Eq, Hash, DefaultJson)]
pub enum Pagination {
    Size(SizePagination),
    Time(TimePagination),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone, Default)]
pub struct GetLinksQueryConfiguration {
    pub headers: bool,
    pub pagination: Option<Pagination>,
    pub sort_order: Option<SortOrder>,
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
    GetLinks(
        Option<String>,
        Option<String>,
        Option<CrudStatus>,
        GetLinksNetworkQuery,
    ),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum NetworkQueryResult {
    Entry(Option<EntryWithMetaAndHeader>),
    Links(GetLinksNetworkResult, Option<String>, Option<String>),
}
