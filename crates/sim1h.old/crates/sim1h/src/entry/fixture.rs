use crate::agent::fixture::core_agent_id_fresh;
use crate::agent::fixture::provenances_fresh;
use crate::aspect::fixture::aspect_list_fresh;
use crate::network::fixture::timestamp_fresh;
use holochain_core_types::chain_header::ChainHeader;
use holochain_core_types::entry::Entry;
use holochain_persistence_api::cas::content::Address;
use holochain_persistence_api::cas::content::AddressableContent;
use lib3h_protocol::data_types::EntryData;
use lib3h_protocol::types::EntryHash;
use uuid::Uuid;

pub fn entry_hash_fresh() -> EntryHash {
    EntryHash::from(Uuid::new_v4().to_string())
}

pub fn entry_fresh() -> Entry {
    Entry::AgentId(core_agent_id_fresh())
}

pub fn header_address_fresh() -> Address {
    Uuid::new_v4().to_string().into()
}

pub fn chain_header_fresh(entry: &Entry) -> ChainHeader {
    ChainHeader::new(
        &entry.entry_type(),
        &entry.address(),
        &provenances_fresh(),
        &Some(header_address_fresh()),
        &Some(header_address_fresh()),
        &Some(header_address_fresh()),
        &timestamp_fresh(),
    )
}

pub fn entry_data_fresh(entry_hash: &EntryHash) -> EntryData {
    EntryData {
        entry_address: entry_hash.clone(),
        aspect_list: aspect_list_fresh(),
    }
}

pub fn link_tag_fresh() -> String {
    Uuid::new_v4().to_string()
}

pub fn link_type_fresh() -> String {
    Uuid::new_v4().to_string()
}
