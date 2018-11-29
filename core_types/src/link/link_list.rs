use crate::{
    entry::{entry_type::EntryType, Entry, ToEntry},
    error::error::HolochainError,
    json::JsonString,
    link::Link,
};
use std::convert::TryInto;

//-------------------------------------------------------------------------------------------------
// LinkList
//-------------------------------------------------------------------------------------------------
//
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, DefaultJson)]
pub struct LinkList {
    links: Vec<Link>,
}

impl LinkList {
    pub fn new(links: &[Link]) -> Self {
        LinkList {
            links: links.to_vec(),
        }
    }

    pub fn links(&self) -> &Vec<Link> {
        &self.links
    }
}
