use super::Dispatch;
use error::ZomeApiResult;
use holochain_core_types::entry::Entry;
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::commit_entry::{
    CommitEntryArgs, CommitEntryOptions, CommitEntryResult,
};

/// Attempts to commit an entry to the local source chain. The entry
/// will also be checked against the defined validation rules for that entry type.
/// If the entry type is defined as public, it will also be published to the DHT.
/// Returns either an address of the committed entry, or an error.
/// # Examples
/// ```rust
/// # extern crate hdk;
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate holochain_core_types;
/// # extern crate holochain_persistence_api;
/// # extern crate holochain_json_api;
/// # #[macro_use]
/// # extern crate holochain_json_derive;
/// # use hdk::error::ZomeApiResult;
/// # use holochain_json_api::json::JsonString;
/// # use holochain_json_api::error::JsonError;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::entry::entry_type::AppEntryType;
/// # use holochain_core_types::entry::Entry;
/// # use holochain_persistence_api::cas::content::Address;
/// # use holochain_core_types::error::RibosomeEncodingBits;
///
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
///
/// # fn main() {
///
/// #[derive(Serialize, Deserialize, Debug, DefaultJson)]
/// pub struct Post {
///     content: String,
///     date_created: String,
/// }
///
/// pub fn handle_create_post(content: String) -> ZomeApiResult<Address> {
///
///     let post_entry = Entry::App("post".into(), Post{
///         content,
///         date_created: "now".into(),
///     }.into());
///
///    let address = hdk::commit_entry(&post_entry)?;
///
///    Ok(address)
///
/// }
///
/// # }
/// ```
pub fn commit_entry(entry: &Entry) -> ZomeApiResult<Address> {
    commit_entry_result(entry, CommitEntryOptions::default()).map(|result| result.address())
}

/// Attempts to commit an entry to your local source chain. The entry
/// will have to pass the defined validation rules for that entry type.
/// If the entry type is defined as public, will also publish the entry to the DHT.
///
/// Additional provenances can be added to the commit using the options argument.
/// Returns a CommitEntryResult which contains the address of the committed entry.
pub fn commit_entry_result(
    entry: &Entry,
    options: CommitEntryOptions,
) -> ZomeApiResult<CommitEntryResult> {
    Dispatch::CommitEntry.with_input(CommitEntryArgs {
        entry: entry.clone(),
        options,
    })
}
