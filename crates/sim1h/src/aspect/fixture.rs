use crate::{
    agent::fixture::core_agent_id_fresh,
    entry::fixture::{
        chain_header_fresh, entry_fresh, entry_hash_fresh, link_tag_fresh, link_type_fresh,
    },
};
use holochain_core_types::{
    entry::Entry, link::link_data::LinkData, network::entry_aspect::EntryAspect,
};
use holochain_json_api::json::JsonString;
use holochain_persistence_api::cas::content::AddressableContent;
use lib3h_protocol::{
    data_types::{EntryAspectData, Opaque},
    types::AspectHash,
};
use uuid::Uuid;

pub fn link_add_aspect_fresh(entry: &Entry) -> EntryAspect {
    let link_data = LinkData::new_add(
        &entry.address(),
        &entry_hash_fresh(),
        &link_tag_fresh(),
        &link_type_fresh(),
        chain_header_fresh(&entry_fresh()),
        core_agent_id_fresh(),
    );
    EntryAspect::LinkAdd(link_data, chain_header_fresh(entry))
}

pub fn link_remove_aspect_fresh(entry: &Entry) -> EntryAspect {
    let link_data = LinkData::new_delete(
        &entry.address(),
        &entry_hash_fresh(),
        &link_tag_fresh(),
        &link_type_fresh(),
        chain_header_fresh(&entry_fresh()),
        core_agent_id_fresh(),
    );
    EntryAspect::LinkRemove((link_data, Vec::new()), chain_header_fresh(entry))
}

pub fn update_aspect_fresh(entry: &Entry) -> EntryAspect {
    EntryAspect::Update(entry.clone(), chain_header_fresh(&entry))
}

pub fn deletion_aspect_fresh(entry: &Entry) -> EntryAspect {
    EntryAspect::Deletion(chain_header_fresh(&entry))
}

pub fn content_aspect_fresh() -> EntryAspect {
    let entry = entry_fresh();
    EntryAspect::Content(entry.clone(), chain_header_fresh(&entry))
}

pub fn header_aspect_fresh(entry: &Entry) -> EntryAspect {
    EntryAspect::Header(chain_header_fresh(entry))
}

pub fn entry_aspect_data_fresh() -> EntryAspectData {
    EntryAspectData {
        aspect_address: aspect_hash_fresh(),
        type_hint: type_hint_fresh(),
        aspect: opaque_aspect_fresh(),
        publish_ts: publish_ts_fresh(),
    }
}

pub fn aspect_list_fresh() -> Vec<EntryAspectData> {
    let mut aspects = Vec::new();

    for _ in 0..10 {
        aspects.push(entry_aspect_data_fresh())
    }

    aspects
}

pub fn opaque_aspect_fresh() -> Opaque {
    JsonString::from(content_aspect_fresh()).to_bytes().into()
}

pub fn aspect_hash_fresh() -> AspectHash {
    AspectHash::from(Uuid::new_v4().to_string())
}

pub fn type_hint_fresh() -> String {
    "content".to_string()
}

pub fn publish_ts_fresh() -> u64 {
    1_568_858_140
}
