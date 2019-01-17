//! This file contains many of the structs, enums, and functions relevant for Zome
//! developers! Detailed references and examples can be found here for how to use the
//! HDK exposed functions to access powerful Holochain functions.

use crate::{
    error::{ZomeApiError, ZomeApiResult},
    globals::*,
};
use holochain_core_types::{
    cas::content::Address,
    dna::capabilities::CapabilityCall,
    entry::Entry,
    error::{RibosomeEncodedAllocation, RibosomeEncodingBits, ZomeApiInternalResult},
};
pub use holochain_wasm_utils::api_serialization::validation::*;
use holochain_wasm_utils::{
    api_serialization::{
        get_entry::{
            EntryHistory, GetEntryArgs, GetEntryOptions, GetEntryResult, GetEntryResultType,
            StatusRequestKind,
        },
        get_links::{GetLinksArgs, GetLinksOptions, GetLinksResult},
        link_entries::LinkEntriesArgs,
        send::SendArgs,
        QueryArgs, QueryArgsNames, QueryResult, UpdateEntryArgs, ZomeFnCallArgs,
    },
    holochain_core_types::{
        hash::HashString,
        json::{JsonString, RawString},
    },
    memory::ribosome::load_ribosome_encoded_json,
};
use init_globals::hc_init_globals;
use serde_json;
use std::convert::{TryFrom, TryInto};

//--------------------------------------------------------------------------------------------------
// ZOME API GLOBAL VARIABLES
//--------------------------------------------------------------------------------------------------

lazy_static! {
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

//--------------------------------------------------------------------------------------------------
// SYSTEM CONSTS
//--------------------------------------------------------------------------------------------------

// HC.GetMask
bitflags! {
  pub struct GetEntryMask: u8 {
    const ENTRY      = 1 << 0;
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

pub enum Dispatch {
    Debug,
    InitGlobals,
    Call,
    CommitEntry,
    GetEntry,
    GetLinks,
    LinkEntries,
    EntryAddress,
    UpdateEntry,
    RemoveEntry,
    Query,
    Send,
}

impl Dispatch {
    pub fn with_input<I: TryInto<JsonString>, O: TryFrom<JsonString> + Into<JsonString>>(
        &self,
        input: I,
    ) -> ZomeApiResult<O> {
        let mut mem_stack = match unsafe { G_MEM_STACK } {
            Some(mem_stack) => mem_stack,
            None => {
                return Err(ZomeApiError::Internal(
                    "debug failed to load mem_stack".to_string(),
                ));
            }
        };

        let wasm_allocation = mem_stack.write_json(input)?;

        // Call Ribosome's commit_entry()
        let encoded_input: RibosomeEncodingBits =
            RibosomeEncodedAllocation::from(wasm_allocation).into();
        let encoded_output: RibosomeEncodingBits = unsafe {
            (match self {
                Dispatch::Debug => hc_debug,
                Dispatch::Call => hc_call,
                Dispatch::CommitEntry => hc_commit_entry,
                Dispatch::GetEntry => hc_get_entry,
                Dispatch::GetLinks => hc_get_links,
                Dispatch::LinkEntries => hc_link_entries,
                Dispatch::InitGlobals => hc_init_globals,
                Dispatch::EntryAddress => hc_entry_address,
                Dispatch::UpdateEntry => hc_update_entry,
                Dispatch::RemoveEntry => hc_remove_entry,
                Dispatch::Query => hc_query,
                Dispatch::Send => hc_send,
            })(encoded_input)
        };

        let result: ZomeApiInternalResult = match load_ribosome_encoded_json(encoded_output) {
            Ok(r) => r,
            Err(e) => {
                mem_stack.deallocate(wasm_allocation)?;
                return Err(e.into());
            }
        };

        // Free result & input allocations
        mem_stack.deallocate(wasm_allocation)?;

        // Done
        if result.ok {
            match JsonString::from(result.value).try_into() {
                Ok(v) => Ok(v),
                Err(_) => Err(ZomeApiError::from(String::from(
                    "Failed to deserialize return value",
                ))),
            }
        } else {
            Err(ZomeApiError::from(result.error))
        }
    }
}

/// Call an exposed function from another zome or another (bridged) instance running
/// on the same agent in the same container.
/// Arguments for the called function are passed as `JsonString`.
/// Returns the value that's returned by the given function as a json str.
/// # Examples
/// In order to utilize `call`, you must have at least two separate Zomes.
/// Here are two Zome examples, where one performs a `call` into the other.
///
/// This first one, is the one that is called into, with the Zome name `summer`.
/// ```rust
/// # #![feature(try_from)]
/// # #[macro_use]
/// # extern crate hdk;
/// # extern crate serde;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # #[macro_use]
/// # extern crate serde_json;
/// # use hdk::holochain_core_types::json::JsonString;
/// # use hdk::holochain_core_types::error::HolochainError;
/// # use hdk::holochain_core_types::error::RibosomeEncodingBits;
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_query(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_call(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_send(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
///
/// # fn main() {
///
/// fn handle_sum(num1: u32, num2: u32) -> JsonString {
///     let sum = num1 + num2;
///     json!({"sum": format!("{}",sum)}).into()
/// }
///
/// define_zome! {
///     entries: []
///
///     genesis: || {
///         Ok(())
///     }
///
///     functions: {
///         main (Public) {
///             sum: {
///                 inputs: |num1: u32, num2: u32|,
///                 outputs: |sum: JsonString|,
///                 handler: handle_sum
///             }
///         }
///     }
/// }
///
/// # }
/// ```
///
/// This second one, is the one that performs the call into the `summer` Zome.
/// ```rust
/// # #![feature(try_from)]
/// # #[macro_use]
/// # extern crate hdk;
/// # extern crate serde;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # #[macro_use]
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate holochain_core_types_derive;
///
/// # use hdk::holochain_core_types::hash::HashString;
/// # use hdk::holochain_core_types::json::JsonString;
/// # use hdk::holochain_core_types::error::HolochainError;
/// # use hdk::error::ZomeApiResult;
/// # use std::convert::TryInto;
/// # use hdk::holochain_core_types::error::RibosomeEncodingBits;
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_query(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_call(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_send(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
///
/// # fn main() {
///
/// fn handle_check_sum(num1: u32, num2: u32) -> ZomeApiResult<JsonString> {
///     #[derive(Serialize, Deserialize, Debug, DefaultJson)]
///     struct SumInput {
///         num1: u32,
///         num2: u32,
///     };
///     let call_input = SumInput {
///         num1: num1,
///         num2: num2,
///     };
///     hdk::call(hdk::THIS_INSTANCE, "summer", "main", "test_token", "sum", call_input.into())
/// }
///
/// define_zome! {
///     entries: []
///
///     genesis: || {
///         Ok(())
///     }
///
///     functions: {
///         main (Public) {
///             check_sum: {
///                 inputs: |num1: u32, num2: u32|,
///                 outputs: |sum: ZomeApiResult<JsonString>|,
///                 handler: handle_check_sum
///             }
///         }
///     }
/// }
///
/// # }
/// ```
pub fn call<S: Into<String>>(
    instance_handle: S,
    zome_name: S,
    cap_name: S, //temporary...
    cap_token: S,
    fn_name: S,
    fn_args: JsonString,
) -> ZomeApiResult<JsonString> {
    Dispatch::Call.with_input(ZomeFnCallArgs {
        instance_handle: instance_handle.into(),
        zome_name: zome_name.into(),
        cap: Some(CapabilityCall::new(
            cap_name.into(),
            Address::from(cap_token.into()),
            None,
        )),
        fn_name: fn_name.into(),
        fn_args: String::from(fn_args),
    })
}

/// Prints a string through the stdout of the running service, and also
/// writes that string to the logger in the execution context
/// # Examples
/// ```rust
/// # #[macro_use]
/// # extern crate hdk;
/// # use hdk::error::ZomeApiResult;
///
/// # fn main() {
/// pub fn handle_some_function(content: String) -> ZomeApiResult<()> {
///     // ...
///     hdk::debug("write a message to the logs");
///     // ...
///     Ok(())
/// }
///
/// # }
/// ```
pub fn debug<J: TryInto<JsonString>>(msg: J) -> ZomeApiResult<()> {
    Dispatch::Debug.with_input(msg)
}

/// Attempts to commit an entry to your local source chain. The entry
/// will have to pass the defined validation rules for that entry type.
/// If the entry type is defined as public, will also publish the entry to the DHT.
/// Returns either an address of the committed entry as a string, or an error.
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
/// # use holochain_core_types::entry::Entry;
/// # use holochain_core_types::cas::content::Address;
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
    Dispatch::CommitEntry.with_input(entry)
}

/// Retrieves latest version of an entry from the local chain or the DHT, by looking it up using
/// the specified address.
/// Returns None if no entry exists at the specified address or
/// if the entry's crud-status is not LIVE.
/// # Examples
/// ```rust
/// # extern crate hdk;
/// # extern crate holochain_core_types;
/// # use hdk::error::ZomeApiResult;
/// # use holochain_core_types::entry::Entry;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::cas::content::Address;
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

/// Returns the Entry at the exact address specified, whatever its crud-status.
/// Returns None if no entry exists at the specified address.
pub fn get_entry_initial(address: &Address) -> ZomeApiResult<Option<Entry>> {
    let entry_result = get_entry_result(
        address,
        GetEntryOptions::new(StatusRequestKind::Initial, true, false, false),
    )?;
    Ok(entry_result.latest())
}

/// Return an EntryHistory filled with all the versions of the entry from the version at
/// the specified address to the latest.
/// Returns None if no entry exists at the specified address.
pub fn get_entry_history(address: &Address) -> ZomeApiResult<Option<EntryHistory>> {
    let entry_result = get_entry_result(
        address,
        GetEntryOptions::new(StatusRequestKind::All, true, false, false),
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
///         hdk::get_entry_result(&in_reply_to_address, GetEntryOptions { status_request: StatusRequestKind::All, entry: false, header: false, sources: false })?;
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
) -> Result<(), ZomeApiError> {
    Dispatch::LinkEntries.with_input(LinkEntriesArgs {
        base: base.clone(),
        target: target.clone(),
        tag: tag.into(),
    })
}

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

/// NOT YET AVAILABLE
pub fn sign<S: Into<String>>(_doc: S) -> ZomeApiResult<String> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// NOT YET AVAILABLE
pub fn verify_signature<S: Into<String>>(
    _signature: S,
    _data: S,
    _pub_key: S,
) -> ZomeApiResult<bool> {
    Err(ZomeApiError::FunctionNotImplemented)
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
pub fn remove_entry(address: &Address) -> ZomeApiResult<()> {
    Dispatch::RemoveEntry.with_input(address.to_owned())
}

/// Consumes three values; the address of an entry get get links from (the base), the tag of the links
/// to be retrieved, and an options struct for selecting what meta data and crud status links to retrieve.
/// Note: the tag is intended to describe the relationship between the `base` and other entries you wish to lookup.
/// This function returns a list of addresses of other entries which matched as being linked by the given `tag`.
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
///
/// # fn main() {
/// pub fn handle_posts_by_agent(agent: Address) -> ZomeApiResult<GetLinksResult> {
///     hdk::get_links_with_options(&agent, "authored_posts", GetLinksOptions::default())
/// }
/// # }
/// ```
pub fn get_links_with_options<S: Into<String>>(
    base: &Address,
    tag: S,
    options: GetLinksOptions,
) -> ZomeApiResult<GetLinksResult> {
    Dispatch::GetLinks.with_input(GetLinksArgs {
        entry_address: base.clone(),
        tag: tag.into(),
        options,
    })
}

/// Helper function for get_links. Returns a vector with the default return results.
pub fn get_links<S: Into<String>>(base: &Address, tag: S) -> ZomeApiResult<GetLinksResult> {
    get_links_with_options(base, tag, GetLinksOptions::default())
}

/// Retrieves data about entries linked to a base address with a given tag. This is the most general version of the various get_links
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
///
/// # fn main() {
/// fn hangle_get_links_result(address: Address) -> ZomeApiResult<Vec<ZomeApiResult<GetEntryResult>>> {
///    hdk::get_links_result(&address, "test-tag", GetLinksOptions::default(), GetEntryOptions::default())
/// }
/// # }
/// ```
pub fn get_links_result<S: Into<String>>(
    base: &Address,
    tag: S,
    options: GetLinksOptions,
    get_entry_options: GetEntryOptions,
) -> ZomeApiResult<Vec<ZomeApiResult<GetEntryResult>>> {
    let get_links_result = get_links_with_options(base, tag, options)?;
    let result = get_links_result
        .addresses()
        .iter()
        .map(|address| get_entry_result(&address, get_entry_options.clone()))
        .collect();
    Ok(result)
}

/// Helper function for get_links. Returns a vector of the entries themselves
pub fn get_links_and_load<S: Into<String>>(
    base: &HashString,
    tag: S,
) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
    let get_links_result = get_links_result(
        base,
        tag,
        GetLinksOptions::default(),
        GetEntryOptions::default(),
    )?;

    let entries = get_links_result
    .into_iter()
    .map(|get_result| {
        let get_type = get_result?.result;
        match get_type {
            GetEntryResultType::Single(elem) => Ok(elem.entry.unwrap().to_owned()),
            GetEntryResultType::All(_) => Err(ZomeApiError::Internal("Invalid response. get_links_result returned all entries when latest was requested".to_string()))
        }
    })
    .collect();

    Ok(entries)
}

/// Returns a list of entries from your local source chain, that match a given entry type name or names.
///
/// Each name may be a plain entry type name, or a "glob" pattern such as "prefix/*" (matches all
/// entry types starting with "prefix/"), or "[!%]*e" (matches all non-system non-name-spaced entry
/// types ending in "e").  All names and patterns are merged into a single efficient Regular
/// Expression for scanning.
///
/// Entry type name-spaces are supported by including "/" in your entry type names; use vec![], "",
/// or "**" to match all names in all name-spaces, "*" to match all non-namespaced names.
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
pub fn query(
    entry_type_names: QueryArgsNames,
    start: u32,
    limit: u32,
) -> ZomeApiResult<QueryResult> {
    Dispatch::Query.with_input(QueryArgs {
        entry_type_names,
        start,
        limit,
    })
}

/// Sends a node-to-node message to the given agent, specified by their address.
/// Addresses of agents can be accessed using [hdk::AGENT_ADDRESS](struct.AGENT_ADDRESS.html).
/// This works in conjunction with the `receive` callback that has to be defined in the
/// [define_zome!](../macro.define_zome.html) macro.
///
/// This function dispatches a message to the receiver, and will wait up to 60 seconds before returning a timeout error. The `send` function will return the string returned
/// by the `receive` callback of the other node.
/// # Examples
/// ```rust
/// # #![feature(try_from)]
/// # #[macro_use]
/// # extern crate hdk;
/// # extern crate holochain_core_types;
/// # extern crate serde;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # #[macro_use]
/// # extern crate serde_json;
/// # use hdk::error::ZomeApiResult;
/// # use holochain_core_types::cas::content::Address;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::error::RibosomeEncodingBits;
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_query(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_call(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
/// # #[no_mangle]
/// # pub fn hc_send(_: RibosomeEncodingBits) -> RibosomeEncodingBits { 0 }
///
/// # fn main() {
/// fn handle_send_message(to_agent: Address, message: String) -> ZomeApiResult<String> {
///     // because the function signature of hdk::send is the same as the
///     // signature of handle_send_message we can just directly return its' result
///     hdk::send(to_agent, message)
/// }
///
/// define_zome! {
///    entries: []
///
///    genesis: || { Ok(()) }
///
///    receive: |payload| {
///        // simply pass back the received value, appended to a modifier
///        format!("Received: {}", payload)
///    }
///
///    functions: {
///        main (Public) {
///            send_message: {
///                inputs: |to_agent: Address, message: String|,
///                outputs: |response: ZomeApiResult<String>|,
///                handler: handle_send_message
///            }
///        }
///    }
///}
/// # }
/// ```
pub fn send(to_agent: Address, payload: String) -> ZomeApiResult<String> {
    Dispatch::Send.with_input(SendArgs { to_agent, payload })
}

/// NOT YET AVAILABLE
pub fn start_bundle(_timeout: usize, _user_param: serde_json::Value) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// NOT YET AVAILABLE
pub fn close_bundle(_action: BundleOnClose) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}
