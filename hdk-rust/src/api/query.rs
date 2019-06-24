use super::Dispatch;
use error::{ZomeApiError, ZomeApiResult};
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::{
    QueryArgs, QueryArgsNames, QueryArgsOptions, QueryResult,
};

/// Returns a list of entries from your local source chain that match a given entry type name or names.
///
/// Each name may be a plain entry type name, or a `"glob"` pattern.  All names and patterns are
/// merged into a single efficient Regular Expression for scanning.
///
/// You can select many names with patterns such as `"boo*"` (match all entry types starting with
/// `"boo"`), or `"[!%]*e"` (all non-system non-name-spaced entry types ending in `"e"`).
///
/// You can organize your entry types using simple name-spaces, by including `"/"` in your entry type
/// names.  For example, if you have several entry types related to fizzing a widget, you might
/// create entry types `"fizz/bar"`, `"fizz/baz"`, `"fizz/qux/foo"` and `"fizz/qux/boo"`.  Query for
/// `"fizz/**"` to match them all.
///
/// Use vec![], `""`, or `"**"` to match all names in all name-spaces.  Matching `"*"` will match only
/// non-namespaced names.
///
/// entry_type_names: Specify type of entry(s) to retrieve, as a String or Vec<String> of 0 or more names, converted into the QueryArgNames type
/// start: First entry in result list to retrieve
/// limit: Max number of entries to retrieve
/// # Examples
/// ```rust
/// # extern crate hdk;
/// # extern crate holochain_core_types;
/// # extern crate holochain_persistence_api;
/// # extern crate holochain_json_api;
/// # use hdk::error::ZomeApiResult;
/// # use holochain_json_api::json::JsonString;
/// # use holochain_persistence_api::cas::content::Address;
///
/// # fn main() {
/// pub fn handle_my_posts_as_commited() -> ZomeApiResult<Vec<Address>> {
///     hdk::query("post".into(), 0, 0)
/// }
/// pub fn all_system_plus_mine() -> ZomeApiResult<Vec<Address>> {
///     hdk::query(vec!["[%]*","mine"].into(), 0, 0)
/// }
/// pub fn everything_including_namespaced_except_system() -> ZomeApiResult<Vec<Address>> {
///     hdk::query("**/[!%]*".into(), 0, 0)
/// }
/// # }
/// ```
///
/// With hdk::query_result, you can specify a package of QueryArgsOptions, and get a
/// variety of return values, such a vector of Headers as a `Vec<ChainHeader>`:
///
/// ```
/// // pub fn get_post_headers() -> ZomeApiResult<QueryResult> {
/// //    hdk::query_result("post".into(), QueryArgsOptions{ headers: true, ..Default::default()})
/// // }
/// ```
///
/// The types of the results available depend on whether `headers` and/or `entries` is set:
///
/// ```
/// //                                                     // headers  entries
/// // pub enum QueryResult {                              // -------  -------
/// //     Addresses(Vec<Address>),                        // false    false
/// //     Headers(Vec<ChainHeader>),                      // true     false
/// //     Entries(Vec<(Address, Entry)>),                 // false    true
/// //     HeadersWithEntries(Vec<(ChainHeader, Entry)>),  // true     true
/// // }
/// ```
pub fn query(
    entry_type_names: QueryArgsNames,
    start: usize,
    limit: usize,
) -> ZomeApiResult<Vec<Address>> {
    // The hdk::query API always returns a simple Vec<Address>
    query_result(
        entry_type_names,
        QueryArgsOptions {
            start,
            limit,
            headers: false,
            entries: false,
        },
    )
    .and_then(|result| match result {
        QueryResult::Addresses(addresses) => Ok(addresses),
        _ => Err(ZomeApiError::FunctionNotImplemented), // should never occur
    })
}

pub fn query_result(
    entry_type_names: QueryArgsNames,
    options: QueryArgsOptions,
) -> ZomeApiResult<QueryResult> {
    Dispatch::Query.with_input(QueryArgs {
        entry_type_names,
        options,
    })
}
