pub mod actions;
pub mod reducers;
pub mod handler;
pub mod state;

use holochain_core_types::{
    entry::SerializedEntry,
    chain_header::ChainHeader,
};


#[derive(Serialize, Deserialize)]
pub struct EntryWithHeader {
    entry: SerializedEntry,
    header: ChainHeader,
}