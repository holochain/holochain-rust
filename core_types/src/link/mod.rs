//! This module contains definitions for what a Link is in Holochain, as well as
//! structs relating to the adding and removing of links between entries
//! and lists of links.

pub mod link_data;
pub mod link_list;

use crate::{cas::content::Address, error::HolochainError, json::JsonString};

type LinkTag = String;
type LinkType = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, DefaultJson)]
pub struct Link {
    base: Address,
    target: Address,
    tag: LinkTag,
    r#type: LinkType,
}

impl Link {
    pub fn new(base: &Address, target: &Address, tag: &str, r#type: &str) -> Self {
        Link {
            base: base.to_owned(),
            target: target.to_owned(),
            tag: tag.to_owned(),
            r#type: r#type.to_owned(),
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

    pub fn r#type(&self) -> &LinkType {
        &self.r#type
    }
}

// HC.LinkAction sync with hdk-rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LinkActionKind {
    ADD,
    REMOVE,
}

#[cfg(test)]
pub mod tests {

    use crate::{
        cas::content::AddressableContent,
        entry::{test_entry_a, test_entry_b},
        link::{Link, LinkActionKind, LinkTag, LinkType},
    };

    pub fn example_link_tag() -> LinkTag {
        LinkTag::from("foo-tag")
    }

    pub fn example_link_type() -> LinkType {
        LinkType::from("foo-type")
    }

    pub fn example_link() -> Link {
        Link::new(
            &test_entry_a().address(),
            &test_entry_b().address(),
            &example_link_tag(),
            &example_link_type(),
        )
    }

    pub fn example_link_action_kind() -> LinkActionKind {
        LinkActionKind::ADD
    }
}
