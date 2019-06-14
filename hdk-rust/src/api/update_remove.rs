use super::Dispatch;
use error::{ZomeApiError, ZomeApiResult};
use holochain_core_types::entry::Entry;
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::UpdateEntryArgs;

/// Commit an entry to your local source chain that "updates" a previous entry, meaning when getting
/// the previous entry, the updated entry will be returned.
/// `update_entry` sets the previous entry's status metadata to `Modified` and adds the updated
/// entry's address in the previous entry's metadata.
/// The updated entry will hold the previous entry's address in its header,
/// which will be used by validation routes.
pub fn update_entry(new_entry: Entry, address: &Address) -> ZomeApiResult<Address> {
    Dispatch::UpdateEntry.with_input(UpdateEntryArgs {
        new_entry,
        address: address.clone(),
    })
}

/// NOT YET AVAILABLE
pub fn update_agent() -> ZomeApiResult<Address> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Commit a DeletionEntry to your local source chain that marks an entry as 'deleted' by setting
/// its status metadata to `Deleted` and adding the DeleteEntry's address in the deleted entry's
/// metadata, which will be used by validation routes.
pub fn remove_entry(address: &Address) -> ZomeApiResult<Address> {
    Dispatch::RemoveEntry.with_input(address.to_owned())
}
