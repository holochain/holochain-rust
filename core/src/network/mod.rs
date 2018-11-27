pub mod actions;
pub mod handler;
pub mod reducers;
pub mod state;

use holochain_core_types::{chain_header::ChainHeader, entry::SerializedEntry};

#[derive(Serialize, Deserialize)]
pub struct EntryWithHeader {
    entry: SerializedEntry,
    header: ChainHeader,
}
