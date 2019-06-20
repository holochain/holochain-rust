use holochain_core_types::{
    cas::content::Address, crud_status::CrudStatus, entry::EntryWithMetaAndHeader,
    error::HolochainError, json::JsonString,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum NetworkQuery {
    GetEntry,
    GetLinks(String, String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum NetworkQueryResult {
    Entry(Option<EntryWithMetaAndHeader>),
    Links(Vec<(Address, CrudStatus)>, String, String),
}
/*
#[cfg(test)]
pub mod tests {
    use super::*;
    fn test_roudtrip_network_query_get_entry_encoding() {
        let query = NetworkQuery::GetEntry;
        let encoded_query: Vec<u8> = query.into();
        let decoded_query: NetworkQuery = encoded_query.into();
        assert_eq!(query,decoded_query);
    }

    fn test_roudtrip_network_query_get_links_encoding() {
        let query = NetworkQuery::GetLinks("foo_link_type","foo_link_tag");
        let encoded_query: Vec<u8> = query.into();
        let decoded_query: NetworkQuery = encoded_query.into();
        assert_eq!(query,decoded_query);
    }

}
*/
