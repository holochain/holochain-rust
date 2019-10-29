use crate::{
    agent::fixture::agent_id_fresh, entry::fixture::entry_data_fresh,
    network::fixture::request_id_fresh,
};
use holochain_core_types::network::query::NetworkQuery;
use holochain_json_api::json::JsonString;
use holochain_persistence_api::cas::content::Address;
use lib3h_protocol::{
    data_types::{Opaque, ProvidedEntryData, QueryEntryData, SpaceData},
    types::EntryHash,
};

pub fn query_fresh(_entry_address: &Address) -> Opaque {
    let query = NetworkQuery::GetEntry;
    let json: JsonString = query.into();
    json.to_bytes().into()
}

pub fn query_entry_data_fresh(space_data: &SpaceData, entry_hash: &EntryHash) -> QueryEntryData {
    QueryEntryData {
        space_address: space_data.space_address.clone(),
        entry_address: entry_hash.clone(),
        request_id: request_id_fresh(),
        requester_agent_id: agent_id_fresh(),
        query: query_fresh(&entry_hash),
    }
}

pub fn provided_entry_data_fresh(
    space_data: &SpaceData,
    entry_hash: &EntryHash,
) -> ProvidedEntryData {
    ProvidedEntryData {
        space_address: space_data.space_address.clone(),
        provider_agent_id: agent_id_fresh(),
        entry: entry_data_fresh(entry_hash),
    }
}
