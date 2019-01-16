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
    error::{CoreError, HolochainError, RibosomeReturnCode, ZomeApiInternalResult},
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
        QueryArgs, QueryArgsNames, QueryArgsOptions, QueryResult, UpdateEntryArgs, ZomeFnCallArgs,
    },
    holochain_core_types::{
        hash::HashString,
        json::{JsonString, RawString},
    },
    memory_allocation::*,
    memory_serialization::*,
};
use serde_json;
use std::{convert::TryInto, os::raw::c_char};

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
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };

    let allocation_of_input = store_as_json(&mut mem_stack, msg)?;

    unsafe {
        hc_debug(allocation_of_input.encode());
    }

    mem_stack
        .deallocate(allocation_of_input)
        .expect("should be able to deallocate input that has been allocated on memory stack");

    Ok(())
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
/// # #[macro_use]
/// # extern crate hdk;
/// # extern crate serde;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # #[macro_use]
/// # extern crate serde_json;
/// # use hdk::holochain_core_types::json::JsonString;
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_query(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_call(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_send(_: u32) -> u32 { 0 }
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
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_query(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_call(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_send(_: u32) -> u32 { 0 }
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
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }

    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(
        &mut mem_stack,
        ZomeFnCallArgs {
            instance_handle: instance_handle.into(),
            zome_name: zome_name.into(),
            cap: Some(CapabilityCall::new(
                cap_name.into(),
                Address::from(cap_token.into()),
                None,
            )),
            fn_name: fn_name.into(),
            fn_args: String::from(fn_args),
        },
    )?;

    // Call WASMI-able commit
    let encoded_allocation_of_result: u32 = unsafe { hc_call(allocation_of_input.encode() as u32) };
    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result)?;

    // Free result & input allocations.
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
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
///
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: u32) -> u32 { 0 }
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
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }

    let allocation_of_input = store_as_json(&mut mem_stack, entry)?;

    // Call Ribosome's commit_entry()
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_commit_entry(allocation_of_input.encode() as u32);
    }

    // Deserialize complex result stored in wasm memory
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
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
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }

    let entry_args = GetEntryArgs {
        address: address.clone(),
        options,
    };

    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(&mut mem_stack, entry_args)?;

    // Call WASMI-able get_entry
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_get_entry(allocation_of_input.encode() as u32);
    }
    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
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
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };

    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(
        &mut mem_stack,
        LinkEntriesArgs {
            base: base.clone(),
            target: target.clone(),
            tag: tag.into(),
        },
    )?;

    let encoded_allocation_of_result: u32 =
        unsafe { hc_link_entries(allocation_of_input.encode() as u32) };

    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
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
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }
    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(&mut mem_stack, entry)?;

    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_entry_address(allocation_of_input.encode() as u32);
    }

    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
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
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }

    let update_args = UpdateEntryArgs {
        new_entry,
        address: address.clone(),
    };

    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(&mut mem_stack, update_args)?;

    // Call Ribosome
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_update_entry(allocation_of_input.encode() as u32);
    }
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
}

/// NOT YET AVAILABLE
pub fn update_agent() -> ZomeApiResult<Address> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Commit a DeletionEntry to your local source chain that marks an entry as 'deleted' by setting
/// its status metadata to `Deleted` and adding the DeleteEntry's address in the deleted entry's
/// metadata, which will be used by validation routes.
pub fn remove_entry(address: &Address) -> ZomeApiResult<()> {
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }
    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(&mut mem_stack, address.clone())?;

    // Call WASMI-able get_entry
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_remove_entry(allocation_of_input.encode() as u32);
    }
    let res = check_for_ribosome_error(encoded_allocation_of_result);
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    res
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
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };
    // Put args in struct and serialize into memory

    let allocation_of_input = store_as_json(
        &mut mem_stack,
        GetLinksArgs {
            entry_address: base.clone(),
            tag: tag.into(),
            options: options,
        },
    )?;

    // Call Ribosome
    let encoded_allocation_of_result: u32 =
        unsafe { hc_get_links(allocation_of_input.encode() as u32) };

    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;

    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
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
/// With hdk::query_result, you can specify a package of QueryArgsOptions, including the ability
/// to get a Vec<ChainHeaders>:
/// 
/// pub fn get_post_headers() -> ZomeApiResult<QueryResult> {
///     hdk::query_result("post".into(), QueryArgsOptions{ headers: true, ..Default::default()})
/// }
pub fn query(
    entry_type_names: QueryArgsNames,
    start: usize,
    limit: usize,
) -> ZomeApiResult<Vec<Address>> {
    // The hdk::query API always returns a simple Vec<Address>
    match query_result( entry_type_names,
                        QueryArgsOptions {
                            start: Some(start),
                            limit: Some(limit),
                            headers: None,
                        }) {
        Ok(result) => match result {
            QueryResult::Addresses(addresses) => Ok(addresses),
            _ => return Err(ZomeApiError::FunctionNotImplemented), // should never occur
        }
        Err(e) => Err(e),
    }
}

pub fn query_result(
    entry_type_names: QueryArgsNames,
    options: QueryArgsOptions,
) -> ZomeApiResult<QueryResult> {
    let mut mem_stack: SinglePageStack = unsafe { G_MEM_STACK.unwrap() };

    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(
        &mut mem_stack,
        QueryArgs {
            entry_type_names,
            options: Some(options),
        },
    )?;

    let encoded_allocation_of_result: u32 =
        unsafe { hc_query(allocation_of_input.encode() as u32) };

    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
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
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_query(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_call(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_send(_: u32) -> u32 { 0 }
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
    let mut mem_stack: SinglePageStack = unsafe { G_MEM_STACK.unwrap() };

    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(&mut mem_stack, SendArgs { to_agent, payload })?;

    let encoded_allocation_of_result: u32 = unsafe { hc_send(allocation_of_input.encode() as u32) };

    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(String::from(result.value))
    } else {
        Err(ZomeApiError::from(result.error))
    }
}

/// NOT YET AVAILABLE
pub fn start_bundle(_timeout: usize, _user_param: serde_json::Value) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// NOT YET AVAILABLE
pub fn close_bundle(_action: BundleOnClose) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}

//--------------------------------------------------------------------------------------------------
// Helpers
//--------------------------------------------------------------------------------------------------

#[doc(hidden)]
pub fn check_for_ribosome_error(encoded_allocation: u32) -> ZomeApiResult<()> {
    // Check for error from Ribosome
    let rib_result = decode_encoded_allocation(encoded_allocation);
    match rib_result {
        // Expecting a 'Success' return code
        Err(ret_code) => match ret_code {
            RibosomeReturnCode::Success => Ok(()),
            RibosomeReturnCode::Failure(err_code) => {
                Err(ZomeApiError::Internal(err_code.to_string()))
            }
        },
        // If we have an allocation, than it should be a CoreError
        Ok(allocation) => {
            let maybe_err: Result<CoreError, HolochainError> =
                load_json_from_raw(allocation.offset() as *mut c_char);
            match maybe_err {
                Err(hc_err) => Err(ZomeApiError::Internal(hc_err.to_string())),
                Ok(core_err) => Err(ZomeApiError::Internal(core_err.to_string())),
            }
        }
    }
}
