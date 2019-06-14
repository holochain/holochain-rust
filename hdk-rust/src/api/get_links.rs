use super::Dispatch;
use api::get_entry::get_entry_result;
use error::{ZomeApiError, ZomeApiResult};
use holochain_core_types::{
    cas::content::Address, entry::Entry, link::LinkMatch,
};
use holochain_wasm_utils::api_serialization::{
    get_entry::{GetEntryOptions, GetEntryResult, GetEntryResultItem, GetEntryResultType},
    get_links::{GetLinksArgs, GetLinksOptions, GetLinksResult},
};

/// Consumes four values; the address of an entry get get links from (the base), the type of the links
/// to be retrieved, an optional tag to match, and an options struct for selecting what meta data and crud status links to retrieve.
/// Note: the type is intended to describe the relationship between the `base` and other entries you wish to lookup.
/// This function returns a list of addresses of other entries which matched as being linked by the given `type`. If the `tag` is not None
/// it will return only links that match the tag exactly. If the tag parameter is None it will return all links of the given type
/// regardless of their tag.
/// Links are created using the Zome API function [link_entries](fn.link_entries.html).
/// If you also need the content of the entry consider using one of the helper functions:
/// [get_links_result](fn.get_links_result) or [get_links_and_load](fn._get_links_and_load)
/// # Examples
/// ```rust
/// # extern crate hdk;
/// # extern crate holochain_core_types;
/// # extern crate holochain_wasm_utils;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::cas::content::Address;
/// # use hdk::error::ZomeApiResult;
/// # use holochain_wasm_utils::api_serialization::get_links::{GetLinksResult, GetLinksOptions};
/// # use holochain_core_types::link::LinkMatch;
///
/// # fn main() {
/// pub fn handle_posts_by_agent(agent: Address) -> ZomeApiResult<GetLinksResult> {
///     hdk::get_links_with_options(&agent, LinkMatch::Exactly("authored_posts"), LinkMatch::Any, GetLinksOptions::default())
/// }
/// # }
/// ```
pub fn get_links_with_options<T1: Into<LinkMatch<String>>, T2: Into<LinkMatch<String>>>(
    base: &Address,
    link_type: T1,
    tag: T2,
    options: &GetLinksOptions,
) -> ZomeApiResult<GetLinksResult> {
    let type_re = link_type.into().to_regex_string()?;
    let tag_re = tag.into().to_regex_string()?;

    Dispatch::GetLinks.with_input(GetLinksArgs {
        entry_address: base.clone(),
        link_type: type_re,
        tag: tag_re,
        options: options.to_owned(),
    })
}

/// Helper function for get_links. Returns a vector with the default return results.
pub fn get_links<T1: Into<LinkMatch<String>>, T2: Into<LinkMatch<String>>>(
    base: &Address,
    link_type: T1,
    tag: T2,
) -> ZomeApiResult<GetLinksResult> {
    get_links_with_options(
        base,
        link_type.into(),
        tag.into(),
        &GetLinksOptions::default(),
    )
}

/// Retrieves data about entries linked to a base address with a given type and tag. This is the most general version of the various get_links
/// helpers (such as get_links_and_load) and can return the linked addresses, entries, headers and sources. Also supports CRUD status_request.
/// The data returned is configurable with the GetLinksOptions to specify links options and GetEntryOptions argument wto specify options when loading the entries.
/// # Examples
/// ```rust
/// # extern crate hdk;
/// # extern crate holochain_core_types;
/// # extern crate holochain_wasm_utils;
/// # use hdk::error::ZomeApiResult;
/// # use holochain_core_types::cas::content::Address;
/// # use holochain_wasm_utils::api_serialization::{
/// #    get_entry::{GetEntryOptions, GetEntryResult},
/// #    get_links::GetLinksOptions};
/// # use holochain_core_types::link::LinkMatch;
///
/// # fn main() {
/// fn hangle_get_links_result(address: Address) -> ZomeApiResult<Vec<ZomeApiResult<GetEntryResult>>> {
///    hdk::get_links_result(&address, LinkMatch::Exactly("test-link"), LinkMatch::Any, GetLinksOptions::default(), GetEntryOptions::default())
/// }
/// # }
/// ```
pub fn get_links_result<T1: Into<LinkMatch<String>>, T2: Into<LinkMatch<String>>>(
    base: &Address,
    link_type: T1,
    tag: T2,
    options: &GetLinksOptions,
    get_entry_options: &GetEntryOptions,
) -> ZomeApiResult<Vec<ZomeApiResult<GetEntryResult>>> {
    let get_links_result = get_links_with_options(base, link_type, tag, options)?;
    let result = get_links_result
        .addresses()
        .iter()
        .map(|address| get_entry_result(&address, &get_entry_options))
        .collect();
    Ok(result)
}

/// Helper function for get_links. Returns a vector of the entries themselves
pub fn get_links_and_load<T1: Into<LinkMatch<String>>, T2: Into<LinkMatch<String>>>(
    base: &Address,
    link_type: T1,
    tag: T2,
) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
    let get_links_result = get_links_result(
        base,
        link_type,
        tag,
        &GetLinksOptions::default(),
        &GetEntryOptions::default(),
    )?;

    let entries = get_links_result
    .into_iter()
    .map(|get_result| {
        let get_type = get_result?.result;
        match get_type {
            GetEntryResultType::Single(GetEntryResultItem{entry: Some(entry), ..}) => Ok(entry),
            GetEntryResultType::Single(GetEntryResultItem{entry: None, ..}) => Err(ZomeApiError::Internal("Entry is None so most likely has been marked as deleted".to_string())),
            GetEntryResultType::All(_) => Err(ZomeApiError::Internal("Invalid response. get_links_result returned all entries when latest was requested".to_string())),
        }
    })
    .collect();

    Ok(entries)
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use holochain_core_types::{
        link::LinkMatch,
    };

    #[test]
    fn can_call_with_string_variations() {
        get_links(&Address::from("x"), LinkMatch::Exactly(""), LinkMatch::Any);
    }

}
