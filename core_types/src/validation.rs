extern crate serde_json;
use chain_header::ChainHeader;
use entry::Entry;
use hash::HashString;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ValidationPackage {
    pub chain_header: Option<ChainHeader>,
    pub source_chain_entries: Option<Vec<Entry>>,
    pub source_chain_headers: Option<Vec<ChainHeader>>,
    pub custom: Option<String>,
}

impl ValidationPackage {
    pub fn only_header(header: ChainHeader) -> ValidationPackage {
        ValidationPackage {
            chain_header: Some(header),
            source_chain_entries: None,
            source_chain_headers: None,
            custom: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ValidationPackageDefinition {
    Entry,          //sending only the entry
    ChainEntries,   //sending all (public?) source chain entries
    ChainHeaders,   //sending all source chain headers
    ChainFull,      //sending the whole chain, entries and headers
    Custom(String), //sending something custom
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ValidationData {
    pub package: ValidationPackage,
    pub sources: Vec<HashString>,
    pub lifecycle: EntryLifecycle,
    pub action: EntryAction,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum EntryLifecycle {
    Chain,
    Dht,
    Meta,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum EntryAction {
    Commit,
    Modify,
    Delete,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum LinkAction {
    Commit,
    Delete,
}
