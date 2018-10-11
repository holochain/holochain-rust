use cas::content::{Address, AddressableContent};
use entry::Entry;
use entry_type::EntryType;
use serde_json;
use entry::ToEntry;

//-------------------------------------------------------------------------------------------------
// Link
//-------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Link {
    base: Address,
    target: Address,
    tag: String,
}

impl Link {
    pub fn new(base: &Address, target: &Address, tag: &str) -> Self {
        Link {
            base: base.clone(),
            target: target.clone(),
            tag: tag.to_string(),
        }
    }
    // Key for HashTable
    pub fn key(&self) -> String {
        format!("link:{}:{}:{}", self.base, self.target, self.tag)
    }
    pub fn to_attribute_name(&self) -> String {
        format!("link:{}:{}", self.base, self.tag)
    }
    // Getters
    pub fn base(&self) -> &Address {
        &self.base
    }
    pub fn target(&self) -> &Address {
        &self.target
    }
    pub fn tag(&self) -> &String {
        &self.tag
    }
}
//-------------------------------------------------------------------------------------------------
// LinkEntry
//-------------------------------------------------------------------------------------------------

// HC.LinkAction sync with hdk-rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LinkActionKind {
    ADD,
    DELETE,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LinkEntry {
    action_kind: LinkActionKind,
    link: Link,
}

impl LinkEntry {
    pub fn new(action_kind: LinkActionKind, base: &Address, target: &Address, tag: &str) -> Self {
        LinkEntry {
            action_kind: action_kind,
            link: Link::new(base, target, tag),
        }
    }

    pub fn from_link(action_kind: LinkActionKind, link: &Link) -> Self {
        LinkEntry {
            action_kind: action_kind,
            link: link.clone(),
        }
    }
}
impl ToEntry for LinkEntry {
    // Convert a LinkEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        let json_array = serde_json::to_string(self).expect("LinkEntry should serialize");
        Entry::new(&EntryType::Link, &Entry::from(json_array))
    }

    fn from_entry(entry: &Entry) -> Self {
        serde_json::from_str(&entry.content()).expect("entry is not a valid LinkEntry")
    }
}
//-------------------------------------------------------------------------------------------------
// LinkListEntry
//-------------------------------------------------------------------------------------------------
//
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct LinkListEntry {
    pub links: Vec<Link>,
}

impl LinkListEntry {
    pub fn new(links: &[Link]) -> Self {
        LinkListEntry {
            links: links.to_vec(),
        }
    }
}

impl ToEntry for LinkListEntry {
    // Convert a LinkListEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        let json_array = serde_json::to_string(self).expect("LinkListEntry failed to serialize");
        Entry::new(&EntryType::LinkList, &Entry::from(json_array))
    }

    fn from_entry(entry: &Entry) -> Self {
        serde_json::from_str(&entry.content()).expect("entry failed converting into LinkListEntry")
    }
}
