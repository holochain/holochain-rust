use super::Dispatch;
use error::{ZomeApiError, ZomeApiResult};
use holochain_core_types::entry::Entry;
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::get_entry::{
    EntryHistory, GetEntryArgs, GetEntryOptions, GetEntryResult, GetEntryResultType,
    StatusRequestKind,
};

/// Retrieves latest version of an entry from the local chain or the DHT, by looking it up using
/// the specified address.
/// Returns None if no entry exists at the specified address or
/// if the entry's status is DELETED.  Note that if the entry was updated, the value retrieved
/// may be of the updated entry which will have a different hash value.  If you need
/// to get the original value whatever the status, use [get_entry_initial](fn.get_entry_initial.html), or if you need to know
/// the address of the updated entry use [get_entry_result](fn.get_entry_result.html)
/// # Examples
/// ```rust
/// # extern crate hdk;
/// # extern crate holochain_core_types;
/// # extern crate holochain_persistence_api;
/// # extern crate holochain_json_api;
/// # use hdk::error::ZomeApiResult;
/// # use holochain_core_types::entry::Entry;
/// # use holochain_json_api::json::JsonString;
/// # use holochain_persistence_api::cas::content::Address;
/// # fn main() {
/// pub fn handle_get_post(post_address: Address) -> ZomeApiResult<Option<Entry>> {
///     // get_entry returns a Result<Option<T>, ZomeApiError>
///     // where T is the type that you used to commit the entry, in this case a Blog
///     // It's a ZomeApiError if something went wrong (i.e. wrong type in deserialization)
///     // Otherwise its a Some(T) or a None
///     hdk::get_entry(&post_address)
/// }
/// # }
/// ```
pub fn get_entry(address: &Address) -> ZomeApiResult<Option<Entry>> {
    let entry_result = get_entry_result(address, GetEntryOptions::default())?;

    let entry = if !entry_result.found() {
        None
    } else {
        entry_result.latest()
    };

    Ok(entry)
}

/// Returns the Entry at the exact address specified, whatever its status.
/// Returns None if no entry exists at the specified address.
pub fn get_entry_initial(address: &Address) -> ZomeApiResult<Option<Entry>> {
    let entry_result = get_entry_result(
        address,
        GetEntryOptions::new(StatusRequestKind::Initial, true, false, Default::default()),
    )?;
    Ok(entry_result.latest())
}

/// Return an EntryHistory filled with all the versions of the entry from the version at
/// the specified address to the latest.
/// Returns None if no entry exists at the specified address.
pub fn get_entry_history(address: &Address) -> ZomeApiResult<Option<EntryHistory>> {
    let entry_result = get_entry_result(
        address,
        GetEntryOptions::new(StatusRequestKind::All, true, false, Default::default()),
    )?;
    if !entry_result.found() {
        return Ok(None);
    }
    match entry_result.result {
        GetEntryResultType::All(history) => Ok(Some(history)),
        _ => Err(ZomeApiError::from("shouldn't happen".to_string())),
    }
}

/// Retrieves an entry and its metadata from the local chain or the DHT, by looking it up using
/// the specified address.
/// The data returned is configurable with the GetEntryOptions argument.
pub fn get_entry_result(
    address: &Address,
    options: GetEntryOptions,
) -> ZomeApiResult<GetEntryResult> {
    Dispatch::GetEntry.with_input(GetEntryArgs {
        address: address.clone(),
        options,
    })
}
