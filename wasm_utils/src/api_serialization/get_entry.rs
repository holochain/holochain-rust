use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    crud_status::CrudStatus,
    entry::{Entry, EntryWithMeta},
    error::HolochainError,
    json::*,
};
use std::collections::HashMap;

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone, PartialEq)]
pub enum StatusRequestKind {
    Initial,
    Latest,
    All,
}
impl Default for StatusRequestKind {
    fn default() -> Self {
        StatusRequestKind::Latest
    }
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct GetEntryOptions {
    pub status_request: StatusRequestKind,
    pub entry: bool,
    pub header: bool,
    pub sources: bool,
}

impl Default for GetEntryOptions {
    fn default() -> Self {
        GetEntryOptions {
            status_request: StatusRequestKind::default(),
            entry: true,
            header: false,
            sources: false,
        }
    }
}

impl GetEntryOptions {
    pub fn new(
        status_request: StatusRequestKind,
        entry: bool,
        header: bool,
        sources: bool,
    ) -> Self {
        GetEntryOptions {
            status_request,
            entry,
            header,
            sources,
        }
    }
}

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct GetEntryArgs {
    pub address: Address,
    pub options: GetEntryOptions,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct EntryHistory {
    pub addresses: Vec<Address>,
    pub entries: Vec<Entry>,
    pub crud_status: Vec<CrudStatus>,
    pub crud_links: HashMap<Address, Address>,
}
impl EntryHistory {
    pub fn new() -> Self {
        EntryHistory {
            addresses: Vec::new(),
            entries: Vec::new(),
            crud_status: Vec::new(),
            crud_links: HashMap::new(),
        }
    }

    pub fn push(&mut self, entry_with_meta: &EntryWithMeta) {
        let address = entry_with_meta.entry.address();
        self.addresses.push(address.clone());
        self.entries.push(entry_with_meta.entry.clone());
        self.crud_status.push(entry_with_meta.crud_status);
        if let Some(new_address) = entry_with_meta.maybe_crud_link.clone() {
            self.crud_links.insert(address, new_address);
        }
    }
}

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct GetEntryResult {
    pub found: bool, // indicates search was successful
    // pub header: Option<ChainHeader>,   // header if requested in options
    // pub sources: Option<Vec<Address>>, // sources if requested in options
    pub history: Option<EntryHistory>,
}
impl GetEntryResult {
    pub fn new() -> Self {
        GetEntryResult {
            found: false,
            //            sources: None,
            history: None,
        }
    }
    pub fn found(&self) -> bool {
        self.found
    }

    fn add_history(&mut self) {
        self.history = Some(EntryHistory::new())
    }

    pub fn push(&mut self, entry_with_meta: &EntryWithMeta) {
        if !self.found {
            self.found = true;
            self.add_history();
        }
        match self.history {
            None => unreachable!(),
            Some(ref mut history) => history.push(entry_with_meta),
        };
    }

    pub fn latest(&self) -> Option<Entry> {
        if !self.found() {
            return None;
        }
        let entry = match self.history {
            None => None,
            Some(ref h) => Some(h.entries.iter().next().unwrap().clone()),
        };
        entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use holochain_core_types::entry::test_entry;

    #[test]
    fn test_get_entry_result_found() {
        let result = GetEntryResult::new();
        assert!(!result.found())
    }

    #[test]
    fn test_add_history() {
        let mut result = GetEntryResult::new();
        assert!(result.history.is_none());
        result.add_history();
        assert!(result.history.is_some());
    }

    #[test]
    fn test_get_entry_latest() {
        let mut result = GetEntryResult::new();
        assert_eq!(result.latest(), None);
        result.push(&EntryWithMeta {
            entry: test_entry(),
            crud_status: CrudStatus::LIVE,
            maybe_crud_link: None,
        });
        assert!(result.found());
        assert_eq!(result.latest(), Some(test_entry()));
    }
}
