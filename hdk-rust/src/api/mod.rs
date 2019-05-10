//! developers! Detailed references and examples can be found here for how to use the
//! HDK exposed functions to access powerful Holochain functions.

use crate::error::{ZomeApiError, ZomeApiResult};
use holochain_core_types::{
    cas::content::Address,
    dna::capabilities::CapabilityRequest,
    entry::{
        Entry,
    },
    error::{RibosomeEncodedAllocation, RibosomeEncodingBits, ZomeApiInternalResult},
    signature::Provenance,
};
pub use holochain_wasm_utils::api_serialization::validation::*;
use holochain_wasm_utils::{
    api_serialization::{
        verify_signature::VerifySignatureArgs,
        QueryArgs, QueryArgsNames, QueryArgsOptions, QueryResult, UpdateEntryArgs, ZomeApiGlobals,
    },
    holochain_core_types::{
        hash::HashString,
        json::{JsonString, RawString},
    },
    memory::{ribosome::load_ribosome_encoded_json, stack::WasmStack},
};
use init_globals::init_globals;
use serde_json;
use std::{
    convert::{TryFrom, TryInto},
    time::Duration,
};

mod call;
mod debug;
mod commit_entry;
mod get_entry;
mod link_entries;
mod remove_link;
mod keystore;
mod sign;
mod get_links;
mod send;
mod capability;

pub use self::{
    call::call,
    debug::debug,
    commit_entry::{commit_entry, commit_entry_result},
    get_entry::{get_entry, get_entry_initial, get_entry_history, get_entry_result},
    link_entries::link_entries,
    remove_link::remove_link,
    keystore::{
        keystore_list,
        keystore_new_random,
        keystore_derive_seed,
        keystore_derive_key,
        keystore_sign,
        keystore_get_public_key,
    },
    sign::{sign, sign_one_time},
    get_links::{get_links, get_links_with_options, get_links_result, get_links_and_load},
    send::send,
    capability::{commit_capability_grant, commit_capability_claim},
};

macro_rules! def_api_fns {
    (
        $(
            $function_name:ident, $enum_variant:ident ;
        )*
    ) => {

        pub enum Dispatch {
            $( $enum_variant ),*
        }

        impl Dispatch {

            pub fn without_input<O: TryFrom<JsonString> + Into<JsonString>>(
                &self,
            ) -> ZomeApiResult<O> {
                self.with_input(JsonString::empty_object())
            }

            pub fn with_input<I: TryInto<JsonString>, O: TryFrom<JsonString>>(
                &self,
                input: I,
            ) -> ZomeApiResult<O> {
                let mut mem_stack = unsafe { G_MEM_STACK }
                .ok_or_else(|| ZomeApiError::Internal("debug failed to load mem_stack".to_string()))?;

                let wasm_allocation = mem_stack.write_json(input)?;

                // Call Ribosome's function
                let encoded_input: RibosomeEncodingBits =
                    RibosomeEncodedAllocation::from(wasm_allocation).into();
                let encoded_output: RibosomeEncodingBits = unsafe {
                    (match self {
                        $(Dispatch::$enum_variant => $function_name),*
                    })(encoded_input)
                };

                let result: ZomeApiInternalResult =
                    load_ribosome_encoded_json(encoded_output).or_else(|e| {
                        mem_stack.deallocate(wasm_allocation)?;
                        Err(ZomeApiError::from(e))
                    })?;

                // Free result & input allocations
                mem_stack.deallocate(wasm_allocation)?;

                // Done
                if result.ok {
                    JsonString::from_json(&result.value)
                        .try_into()
                        .map_err(|_| ZomeApiError::from(String::from("Failed to deserialize return value")))
                } else {
                    Err(ZomeApiError::from(result.error))
                }
            }
        }

        // Invokable functions in the Ribosome
        // WARNING Names must be in sync with ZomeAPIFunction in holochain-rust
        // WARNING All these fns need to be defined in wasms too @see the hdk integration_test.rs
        #[allow(dead_code)]
        extern "C" {
            pub(crate) fn hc_property(_: RibosomeEncodingBits) -> RibosomeEncodingBits;
            pub(crate) fn hc_start_bundle(_: RibosomeEncodingBits) -> RibosomeEncodingBits;
            pub(crate) fn hc_close_bundle(_: RibosomeEncodingBits) -> RibosomeEncodingBits;
            $( pub(crate) fn $function_name (_: RibosomeEncodingBits) -> RibosomeEncodingBits;) *
        }

        /// Add stubs for all core API functions when compiled in test mode.
        /// This makes it possible to actually build test executable from zome projects to run unit tests
        /// on zome functions (though: without being able to actually test integration with core - that is
        /// what we need holochain-nodejs for).
        ///
        /// Without these stubs we would have unresolved references since the API functions are
        /// provided by the Ribosome runtime.
        ///
        /// Attention:
        /// We need to make sure to only add these function stubs when compiling tests
        /// BUT NOT when building to a WASM binary to be run in a Holochain instance.
        /// Hence the `#[cfg(test)]` which is really important!
        #[cfg(test)]
        mod tests {
            use crate::holochain_core_types::error::{RibosomeEncodedValue, RibosomeEncodingBits};

            $( #[no_mangle]
                 pub fn $function_name(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
                     RibosomeEncodedValue::Success.into()
                 }) *
        }

    };

}

def_api_fns! {
    hc_init_globals, InitGlobals;
    hc_commit_entry, CommitEntry;
    hc_get_entry, GetEntry;
    hc_entry_address, EntryAddress;
    hc_query, Query;
    hc_update_entry, UpdateEntry;
    hc_remove_entry, RemoveEntry;
    hc_send, Send;
    hc_debug, Debug;
    hc_call, Call;
    hc_sign, Sign;
    hc_sign_one_time, SignOneTime;
    hc_verify_signature, VerifySignature;
    hc_link_entries, LinkEntries;
    hc_remove_link, RemoveLink;
    hc_get_links, GetLinks;
    hc_sleep, Sleep;
    hc_keystore_list, KeystoreList;
    hc_keystore_new_random, KeystoreNewRandom;
    hc_keystore_derive_seed, KeystoreDeriveSeed;
    hc_keystore_derive_key, KeystoreDeriveKey;
    hc_keystore_sign, KeystoreSign;
    hc_keystore_get_public_key, KeystoreGetPublicKey;
    hc_commit_capability_grant, CommitCapabilityGrant;
    hc_commit_capability_claim, CommitCapabilityClaim;
}

//--------------------------------------------------------------------------------------------------
// ZOME API GLOBAL VARIABLES
//--------------------------------------------------------------------------------------------------

/// Internal global for memory usage
pub static mut G_MEM_STACK: Option<WasmStack> = None;

lazy_static! {
    /// Internal global for retrieving all Zome API globals
    pub(crate) static ref GLOBALS: ZomeApiGlobals = init_globals().unwrap();

    /// The `name` property as taken from the DNA.
    pub static ref DNA_NAME: &'static str = &GLOBALS.dna_name;

    /// The address of the DNA the Zome is embedded within.
    /// This is often useful as a fixed value that is known by all
    /// participants running the DNA.
    pub static ref DNA_ADDRESS: &'static Address = &GLOBALS.dna_address;

    /// The identity string used when the chain was first initialized.
    pub static ref AGENT_ID_STR: &'static str = &GLOBALS.agent_id_str;

    /// The hash of your public key.
    /// This is your node address on the DHT.
    /// It can be used for node-to-node messaging with `send` and `receive` functions.
    pub static ref AGENT_ADDRESS: &'static Address = &GLOBALS.agent_address;

    /// The hash of the first identity entry on your chain (The second entry on your chain).
    /// This is your peer's identity on the DHT.
    pub static ref AGENT_INITIAL_HASH: &'static HashString = &GLOBALS.agent_initial_hash;

    #[doc(hidden)]
    /// The hash of the most recent identity entry that has been committed to your chain.
    /// Starts with the same value as AGENT_INITIAL_HASH.
    /// After a call to `update_agent` it will have the value of the hash of the newly committed identity entry.
    pub static ref AGENT_LATEST_HASH: &'static HashString = &GLOBALS.agent_latest_hash;

    /// The Address of the public token (if any)
    pub static ref PUBLIC_TOKEN: &'static Address = &GLOBALS.public_token;

    /// The CapabilityRequest under which this wasm function is executing
    pub static ref CAPABILITY_REQ: &'static CapabilityRequest = &GLOBALS.cap_request;

}

impl From<DNA_NAME> for JsonString {
    fn from(dna_name: DNA_NAME) -> JsonString {
        JsonString::from(RawString::from(dna_name.to_string()))
    }
}

impl From<DNA_ADDRESS> for JsonString {
    fn from(dna_address: DNA_ADDRESS) -> JsonString {
        JsonString::from(HashString::from(dna_address.to_string()))
    }
}

impl From<AGENT_ID_STR> for JsonString {
    fn from(agent_id: AGENT_ID_STR) -> JsonString {
        JsonString::from(RawString::from(agent_id.to_string()))
    }
}

impl From<AGENT_ADDRESS> for JsonString {
    fn from(agent_address: AGENT_ADDRESS) -> JsonString {
        JsonString::from(Address::from(agent_address.to_string()))
    }
}

impl From<AGENT_INITIAL_HASH> for JsonString {
    fn from(agent_initial_hash: AGENT_INITIAL_HASH) -> JsonString {
        JsonString::from(HashString::from(agent_initial_hash.to_string()))
    }
}

impl From<AGENT_LATEST_HASH> for JsonString {
    fn from(agent_latest_hash: AGENT_LATEST_HASH) -> JsonString {
        JsonString::from(HashString::from(agent_latest_hash.to_string()))
    }
}

impl From<PUBLIC_TOKEN> for JsonString {
    fn from(public_token: PUBLIC_TOKEN) -> JsonString {
        JsonString::from(Address::from(public_token.to_string()))
    }
}

impl From<CAPABILITY_REQ> for JsonString {
    fn from(cap_request: CAPABILITY_REQ) -> JsonString {
        JsonString::from(*cap_request)
    }
}

//--------------------------------------------------------------------------------------------------
// SYSTEM CONSTS
//--------------------------------------------------------------------------------------------------

// HC.GetMask
bitflags! {
  pub struct GetEntryMask: u8 {
    const ENTRY      = 1;
    const ENTRY_TYPE = 1 << 1;
    const SOURCES    = 1 << 2;
  }
}
// explicit `Default` implementation
impl Default for GetEntryMask {
    fn default() -> GetEntryMask {
        GetEntryMask::ENTRY
    }
}

// TODOs
//// HC.LinkAction
//pub enum LinkAction {
//    Add,
//    Delete,
//}
//
//// HC.PkgReq
//pub enum PkgRequest {
//    Chain,
//    ChainOption,
//    EntryTypes,
//}
//
//// HC.PkgReq.ChainOpt
//pub enum ChainOption {
//    None,
//    Headers,
//    Entries,
//    Full,
//}
//
//// HC.Bridge
//pub enum BridgeSide {
//    From,
//    To,
//}
//
//// HC.SysEntryType
//// WARNING Keep in sync with SystemEntryType in holochain-rust
//enum SystemEntryType {
//    Dna,
//    Agent,
//    Key,
//    Headers,
//    Deletion,
//}
//
//mod bundle_cancel {
//    // HC.BundleCancel.Reason
//    pub enum Reason {
//        UserCancel,
//        Timeout,
//    }
//    // HC.BundleCancel.Response
//    pub enum Response {
//        Ok,
//        Commit,
//    }
//}

/// Allowed input for close_bundle()
pub enum BundleOnClose {
    Commit,
    Discard,
}

//--------------------------------------------------------------------------------------------------
// API FUNCTIONS
//--------------------------------------------------------------------------------------------------






/// NOT YET AVAILABLE
// Returns a DNA property, which are defined by the DNA developer.
// They are custom values that are defined in the DNA file
// that can be used in the zome code for defining configurable behaviors.
// (e.g. Name, Language, Description, Author, etc.).
pub fn property<S: Into<String>>(_name: S) -> ZomeApiResult<String> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Reconstructs an address of the given entry data.
/// This is the same value that would be returned if `entry_type_name` and `entry_value` were passed
/// to the [commit_entry](fn.commit_entry.html) function and by which it would be retrievable from the DHT using [get_entry](fn.get_entry.html).
/// This is often used to reconstruct an address of a `base` argument when calling [get_links](fn.get_links.html).
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
/// # use hdk::error::ZomeApiResult;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::entry::entry_type::AppEntryType;
/// # use holochain_core_types::entry::AppEntryValue;
/// # use holochain_core_types::entry::Entry;
/// # use holochain_core_types::cas::content::Address;
/// # fn main() {
///
/// #[derive(Serialize, Deserialize, Debug, DefaultJson)]
/// pub struct Post {
///     content: String,
///     date_created: String,
/// }
///
/// pub fn handle_post_address(content: String) -> ZomeApiResult<Address> {
///     let post_entry = Entry::App("post".into(), Post {
///         content,
///         date_created: "now".into(),
///     }.into());
///
///     hdk::entry_address(&post_entry)
/// }
///
/// # }
/// ```
pub fn entry_address(entry: &Entry) -> ZomeApiResult<Address> {
    Dispatch::EntryAddress.with_input(entry)
}

/// Verifies a provenance (public key, signature) against a payload
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
/// # use holochain_core_types::signature::Provenance;
/// # use hdk::error::ZomeApiResult;
/// # fn main() {
/// pub fn handle_verify_message(message: String, provenance: Provenance) -> ZomeApiResult<bool> {
///     hdk::verify_signature(provenance, message)
/// }
/// # }
/// ```
pub fn verify_signature<S: Into<String>>(
    provenance: Provenance,
    payload: S,
) -> ZomeApiResult<bool> {
    Dispatch::VerifySignature.with_input(VerifySignatureArgs {
        provenance,
        payload: payload.into(),
    })
}

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
/// # use hdk::error::ZomeApiResult;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::cas::content::Address;
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


/// NOT YET AVAILABLE
pub fn start_bundle(_timeout: usize, _user_param: serde_json::Value) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// NOT YET AVAILABLE
pub fn close_bundle(_action: BundleOnClose) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Lets the DNA runtime sleep for the given duration.
/// # Examples
/// ```rust
/// # #[macro_use]
/// # extern crate hdk;
/// # use hdk::error::ZomeApiResult;
/// # use std::time::Duration;
///
/// # fn main() {
/// pub fn handle_some_function(content: String) -> ZomeApiResult<()> {
///     // ...
///     hdk::sleep(Duration::from_millis(100));
///     // ...
///     Ok(())
/// }
///
/// # }
/// ```
pub fn sleep(duration: Duration) -> ZomeApiResult<()> {
    let _: ZomeApiResult<()> = Dispatch::Sleep.with_input(JsonString::from(duration.as_nanos()));
    // internally returns RibosomeEncodedValue::Success which is a zero length allocation
    // return Ok(()) unconditionally instead of the "error" from success
    Ok(())
}
