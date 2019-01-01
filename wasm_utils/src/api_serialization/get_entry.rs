use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    crud_status::CrudStatus,
    entry::{entry_type::EntryType, Entry, EntryWithMeta},
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

/// Structure used to specify what should be returned to a call to get_entry_result()
/// The default is to return the latest entry.
#[derive(Deserialize, Debug, Serialize, DefaultJson, PartialEq, Clone)]
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

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct EntryResultMeta {
    pub address: Address,
    pub entry_type: EntryType,
    pub crud_status: CrudStatus,
}

/// Structure that holds data returned from a get entry request.
/// When the meta is None, we know the entry wasn't found.  This is
/// because at the very least the entry_type and the address will be
/// returned if nothing else was requested in the GetEntryOptions
#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct GetEntryResultItem {
    pub meta: Option<EntryResultMeta>,
    pub entry: Option<Entry>,
}
impl GetEntryResultItem {
    pub fn new(maybe_entry_with_meta: Option<&EntryWithMeta>) -> Self {
        match maybe_entry_with_meta {
            Some(entry_with_meta) => GetEntryResultItem {
                meta: Some(EntryResultMeta {
                    address: entry_with_meta.entry.address(),
                    entry_type: entry_with_meta.entry.entry_type(),
                    crud_status: entry_with_meta.crud_status,
                }),
                entry: Some(entry_with_meta.entry.clone()),
            },
            _ => GetEntryResultItem {
                meta: None,
                entry: None,
            },
        }
    }
}

/// Structure that holds a whole crud status history if the status request
/// in the GetEntryOptions was set to StatusRequestKind::All
#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct EntryHistory {
    pub items: Vec<GetEntryResultItem>,
    pub crud_links: HashMap<Address, Address>,
}
impl EntryHistory {
    pub fn new() -> Self {
        EntryHistory {
            items: Vec::new(),
            crud_links: HashMap::new(),
        }
    }

    pub fn push(&mut self, entry_with_meta: &EntryWithMeta) {
        let address = entry_with_meta.entry.address();
        let item = GetEntryResultItem::new(Some(entry_with_meta));
        self.items.push(item);
        if let Some(new_address) = entry_with_meta.maybe_crud_link.clone() {
            self.crud_links.insert(address, new_address);
        }
    }
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub enum GetEntryResultType {
    Single(GetEntryResultItem),
    All(EntryHistory),
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct GetEntryResult {
    pub result: GetEntryResultType,
    // pub header: Option<ChainHeader>,   // header if requested in options
    // pub sources: Option<Vec<Address>>, // sources if requested in options
}
impl GetEntryResult {
    pub fn new(
        request_kind: StatusRequestKind,
        maybe_entry_with_meta: Option<&EntryWithMeta>,
    ) -> Self {
        match request_kind {
            StatusRequestKind::All => {
                let mut entry_result = GetEntryResult {
                    result: GetEntryResultType::All(EntryHistory::new()),
                };
                if maybe_entry_with_meta.is_some() {
                    entry_result.push(maybe_entry_with_meta.unwrap());
                }
                entry_result
            }
            _ => GetEntryResult {
                result: GetEntryResultType::Single(GetEntryResultItem::new(maybe_entry_with_meta)),
            },
        }
    }
    pub fn found(&self) -> bool {
        match self.result {
            GetEntryResultType::Single(ref item) => item.meta.is_some(),
            GetEntryResultType::All(ref history) => !history.items.is_empty(),
        }
    }

    /// clears the entry result to be equivalent to not found
    pub fn clear(&mut self) {
        match self.result {
            GetEntryResultType::Single(_) => {
                self.result = GetEntryResultType::Single(GetEntryResultItem::new(None))
            }
            GetEntryResultType::All(ref mut history) => history.items.clear(),
        };
    }

    /// adds an item to history, or if Single, writes over the current value of the item
    pub fn push(&mut self, entry_with_meta: &EntryWithMeta) {
        match self.result {
            GetEntryResultType::Single(_) => {
                self.result =
                    GetEntryResultType::Single(GetEntryResultItem::new(Some(entry_with_meta)))
            }
            GetEntryResultType::All(ref mut history) => history.push(entry_with_meta),
        };
    }

    /// returns the entry searched for.  Note that if the GetEntryOptions did not
    /// include a request for the entry value, this function will return None even if the
    /// entry was found.
    pub fn latest(&self) -> Option<Entry> {
        match self.result {
            GetEntryResultType::Single(ref item) => item.entry.clone(),
            GetEntryResultType::All(ref history) => {
                let last = history.items.last();
                if last.is_none() {
                    return None;
                }
                last.unwrap().entry.clone()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use holochain_core_types::entry::{test_entry, test_entry_a, test_entry_b};

    #[test]
    fn test_get_entry_result_found() {
        let result = GetEntryResult::new(StatusRequestKind::Initial, None);
        assert!(!result.found());
        let result = GetEntryResult::new(StatusRequestKind::Latest, None);
        assert!(!result.found());
        let result = GetEntryResult::new(StatusRequestKind::All, None);
        assert!(!result.found());
    }

    #[test]
    fn test_get_entry_single_latest() {
        let mut result = GetEntryResult::new(StatusRequestKind::Initial, None);
        assert_eq!(result.latest(), None);
        result.push(&EntryWithMeta {
            entry: test_entry(),
            crud_status: CrudStatus::Live,
            maybe_crud_link: None,
        });
        assert!(result.found());
        assert_eq!(result.latest(), Some(test_entry()));
    }

    #[test]
    fn test_get_entry_all_latest() {
        let mut result = GetEntryResult::new(StatusRequestKind::All, None);
        assert_eq!(result.latest(), None);
        result.push(&EntryWithMeta {
            entry: test_entry_a(),
            crud_status: CrudStatus::Modified,
            maybe_crud_link: None,
        });
        result.push(&EntryWithMeta {
            entry: test_entry_b(),
            crud_status: CrudStatus::Live,
            maybe_crud_link: None,
        });
        assert!(result.found());
        assert_eq!(result.latest(), Some(test_entry_b()));
    }

    #[test]
    fn test_clear() {
        let mut result = GetEntryResult::new(StatusRequestKind::All, None);
        result.push(&EntryWithMeta {
            entry: test_entry(),
            crud_status: CrudStatus::Live,
            maybe_crud_link: None,
        });
        assert!(result.found());
        result.clear();
        assert!(!result.found());

        result = GetEntryResult::new(StatusRequestKind::Initial, None);
        result.push(&EntryWithMeta {
            entry: test_entry(),
            crud_status: CrudStatus::Live,
            maybe_crud_link: None,
        });
        assert!(result.found());
        result.clear();
        assert!(!result.found());
    }
}
