use holochain_core_types::{
    cas::content::Address, entry::entry_type::EntryType, error::HolochainError, json::*,
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

impl From<&str> for QueryArgsNames {
    fn from(s: &str) -> QueryArgsNames {
        QueryArgsNames::QueryName(s.to_string())
    }
}

impl From<Vec<String>> for QueryArgsNames {
    fn from(v: Vec<String>) -> QueryArgsNames {
        QueryArgsNames::QueryList(v)
    }
}

impl From<Vec<&str>> for QueryArgsNames {
    fn from(v: Vec<&str>) -> QueryArgsNames {
        QueryArgsNames::QueryList(v.iter().map(|s| s.to_string()).collect())
    }
}

// Query{Args,Result} -- the query API parameters and return type
#[derive(Deserialize, Default, Debug, Serialize, DefaultJson)]
pub struct QueryArgs {
    pub entry_type_names: QueryArgsNames,
    pub start: u32,
    pub limit: u32,
}

pub type QueryResult = Vec<Address>;
