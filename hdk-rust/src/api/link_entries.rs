use holochain_wasm_utils::api_serialization::link_entries::LinkEntriesArgs;
use error::ZomeApiError;
use holochain_core_types::{
	cas::content::Address,
};
use super::Dispatch;

/// Adds a named, directed link between two entries on the DHT.
/// Consumes three values, two of which are the addresses of entries, and one of which is a string that defines a
/// relationship between them, called a `tag`. Later, lists of entries can be looked up by using [get_links](fn.get_links.html). Entries
/// can only be looked up in the direction from the `base`, which is the first argument, to the `target`.
/// # Examples
/// ```rust
/// # #![feature(try_from)]
/// # extern crate hdk;
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate holochain_core_types;
/// # #[macro_use]
/// # extern crate holochain_core_types_derive;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::entry::entry_type::AppEntryType;
/// # use holochain_core_types::entry::Entry;
/// # use holochain_core_types::cas::content::Address;
/// # use hdk::AGENT_ADDRESS;
/// # use hdk::error::ZomeApiResult;
/// # use hdk::holochain_wasm_utils::api_serialization::get_entry::GetEntryOptions;
/// # use hdk::holochain_wasm_utils::api_serialization::get_entry::StatusRequestKind;
/// # fn main() {
///
/// #[derive(Serialize, Deserialize, Debug, DefaultJson)]
/// pub struct Post {
///     content: String,
///     date_created: String,
/// }
///
/// pub fn handle_link_entries(content: String, in_reply_to: Option<Address>) -> ZomeApiResult<Address> {
///
///     let post_entry = Entry::App("post".into(), Post{
///             content,
///             date_created: "now".into(),
///     }.into());
///
///     let address = hdk::commit_entry(&post_entry)?;
///
///     hdk::link_entries(
///         &AGENT_ADDRESS,
///         &address,
///         "authored_posts",
///     )?;
///
///     if let Some(in_reply_to_address) = in_reply_to {
///         // return with Err if in_reply_to_address points to missing entry
///         hdk::get_entry_result(&in_reply_to_address, GetEntryOptions { status_request: StatusRequestKind::All, entry: false, headers: false, timeout: Default::default() })?;
///         hdk::link_entries(&in_reply_to_address, &address, "comments")?;
///     }
///
///     Ok(address)
///
/// }
/// # }
/// ```
pub fn link_entries<S: Into<String>>(
    base: &Address,
    target: &Address,
    tag: S,
) -> Result<Address, ZomeApiError> {
    Dispatch::LinkEntries.with_input(LinkEntriesArgs {
        base: base.clone(),
        target: target.clone(),
        tag: tag.into(),
    })
}
