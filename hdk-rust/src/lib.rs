//! File holding the public Zome API
//! All API Reference documentation should be done here.

pub extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
pub extern crate holochain_wasm_utils;

pub mod globals;
pub mod init_globals;
pub mod macros;

use self::RibosomeError::*;
use globals::*;
use holochain_wasm_utils::{
    api_serialization::{
        commit::{CommitEntryArgs, CommitEntryResult},
        get_entry::{GetEntryArgs, GetEntryResult, GetResultStatus},
        validation::*,
    },
    holochain_core_types::hash::HashString,
    memory_serialization::*, memory_allocation::*,
};
pub use holochain_wasm_utils::api_serialization::validation::*;

pub fn init_memory_stack(encoded_allocation_of_input: u32) {
    // Actual program
    // Init memory stack
    unsafe {
        G_MEM_STACK =
            Some(SinglePageStack::from_encoded(encoded_allocation_of_input));
    }
}

pub fn serialize_wasm_output<T: serde::Serialize>(output: T) -> u32
{
    // Serialize output in WASM memory
    unsafe {
        return serialize_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), output) as u32
    }
}


//--------------------------------------------------------------------------------------------------
// APP GLOBAL VARIABLES
//--------------------------------------------------------------------------------------------------

lazy_static! {
  /// The name of this Holochain taken from its DNA.
  pub static ref APP_NAME: &'static str = &APP_GLOBALS.app_name;

  /// The hash of this Holochain's DNA.
  /// Nodes must run the same DNA to be on the same DHT.
  pub static ref APP_DNA_HASH: &'static HashString = &APP_GLOBALS.app_dna_hash;

  /// The identity string used when the chain was first initialized.
  /// If you used JSON to embed multiple properties (such as FirstName, LastName, Email, etc),
  /// they can be retrieved here as App.Agent.FirstName, etc. (FIXME)
  pub static ref APP_AGENT_ID_STR: &'static str = &APP_GLOBALS.app_agent_id_str;

  /// The hash of your public key.
  /// This is your node address on the DHT.
  /// It can be used for node-to-node messaging with `send` and `receive` functions.
  pub static ref APP_AGENT_KEY_HASH: &'static HashString = &APP_GLOBALS.app_agent_key_hash;

  /// The hash of the first identity entry on your chain (The second entry on your chain).
  /// This is your peer's identity on the DHT.
  pub static ref APP_AGENT_INITIAL_HASH: &'static HashString = &APP_GLOBALS.app_agent_initial_hash;

  /// The hash of the most recent identity entry that has been committed to your chain.
  /// Starts with the same value as APP_AGENT_INITIAL_HASH.
  /// After a call to `update_agent` it will have the value of the hash of the newly committed identity entry.
  pub static ref APP_AGENT_LATEST_HASH: &'static HashString = &APP_GLOBALS.app_agent_latest_hash;
}

//--------------------------------------------------------------------------------------------------
// SYSTEM CONSTS
//--------------------------------------------------------------------------------------------------

// HC.Version
const VERSION: u16 = 1;
const VERSION_STR: &'static str = "1";

// HC.HashNotFound
#[derive(Debug)]
pub enum RibosomeError {
    RibosomeFailed(String),
    FunctionNotImplemented,
    HashNotFound,
    ValidationFailed(String)
}

impl RibosomeError {
    pub fn to_json(&self) -> serde_json::Value {
        let err_str = match self {
            RibosomeFailed(error_desc) => error_desc.clone(),
            FunctionNotImplemented => "Function not implemented".to_string(),
            HashNotFound => "Hash not found".to_string(),
            ValidationFailed(msg) => format!("Validation failed: {}", msg),
        };
        json!({ "error": err_str })
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
pub fn debug(msg: &str) -> Result<(), RibosomeError> {
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };
    let maybe_allocation_of_input = serialize(&mut mem_stack, msg);
    if let Err(err_code) = maybe_allocation_of_input {
        return Err(RibosomeError::RibosomeFailed(err_code.to_string()));
    }
    let allocation_of_input = maybe_allocation_of_input.unwrap();
    unsafe {
        hc_debug(allocation_of_input.encode());
    }
    mem_stack
        .deallocate(allocation_of_input)
        .expect("should be able to deallocate input that has been allocated on memory stack");
    Ok(())
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
pub fn commit_entry(
    entry_type_name: &str,
    entry_content: serde_json::Value,
) -> Result<HashString, RibosomeError> {
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }

    // Put args in struct and serialize into memory
    let input = CommitEntryArgs {
        entry_type_name: entry_type_name.to_string(),
        entry_value: entry_content.to_string(),
    };
    let maybe_allocation_of_input = serialize(&mut mem_stack, input);
    if let Err(err_code) = maybe_allocation_of_input {
        return Err(RibosomeError::RibosomeFailed(err_code.to_string()));
    }
    let allocation_of_input = maybe_allocation_of_input.unwrap();

    // Call WASMI-able commit
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_commit_entry(allocation_of_input.encode() as u32);
    }
    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result = try_deserialize_allocation(encoded_allocation_of_result as u32);

    if let Err(err_str) = result {
        return Err(RibosomeError::RibosomeFailed(err_str));
    }
    let output: CommitEntryResult = result.unwrap();

    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    if output.validation_failure.len() > 0 {
        Err(RibosomeError::ValidationFailed(output.validation_failure))
    } else {
        Ok(HashString::from(output.address))
    }
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
pub fn get_entry(entry_hash: HashString) -> Result<Option<String>, RibosomeError> {
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }

    // Put args in struct and serialize into memory
    let input = GetEntryArgs {
        address: entry_hash,
    };
    let maybe_allocation_of_input = serialize(&mut mem_stack, input);
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
    let result = try_deserialize_allocation(encoded_allocation_of_result as u32);
    if let Err(err_str) = result {
        return Err(RibosomeError::RibosomeFailed(err_str));
    }
    let result : GetEntryResult = result.unwrap();

    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    match result.status {
        GetResultStatus::Found => Ok(Some(result.entry)),
        GetResultStatus::NotFound => Ok(None),
    }
}

/// FIXME DOC
pub fn link_entries<S: Into<String>>(
    _base: HashString,
    _target: HashString,
    _tag: S
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
