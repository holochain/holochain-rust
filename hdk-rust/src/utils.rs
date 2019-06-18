use crate::{
    self as hdk,
    error::{ZomeApiError, ZomeApiResult},
    holochain_core_types::{
        cas::content::Address,
        entry::{AppEntryValue, Entry},
    },
};
use holochain_core_types::link::LinkMatch;
use std::convert::TryFrom;

///
/// Helper function that perfoms a try_from for every entry
/// of a get_links_and_load for a given type. Any entries that either fail to
/// load or cannot be converted to the type will be dropped.
///
pub fn get_links_and_load_type<R: TryFrom<AppEntryValue>>(
    base: &Address,
    link_type: LinkMatch<&str>,
    tag: LinkMatch<&str>,
) -> ZomeApiResult<Vec<R>> {
    let link_load_results = hdk::get_links_and_load(base, link_type, tag)?;

    Ok(link_load_results
        .iter()
        .map(|maybe_entry| match maybe_entry {
            Ok(entry) => match entry {
                Entry::App(_, entry_value) => {
                    let typed_entry = R::try_from(entry_value.to_owned()).map_err(|_| {
                        ZomeApiError::Internal(
                            "Could not convert get_links result to requested type".to_string(),
                        )
                    })?;
                    Ok(typed_entry)
                }
                _ => Err(ZomeApiError::Internal(
                    "get_links did not return an app entry".to_string(),
                )),
            },
            _ => Err(ZomeApiError::Internal(
                "get_links did not return an app entry".to_string(),
            )),
        })
        .filter_map(Result::ok)
        .collect())
}

///
/// Helper function for loading an entry and converting to a given type
///
pub fn get_as_type<R: TryFrom<AppEntryValue>>(address: &Address) -> ZomeApiResult<R> {
    let get_result = hdk::get_entry(&address)?;
    let entry =
        get_result.ok_or_else(|| ZomeApiError::Internal("No entry at this address".into()))?;
    match entry {
        Entry::App(_, entry_value) => R::try_from(entry_value.to_owned()).map_err(|_| {
            ZomeApiError::Internal(
                "Could not convert get_links result to requested type".to_string(),
            )
        }),
        _ => Err(ZomeApiError::Internal(
            "get_links did not return an app entry".to_string(),
        )),
    }
}

/// Creates two links:
/// From A to B, and from B to A, with given link_types.
pub fn link_entries_bidir<
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
    S4: Into<String>,
>(
    a: &Address,
    b: &Address,
    link_type_a_b: S1,
    link_type_b_a: S2,
    link_tag_a_b: S3,
    link_tag_b_a: S4,
) -> ZomeApiResult<()> {
    hdk::link_entries(a, b, link_type_a_b, link_tag_a_b)?;
    hdk::link_entries(b, a, link_type_b_a, link_tag_b_a)?;
    Ok(())
}

/// Commits the given entry and links it from the base
/// with the given link_type.
pub fn commit_and_link<
    S1: Into<String>,
    S2: Into<String>,
>(
    entry: &Entry,
    base: &Address,
    link_type: S1,
    tag: S2,
) -> ZomeApiResult<Address> {
    let entry_addr = hdk::commit_entry(entry)?;
    hdk::link_entries(base, &entry_addr, link_type, tag)?;
    Ok(entry_addr)
}
