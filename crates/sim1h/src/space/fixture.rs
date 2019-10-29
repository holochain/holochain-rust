use crate::{agent::fixture::agent_id_fresh, network::fixture::request_id_fresh};
use holochain_persistence_api::cas::content::Address;
use lib3h_protocol::{data_types::SpaceData, types::SpaceHash};
use uuid::Uuid;

pub fn space_address_fresh() -> SpaceHash {
    Address::from(Uuid::new_v4().to_string()).into()
}

pub fn space_data_fresh() -> SpaceData {
    SpaceData {
        request_id: request_id_fresh(),
        space_address: space_address_fresh(),
        agent_id: agent_id_fresh(),
    }
}
