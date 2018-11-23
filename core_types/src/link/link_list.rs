use crate::{
    entry::{entry_type::EntryType, Entry, ToEntry},
    error::error::HolochainError,
    json::JsonString,
    link::Link,
};
use std::convert::TryInto;

//-------------------------------------------------------------------------------------------------
// LinkListEntry
//-------------------------------------------------------------------------------------------------
//
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, DefaultJson)]
pub struct LinkListEntry {
    links: Vec<Link>,
}

impl LinkListEntry {
    pub fn new(links: &[Link]) -> Self {
        LinkListEntry {
            links: links.to_vec(),
        }
    }

    pub fn links(&self) -> &Vec<Link> {
        &self.links
    }
}

impl ToEntry for LinkListEntry {
    // Convert a LinkListEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        Entry::new(EntryType::LinkList, self.to_owned())
    }

    fn from_entry(entry: &Entry) -> Self {
        assert_eq!(&EntryType::LinkList, entry.entry_type());
        entry
            .value()
            .to_owned()
            .try_into()
            .expect("could not convert Entry to LinkListEntry")
    }
}
