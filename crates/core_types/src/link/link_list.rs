use crate::link::Link;
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_json_derive::DefaultJson;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------
// LinkList
//-------------------------------------------------------------------------------------------------
//
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, DefaultJson, Eq)]
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
