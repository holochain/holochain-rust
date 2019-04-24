use crate::api_serialization::get_entry::{StatusRequestKind,GetEntryArgs,GetEntryResult,GetEntryResultType};

use holochain_core_types::{
    cas::content::Address, chain_header::ChainHeader, error::HolochainError, json::*, time::Timeout,
};

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash, DefaultJson)]
pub struct GetLinksArgs {
    pub entry_address: Address,
    pub tag: String,
    pub options: GetLinksOptions,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone, PartialEq, Eq, Hash)]
pub enum LinksStatusRequestKind {
    Live,
    Deleted,
    All,
}
impl Default for LinksStatusRequestKind {
    fn default() -> Self {
        LinksStatusRequestKind::Live
    }
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone, PartialEq, Hash, Eq)]
pub struct GetLinksOptions {
    pub status_request: LinksStatusRequestKind,
    pub link_status_request : StatusRequestKind,
    pub headers: bool,
    pub timeout: Timeout,
}
impl Default for GetLinksOptions {
    fn default() -> Self {
        GetLinksOptions {
            status_request: LinksStatusRequestKind::default(),
            link_status_request : StatusRequestKind::default(),
            headers: false,
            timeout: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, DefaultJson)]
pub struct LinksResult {
    pub link : GetEntryResult
}

#[derive(Deserialize, Serialize, Debug, DefaultJson)]
pub struct GetLinksResult {
    links: Vec<LinksResult>,
}

impl GetLinksResult {
    pub fn new(links: Vec<LinksResult>) -> GetLinksResult {
        GetLinksResult { links }
    }

    pub fn addresses(self) -> Vec<Address>
    {
        let links = self
                    .links
                    .iter()
                    .map(|s|{
                        match s.link.result
                        {
                        GetEntryResultType::Single(ref single) => vec![single.clone().meta.map(|s|s.address).unwrap_or_default()],
                        GetEntryResultType::All(ref multiple) => multiple.items.iter().map(|m|{
                            m.clone().meta.map(|x|x.address).unwrap_or_default()
                        }).collect()
                        }
                    })
                    .flatten()
                    .filter(|s| !s.to_string().is_empty())
                    .collect();
        links

    }

}
