use super::Dispatch;
use error::ZomeApiError;
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::link_entries::LinkEntriesArgs;

/// Commits a LinkRemove entry to your local source chain that marks a link as 'deleted' by setting
/// its status metadata to `Deleted` which gets published to the DHT.
/// Consumes four values, two of which are the addresses of entries, and two of which are strings that describe the link
/// type and its tag. Both must match exactly to remove a link.
/// Before a RemoveLink is executed, a get_links will have to make sure that we are deleting the right links
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
/// pub fn handle_remove_link(content: String, in_reply_to: Option<Address>) -> ZomeApiResult<()> {
///
///     let post_entry = Entry::App("post".into(), Post{
///             content,
///             date_created: "now".into(),
///     }.into());
///
///     let address = hdk::commit_entry(&post_entry)?;
///
///     hdk::remove_link(
///         &AGENT_ADDRESS,
///         &address,
///         "authored_posts",
///         "test-tag"
///     )?;
///
///
///     Ok(())
///
/// }
/// # }
/// ```
pub fn remove_link<S: Into<String>>(
    base: &Address,
    target: &Address,
    link_type: S,
    tag: S,
) -> Result<(), ZomeApiError> {
    Dispatch::RemoveLink.with_input(LinkEntriesArgs {
        base: base.clone(),
        target: target.clone(),
        link_type: link_type.into(),
        tag: tag.into(),
    })
}
