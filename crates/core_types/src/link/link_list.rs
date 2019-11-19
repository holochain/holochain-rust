use crate::link::Link;
use holochain_json_api::{error::JsonError, json::JsonString};
use std::fmt;
use dump_vec::DumpVec;

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

impl fmt::Display for LinkList {
    fn fmt(&self, f: &mut fmt::formatter) -> fmt::Result {
        let dump_vec_links = DumpVec(*self.links);
        write!(f, "Link list: {}", dump_vec_links)
    }
}
