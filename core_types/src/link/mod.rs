//! This module contains definitions for what a Link is in Holochain, as well as
//! structs relating to the adding and removing of links between entries
//! and lists of links.

pub mod link_data;
pub mod link_list;

use crate::{
    agent::AgentId, cas::content::Address, chain_header::ChainHeader, error::HolochainError,
    json::JsonString,
};
use entry::Entry;
use link::link_data::LinkData;
use regex::Regex;

type LinkType = String;
type LinkTag = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, DefaultJson)]
pub struct Link {
    base: Address,
    target: Address,
    link_type: LinkType,
    tag: LinkTag,
}

impl Link {
    pub fn new(base: &Address, target: &Address, link_type: &str, tag: &str) -> Self {
        Link {
            base: base.to_owned(),
            target: target.to_owned(),
            link_type: link_type.to_owned(),
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

    pub fn link_type(&self) -> &LinkType {
        &self.link_type
    }

    pub fn tag(&self) -> &LinkTag {
        &self.tag
    }

    pub fn add_entry(&self, top_chain_header: ChainHeader, agent_id: AgentId) -> Entry {
        Entry::LinkAdd(LinkData::add_from_link(self, top_chain_header, agent_id))
    }

    pub fn remove_entry(&self, top_chain_header: ChainHeader, agent_id: AgentId) -> Entry {
        Entry::LinkAdd(LinkData::remove_from_link(self, top_chain_header, agent_id))
    }
}

// HC.LinkAction sync with hdk-rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LinkActionKind {
    ADD,
    REMOVE,
}

pub enum LinkMatch<S: Into<String>> {
    Any,
    Exactly(S),
    Regex(S),
}
 
impl<S: Into<String>> LinkMatch<S> {
    pub fn to_regex_string(self) -> Result<String, String> {
        let re_string: String = match self {
            LinkMatch::Any => ".*".into(),
            LinkMatch::Exactly(s) => "^".to_owned() + &regex::escape(&s.into()) + "$",
            LinkMatch::Regex(s) => s.into(),
        };
        // check that it is a valid regex
        match Regex::new(&re_string) {
            Ok(_) => Ok(re_string),
            Err(_) => Err("Invalid regex passed to get_links".into()),
        }
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{
        cas::content::AddressableContent,
        entry::{test_entry_a, test_entry_b},
        link::{Link, LinkActionKind, LinkTag, LinkType},
    };

    pub fn example_link_type() -> LinkType {
        LinkType::from("foo-link-type")
    }

    pub fn example_link_tag() -> LinkTag {
        LinkTag::from("foo-link-tag")
    }

    pub fn example_link() -> Link {
        Link::new(
            &test_entry_a().address(),
            &test_entry_b().address(),
            &example_link_type(),
            &example_link_tag(),
        )
    }

    pub fn example_link_action_kind() -> LinkActionKind {
        LinkActionKind::ADD
    }
}
