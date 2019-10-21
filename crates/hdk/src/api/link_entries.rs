use super::Dispatch;
use error::ZomeApiError;
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::link_entries::LinkEntriesArgs;

/// Adds a named, tagged, directed link between two entries on the DHT.
/// Consumes four values, two of which are the addresses of entries, and two of which are strings used to describe the link.
///
/// The first is the `link_type`. This is analogous to the entry_type and determines which validation callback will be run. The link type must match
/// a type already defined in the DNA using the link!, to! or from! macros.
///
/// The second is the `tag`. This can be any arbitrary string. This will be passed to the validation callback allowing the hApp developer to control what constitutes a valid tag.
///
/// Later, lists of entries can be looked up by using [get_links](fn.get_links.html). Entries
/// can only be looked up in the direction from the `base`, which is the first argument, to the `target`.
/// It is possible to retrieve links that exactly match a particular tag and type or return all links for a given type (along with their tag string).

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
/// # use holochain_json_api::json::JsonString;
/// # use holochain_json_api::error::JsonError;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::entry::entry_type::AppEntryType;
/// # use holochain_core_types::entry::Entry;
/// # use holochain_persistence_api::cas::content::Address;
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
///         ""
///     )?;
///
///     if let Some(in_reply_to_address) = in_reply_to {
///         // return with Err if in_reply_to_address points to missing entry
///         hdk::get_entry_result(&in_reply_to_address, GetEntryOptions { status_request: StatusRequestKind::All, entry: false, headers: false, timeout: Default::default() })?;
///         hdk::link_entries(&in_reply_to_address, &address, "comments", "")?;
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
    link_type: S,
    tag: S,
) -> Result<Address, ZomeApiError> {
    Dispatch::LinkEntries.with_input(LinkEntriesArgs {
        base: base.clone(),
        target: target.clone(),
        link_type: link_type.into(),
        tag: tag.into(),
    })
}
