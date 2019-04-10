use crate::{
    agent::chain_store::{ChainStoreQueryOptions, ChainStoreQueryResult},
    context::Context,
    nucleus::{
        actions::get_entry::get_entry_from_agent,
        ribosome::{api::ZomeApiResult, Runtime},
    },
};
use holochain_core_types::{
    cas::content::Address, chain_header::ChainHeader, entry::Entry, error::HolochainError,
};
use holochain_wasm_utils::api_serialization::{QueryArgs, QueryArgsNames, QueryResult};
use std::{convert::TryFrom, sync::Arc};
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::query function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: ?
/// Returns an HcApiReturnCode as I64
///
/// Specify 0 or more simple or "glob" patterns matching EntryType names, returning Vec<Address>.
///
/// The empty String or an empty Vec matches all.  The '*' glob pattern matches all simple EntryType
/// names (with no '/'), while the ** pattern matches everything (use "" or [] for efficiency).
///
/// `[]`
/// `[""]`
/// `["**"]`
///
/// Namespaces (groups of related EntryType names) can be queried easily, eg:
///
/// `["name/*"]`
///
/// Several simple names and/or "glob" patterns can be supplied, and are efficiently
/// searched for in a single pass using a single efficient Regular Expression engine:
///
/// `["name/*", "and_another", "something_else"]`
///
/// EntryType names can be excluded, eg. to return every simple (non-namespaced) EntryType except System:
///
/// `["[!%]*"]`
///
/// To match a pattern, including all namespaced EntryType names, eg. every EntryType except System:
///
/// `["**/[!%]*"]`
///
/// The following standard "glob" patterns are supported:
///
/// Pattern     Match
/// =======     =====
/// `.`         One character (other than a '/')
/// `[abcd]`    One of a set of characters
/// `[a-d]`     One of a range of characters
/// `[!a-d]`    Not one of  range of characters
/// `{abc,123}` One of a number of sequences of characters
/// `*`         Zero or more of any character
/// `**/`       Zero or more of any namespace component
///
pub fn invoke_query(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args.
    let args_str = runtime.load_json_string_from_args(&args);
    let query = match QueryArgs::try_from(args_str) {
        Ok(input) => input,
        Err(..) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };

    // Perform query
    let agent = context.state().unwrap().agent();
    let top = agent
        .top_chain_header()
        .expect("Should have genesis entries.");
    let maybe_result = match query.entry_type_names {
        // Result<ChainStoreQueryResult,...>
        QueryArgsNames::QueryList(pats) => {
            let refs: Vec<&str> = pats.iter().map(AsRef::as_ref).collect(); // Vec<String> -> Vec<&str>
            agent.chain_store().query(
                &Some(top),
                refs.as_slice(), // Vec<&str> -> Vec[&str]
                ChainStoreQueryOptions {
                    start: query.options.start,
                    limit: query.options.limit,
                    headers: query.options.headers,
                },
            )
        }
        QueryArgsNames::QueryName(name) => {
            let refs: Vec<&str> = vec![&name]; // String -> Vec<&str>
            agent.chain_store().query(
                &Some(top),
                refs.as_slice(), // Vec<&str> -> &[&str]
                ChainStoreQueryOptions {
                    start: query.options.start,
                    limit: query.options.limit,
                    headers: query.options.headers,
                },
            )
        }
    };
    let result = match maybe_result {
        // TODO #793: the Err(_code) is the RibosomeErrorCode, but we can't import that type here.
        // Perhaps return chain_store().query should return Some(result)/None instead, and the fixed
        // UnknownEntryType code here, rather than trying to return a specific error code.
        Ok(result) => Ok(match (query.options.entries, result) {
            (false, ChainStoreQueryResult::Addresses(addresses)) => {
                QueryResult::Addresses(addresses)
            }
            (false, ChainStoreQueryResult::Headers(headers)) => QueryResult::Headers(headers),
            (true, ChainStoreQueryResult::Addresses(addresses)) => {
                let maybe_entries: Result<Vec<(Address, Entry)>, HolochainError> = addresses
                    .iter()
                    .map(|address| // -> Result<Entry, HolochainError>
                         Ok((address.to_owned(), get_entry_from_chain(&context, address)?)))
                    .collect();

                match maybe_entries {
                    Ok(entries) => QueryResult::Entries(entries),
                    Err(_e) => return ribosome_error_code!(UnknownEntryType), // TODO: return actual error?
                }
            }
            (true, ChainStoreQueryResult::Headers(headers)) => {
                let maybe_headers_with_entries: Result<Vec<(ChainHeader,Entry)>,HolochainError> = headers
                    .iter()
                    .map(|header| // -> Result<Entry, HolochainError>
                         Ok((header.to_owned(), get_entry_from_chain(&context,header.entry_address())?)))
                    .collect();
                match maybe_headers_with_entries {
                    Ok(headers_with_entries) => {
                        QueryResult::HeadersWithEntries(headers_with_entries)
                    }
                    Err(_e) => return ribosome_error_code!(UnknownEntryType), // TODO: return actual error?
                }
            }
        }),
        Err(_code) => return ribosome_error_code!(UnknownEntryType),
    };

    runtime.store_result(result)
}

/// Get an local-chain Entry via the provided context, returning Entry or HolochainError on failure
fn get_entry_from_chain(
    context: &Arc<Context>,
    address: &Address,
) -> Result<Entry, HolochainError> {
    get_entry_from_agent(context, address)?.ok_or_else(|| {
        HolochainError::ErrorGeneric(format!("Failed to obtain Entry for Address {}", address))
    })
}
