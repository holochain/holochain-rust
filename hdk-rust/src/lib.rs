//! Holochain Development Kit (HDK)
//!
//! The HDK helps in writing Holochain applications.
//! Holochain DNAs need to be written in WebAssembly, or a language that compiles to Wasm,
//! such as Rust. The HDK handles some of the low-level details of Wasm execution like
//! memory allocation, (de)serializing data, and shuffling data and functions into and out of Wasm
//! memory via some helper functions and Holochain-specific macros.
//!
//! The HDK lets the developer focus on application logic and, as much as possible, forget about the
//! underlying low-level implementation. It would be possible to write DNA source code without an
//! HDK, but it would be extremely tedious!
#![feature(try_from)]
#![feature(never_type)]
pub extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate serde_derive;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
pub extern crate holochain_core_types;
pub extern crate holochain_dna;
pub extern crate holochain_wasm_utils;

mod api;
pub mod entry_definition;
pub mod global_fns;
pub mod globals;
pub mod init_globals;
pub mod macros;
use serde::{Serialize, Serializer};
use std::convert::TryInto;

use self::RibosomeError::*;
use globals::*;
pub use holochain_wasm_utils::api_serialization::validation::*;
use holochain_wasm_utils::{
    api_serialization::get_entry::{GetEntryResult, GetResultStatus},
    holochain_core_types::{
        cas::content::Address, entry::SerializedEntry, hash::HashString, json::JsonString,
    },
    memory_allocation::*,
    memory_serialization::*,
};

pub mod meta;

pub use api::*;
pub use holochain_core_types::validation::*;

pub fn init_memory_stack(encoded_allocation_of_input: u32) {
    // Actual program
    // Init memory stack
    unsafe {
        G_MEM_STACK =
            Some(SinglePageStack::from_encoded_allocation(encoded_allocation_of_input).unwrap());
    }
}

pub fn serialize_wasm_output<J: TryInto<JsonString>>(jsonable: J) -> u32 {
    // Serialize output in WASM memory
    unsafe { store_as_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), jsonable) as u32 }
}

//--------------------------------------------------------------------------------------------------
// SYSTEM CONSTS
//--------------------------------------------------------------------------------------------------
/*
// HC.Version
const VERSION: u16 = 1;
const VERSION_STR: &'static str = "1";
*/
// HC.HashNotFound
#[derive(Debug)]
pub enum RibosomeError {
    RibosomeFailed(String),
    FunctionNotImplemented,
    HashNotFound,
    ValidationFailed(String),
}

impl Serialize for RibosomeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&match self {
            RibosomeFailed(ref error_desc) => error_desc.to_owned(),
            FunctionNotImplemented => String::from("Function not implemented"),
            HashNotFound => String::from("Hash not found"),
            ValidationFailed(ref msg) => format!("Validation failed: {}", msg),
        })
    }
}

impl From<RibosomeError> for JsonString {
    fn from(ribosome_error: RibosomeError) -> JsonString {
        JsonString::from(
            serde_json::to_string(&ribosome_error).expect("could not Jsonify RibosomeError"),
        )
        // let err_str = match ribosome_error {
        //     RibosomeFailed(error_desc) => error_desc.clone(),
        //     FunctionNotImplemented => "Function not implemented".to_string(),
        //     HashNotFound => "Hash not found".to_string(),
        //     ValidationFailed(msg) => format!("Validation failed: {}", msg),
        // };
        // JsonString::from(RawString::from(err_str))
    }
}

// HC.Status
// WARNING keep in sync with CRUDStatus
bitflags! {
  pub struct EntryStatus: u8 {
    const LIVE     = 1 << 0;
    const REJECTED = 1 << 1;
    const DELETED  = 1 << 2;
    const MODIFIED = 1 << 3;
  }
}

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
/*
// HC.LinkAction
pub enum LinkAction {
    Add,
    Delete,
}

// HC.PkgReq
pub enum PkgRequest {
    Chain,
    ChainOption,
    EntryTypes,
}

// HC.PkgReq.ChainOpt
pub enum ChainOption {
    None,
    Headers,
    Entries,
    Full,
}

// HC.Bridge
pub enum BridgeSide {
    From,
    To,
}

// HC.SysEntryType
// WARNING Keep in sync with SystemEntryType in holochain-rust
enum SystemEntryType {
    Dna,
    Agent,
    Key,
    Headers,
    Deletion,
}

mod bundle_cancel {
    // HC.BundleCancel.Reason
    pub enum Reason {
        UserCancel,
        Timeout,
    }
    // HC.BundleCancel.Response
    pub enum Response {
        Ok,
        Commit,
    }
}
*/
/// Allowed input for close_bundle()
pub enum BundleOnClose {
    Commit,
    Discard,
}

//--------------------------------------------------------------------------------------------------
// API FUNCTIONS
//--------------------------------------------------------------------------------------------------

/// FIXME DOC
/// Returns an application property, which are defined by the app developer.
/// It returns values from the DNA file that you set as properties of your application
/// (e.g. Name, Language, Description, Author, etc.).
pub fn property<S: Into<String>>(_name: S) -> Result<String, RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// FIXME DOC
pub fn make_hash<S: Into<String>>(
    _entry_type: S,
    _entry_data: serde_json::Value,
) -> Result<HashString, RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// FIXME DOC
pub fn call<S: Into<String>>(
    _zome_name: S,
    _function_name: S,
    _arguments: serde_json::Value,
) -> Result<serde_json::Value, RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// FIXME DOC
pub fn sign<S: Into<String>>(_doc: S) -> Result<String, RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// FIXME DOC
pub fn verify_signature<S: Into<String>>(
    _signature: S,
    _data: S,
    _pub_key: S,
) -> Result<bool, RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// FIXME DOC
pub fn update_entry<S: Into<String>>(
    _entry_type: S,
    _entry: serde_json::Value,
    _replaces: HashString,
) -> Result<HashString, RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// FIXME DOC
pub fn update_agent() -> Result<HashString, RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// FIXME DOC
/// Commit a Deletion System Entry
pub fn remove_entry<S: Into<String>>(
    _entry: HashString,
    _message: S,
) -> Result<HashString, RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// implements access to low-level WASM hc_get_entry
pub fn get_entry(entry_address: Address) -> Result<Option<SerializedEntry>, RibosomeError> {
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }

    // Put args in struct and serialize into memory
    let input = GetEntryArgs {
        address: entry_address,
    };
    let maybe_allocation_of_input = store_as_json(&mut mem_stack, input);
    if let Err(err_code) = maybe_allocation_of_input {
        return Err(RibosomeError::RibosomeFailed(err_code.to_string()));
    }
    let allocation_of_input = maybe_allocation_of_input.unwrap();

    // Call WASMI-able get_entry
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_get_entry(allocation_of_input.encode() as u32);
    }
    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result = load_json(encoded_allocation_of_result as u32);
    if let Err(err_raw_str) = result {
        return Err(RibosomeError::RibosomeFailed(String::from(err_raw_str)));
    }
    let get_entry_result: GetEntryResult = result.unwrap();

    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    match get_entry_result.status {
        GetResultStatus::Found => Ok(get_entry_result.maybe_serialized_entry),
        GetResultStatus::NotFound => Ok(None),
    }
}

/// FIXME DOC
pub fn link_entries<S: Into<String>>(
    _base: HashString,
    _target: HashString,
    _tag: S,
) -> Result<(), RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// FIXME DOC
pub fn get_links<S: Into<String>>(
    _base: HashString,
    _tag: S,
) -> Result<Vec<HashString>, RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// FIXME DOC
pub fn query() -> Result<Vec<String>, RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// FIXME DOC
pub fn send(
    _to: HashString,
    _message: serde_json::Value,
) -> Result<serde_json::Value, RibosomeError> {
    // FIXME
    Err(RibosomeError::FunctionNotImplemented)
}

/// FIXME DOC
pub fn start_bundle(_timeout: usize, _user_param: serde_json::Value) {
    // FIXME
}

/// FIXME DOC
pub fn close_bundle(_action: BundleOnClose) {
    // FIXME
}
