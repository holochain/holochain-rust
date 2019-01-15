use holochain_core_types::{
    cas::content::Address,
    chain_header::ChainHeader,
    entry::Entry, 
    entry::entry_type::EntryType,
    error::HolochainError,
    json::*,
};

// QueryArgsNames -- support querying single/multiple EntryType names
#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone, PartialEq)]
#[serde(untagged)] // No type in serialized data; try deserializing as a String, then as a Vec<String>
pub enum QueryArgsNames {
    QueryName(String),
    QueryList(Vec<String>),
}

impl Default for QueryArgsNames {
    fn default() -> QueryArgsNames {
        QueryArgsNames::QueryList(vec![])
    }
}

// Handle automatic convertions from various types into the appropriate QueryArgsNames enum type
impl From<EntryType> for QueryArgsNames {
    fn from(e: EntryType) -> QueryArgsNames {
        QueryArgsNames::QueryName(e.to_string())
    }
}

impl From<String> for QueryArgsNames {
    fn from(s: String) -> QueryArgsNames {
        QueryArgsNames::QueryName(s)
    }
}

impl<'a> From<&'a str> for QueryArgsNames {
    fn from(s: &'a str) -> QueryArgsNames {
        QueryArgsNames::QueryName(s.to_string())
    }
}

impl From<Vec<String>> for QueryArgsNames {
    fn from(v: Vec<String>) -> QueryArgsNames {
        QueryArgsNames::QueryList(v)
    }
}

impl<'a> From<Vec<&'a str>> for QueryArgsNames {
    fn from(v: Vec<&'a str>) -> QueryArgsNames {
        QueryArgsNames::QueryList(v.iter().map(|s| s.to_string()).collect())
    }
}

#[derive(Deserialize, Default, Debug, Serialize, DefaultJson)]
pub struct QueryArgs {
    pub entry_type_names: QueryArgsNames,
    pub start: Option<u32>, // TODO: These should be "typed", so order cannot be confued
    pub limit: Option<u32>,
    pub headers: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone, PartialEq)]
pub struct QueryResultItem {
    header: Option<ChainHeader>,
    entry: Option<Entry>,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone, PartialEq)]
pub enum QueryResult {
    Addresses(Vec<Address>),
    Headers(Vec<ChainHeader>),
}
