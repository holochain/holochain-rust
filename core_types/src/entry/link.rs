use cas::content::Address;
use entry::{entry_type::EntryType, Entry, ToEntry};
use serde_json;

//-------------------------------------------------------------------------------------------------
// Link
//-------------------------------------------------------------------------------------------------

pub type LinkTag = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Link {
    base: Address,
    target: Address,
    tag: LinkTag,
}

impl Link {
    pub fn new(base: &Address, target: &Address, tag: &str) -> Self {
        Link {
            base: base.to_owned(),
            target: target.to_owned(),
            tag: tag.to_owned(),
        }
    }

    // Getters
    pub fn base(&self) -> &Address {
        &self.base
    }

    pub fn target(&self) -> &Address {
        &self.target
    }

    pub fn tag(&self) -> &LinkTag {
        &self.tag
    }
}

//-------------------------------------------------------------------------------------------------
// LinkEntry
//-------------------------------------------------------------------------------------------------

// HC.LinkAction sync with hdk-rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LinkActionKind {
    ADD,
    DELETE,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

    pub fn action_kind(&self) -> &LinkActionKind {
        &self.action_kind
    }

    pub fn link(&self) -> &Link {
        &self.link
    }

    pub fn from_link(action_kind: LinkActionKind, link: &Link) -> Self {
        LinkEntry {
            action_kind: action_kind,
            link: link.clone(),
        }
    }
}

impl ToString for LinkEntry {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("LinkEntry failed to serialize")
    }
}

impl From<String> for LinkEntry {
    fn from(s: String) -> LinkEntry {
        serde_json::from_str(&s).expect("LinkEntry failed to deserialize")
    }
}

impl ToEntry for LinkEntry {
    // Convert a LinkEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        let json_array = serde_json::to_string(self).expect("LinkEntry should serialize");
        Entry::new(&EntryType::Link, &json_array)
    }

    fn from_entry(entry: &Entry) -> Self {
        assert_eq!(&EntryType::Link, entry.entry_type());
        serde_json::from_str(&entry.value().to_owned()).expect("entry is not a valid LinkEntry")
    }
}
