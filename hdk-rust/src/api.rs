use error::{ZomeApiError, ZomeApiResult};
use globals::*;
pub use holochain_wasm_utils::api_serialization::validation::*;
use holochain_wasm_utils::{
    api_serialization::{
        commit::{CommitEntryArgs, CommitEntryResult},
        get_entry::{GetEntryArgs, GetEntryOptions, GetEntryResult, GetResultStatus},
        get_links::{GetLinksArgs, GetLinksResult},
        link_entries::LinkEntriesArgs,
        HashEntryArgs, ZomeFnCallArgs,
        QueryResult, QueryArgs,
    },
    holochain_core_types::{
        error::{CoreError, HolochainError, RibosomeReturnCode},
        hash::HashString,
    },
    memory_allocation::*,
    memory_serialization::*,
};
use serde::de::DeserializeOwned;
use serde_json;
use std::os::raw::c_char;

//--------------------------------------------------------------------------------------------------
// ZOME API GLOBAL VARIABLES
//--------------------------------------------------------------------------------------------------

lazy_static! {
  /// The `name` property as taken from the DNA.
  pub static ref DNA_NAME: &'static str = &GLOBALS.dna_name;

  /// The hash of the DNA the Zome is embedded within.
  /// This is often useful as a fixed value that is known by all
  /// participants running the DNA.
  pub static ref DNA_HASH: &'static HashString = &GLOBALS.dna_hash;

  /// The identity string used when the chain was first initialized.
  pub static ref AGENT_ID_STR: &'static str = &GLOBALS.agent_id_str;

  /// The hash of your public key.
  /// This is your node address on the DHT.
  /// It can be used for node-to-node messaging with `send` and `receive` functions.
  pub static ref AGENT_ADDRESS: &'static HashString = &GLOBALS.agent_address;

  /// The hash of the first identity entry on your chain (The second entry on your chain).
  /// This is your peer's identity on the DHT.
  pub static ref AGENT_INITIAL_HASH: &'static HashString = &GLOBALS.agent_initial_hash;

  #[doc(hidden)]
  /// The hash of the most recent identity entry that has been committed to your chain.
  /// Starts with the same value as AGENT_INITIAL_HASH.
  /// After a call to `update_agent` it will have the value of the hash of the newly committed identity entry.
  pub static ref AGENT_LATEST_HASH: &'static HashString = &GLOBALS.agent_latest_hash;
}

//--------------------------------------------------------------------------------------------------
// SYSTEM CONSTS
//--------------------------------------------------------------------------------------------------

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

// Allowed input for close_bundle()
pub enum BundleOnClose {
    Commit,
    Discard,
}

//--------------------------------------------------------------------------------------------------
// API FUNCTIONS
//--------------------------------------------------------------------------------------------------

/// Prints a string through the stdout of the running service, and also
/// writes that string to the logger in the execution context
pub fn debug(msg: &str) -> ZomeApiResult<()> {
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };
    let maybe_allocation_of_input = store_as_json(&mut mem_stack, msg);
    if let Err(err_code) = maybe_allocation_of_input {
        return Err(ZomeApiError::Internal(err_code.to_string()));
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

/// Call an exposed function from another zome.
/// Arguments for the called function are passed as serde_json::Value.
/// Returns the value that's returned by the given function as json str.
pub fn call<S: Into<String>>(
    zome_name: S,
    cap_name: S,
    fn_name: S,
    fn_args: serde_json::Value,
) -> ZomeApiResult<String> {
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }

    // Put args in struct and serialize into memory
    let input = ZomeFnCallArgs {
        zome_name: zome_name.into(),
        cap_name: cap_name.into(),
        fn_name: fn_name.into(),
        fn_args: fn_args.to_string(),
    };
    let maybe_allocation_of_input = store_as_json(&mut mem_stack, input.clone());
    if let Err(err_code) = maybe_allocation_of_input {
        return Err(ZomeApiError::Internal(err_code.to_string()));
    }
    let allocation_of_input = maybe_allocation_of_input.unwrap();

    // Call WASMI-able commit
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_call(allocation_of_input.encode() as u32);
    }
    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result = load_string(encoded_allocation_of_result as u32);
    // Free result & input allocations.
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    result.map_err(|err_code| ZomeApiError::Internal(err_code.to_string()))
}

/// Attempts to commit an entry to your local source chain. The entry
/// will have to pass the defined validation rules for that entry type.
/// If the entry type is defined as public, will also publish the entry to the DHT.
/// Returns either an address of the committed entry as a string, or an error.
pub fn commit_entry(
    entry_type_name: &str,
    entry_value: serde_json::Value,
) -> ZomeApiResult<HashString> {
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }
    // Put args in struct and serialize into memory
    let input = CommitEntryArgs {
        entry_type_name: entry_type_name.to_string(),
        entry_value: entry_value.to_string(),
    };
    let maybe_allocation_of_input = store_as_json(&mut mem_stack, input);
    if let Err(err_code) = maybe_allocation_of_input {
        return Err(ZomeApiError::Internal(err_code.to_string()));
    }
    let allocation_of_input = maybe_allocation_of_input.unwrap();

    // Call Ribosome's commit_entry()
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_commit_entry(allocation_of_input.encode() as u32);
    }
    // Deserialize complex result stored in wasm memory
    let result = load_json(encoded_allocation_of_result as u32);
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    result
        .map(|output: CommitEntryResult| HashString::from(output.address))
        .map_err(|hc_err| match hc_err {
            HolochainError::ValidationFailed(msg) => ZomeApiError::ValidationFailed(msg),
            _ => ZomeApiError::Internal(hc_err.to_string()),
        })
}

/// Retrieves an entry from the local chain or the DHT, by looking it up using
/// its address.
pub fn get_entry<T>(address: HashString) -> Result<Option<T>, ZomeApiError>
where
    T: DeserializeOwned,
{
    let res = get_entry_result(address, GetEntryOptions {});
    match res {
        Ok(result) => match result.status {
            GetResultStatus::Found => {
                let maybe_entry_value: Result<T, _> = serde_json::from_str(&result.entry);
                match maybe_entry_value {
                    Ok(entry_value) => Ok(Some(entry_value)),
                    Err(err) => Err(ZomeApiError::Internal(err.to_string())),
                }
            }
            GetResultStatus::NotFound => Ok(None),
        },
        Err(err) => Err(err),
    }
}

/// Retrieves an entry and meta data from the local chain or the DHT, by looking it up using
/// its address, and a the full options to specify exactly what data to return
pub fn get_entry_result(
    address: HashString,
    _options: GetEntryOptions,
) -> ZomeApiResult<GetEntryResult> {
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }
    // Put args in struct and serialize into memory
    let input = GetEntryArgs { address: address };
    let maybe_allocation_of_input = store_as_json(&mut mem_stack, input);
    if let Err(err_code) = maybe_allocation_of_input {
        return Err(ZomeApiError::Internal(err_code.to_string()));
    }
    let allocation_of_input = maybe_allocation_of_input.unwrap();
    // Call Ribosome get_entry()
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_get_entry(allocation_of_input.encode() as u32);
    }
    // Deserialize complex result stored in memory
    let result = load_json(encoded_allocation_of_result as u32);
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    result.map_err(|hc_err| ZomeApiError::Internal(hc_err.to_string()))
}

/// Consumes three values, two of which are the addresses of entries, and one of which is a string that defines a
/// relationship between them, called a `tag`. Later, lists of entries can be looked up by using `get_links`. Entries
/// can only be looked up in the direction from the `base`, which is the first argument, to the `target`.
pub fn link_entries<S: Into<String>>(
    base: &HashString,
    target: &HashString,
    tag: S,
) -> Result<(), ZomeApiError> {
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };

    // Put args in struct and serialize into memory
    let input = LinkEntriesArgs {
        base: base.clone(),
        target: target.clone(),
        tag: tag.into(),
    };

    let allocation_of_input = store_as_json(&mut mem_stack, input)
        .map_err(|err_code| ZomeApiError::Internal(err_code.to_string()))?;

    let encoded_allocation_of_result: u32 =
        unsafe { hc_link_entries(allocation_of_input.encode() as u32) };
    let result = check_for_ribosome_error(encoded_allocation_of_result);
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    result
}

/// Not Yet Available
// Returns a DNA property, which are defined by the DNA developer.
// They are custom values that are defined in the DNA file
// that can be used in the zome code for defining configurable behaviors.
// (e.g. Name, Language, Description, Author, etc.).
pub fn property<S: Into<String>>(_name: S) -> ZomeApiResult<String> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Reconstructs an address of the given entry data.
/// This is the same value that would be returned if `entry_type_name` and `entry_value` were passed
/// to the `commit_entry` function and by which it would be retrievable from the DHT using `get_entry`.
/// This is often used to reconstruct an address of a `base` argument when calling `get_links`.
pub fn hash_entry<S: Into<String>>(
    entry_type_name: S,
    entry_value: serde_json::Value,
) -> ZomeApiResult<HashString> {
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };
    // Put args in struct and serialize into memory
    let input = HashEntryArgs {
        entry_type_name: entry_type_name.into(),
        entry_value: entry_value.to_string(),
    };
    let allocation_of_input = store_as_json(&mut mem_stack, input)
        .map_err(|err_code| ZomeApiError::Internal(err_code.to_string()))?;
    let encoded_allocation_of_result: u32 =
        unsafe { hc_hash_entry(allocation_of_input.encode() as u32) };
    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result = load_string(encoded_allocation_of_result as u32);
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    result
        .map(|hash_str| HashString::from(hash_str))
        .map_err(|err_code| ZomeApiError::Internal(err_code.to_string()))
}

/// Not Yet Available
pub fn sign<S: Into<String>>(_doc: S) -> ZomeApiResult<String> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn verify_signature<S: Into<String>>(
    _signature: S,
    _data: S,
    _pub_key: S,
) -> ZomeApiResult<bool> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn update_entry<S: Into<String>>(
    _entry_type: S,
    _entry: serde_json::Value,
    _replaces: HashString,
) -> ZomeApiResult<HashString> {
    // FIXME
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn update_agent() -> ZomeApiResult<HashString> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn remove_entry<S: Into<String>>(_entry: HashString, _message: S) -> ZomeApiResult<HashString> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Consumes two values, the first of which is the address of an entry, `base`,
/// and the second of which is a string, `tag`, used to describe the relationship between the `base`
/// and other entries you wish to lookup. Returns a list of addresses of other entries which matched
/// as being linked by the given `tag`.
/// Links are created in the first place using the Zome API function `link_entries`.
pub fn get_links<S: Into<String>>(base: &HashString, tag: S) -> ZomeApiResult<GetLinksResult> {
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };
    // Put args in struct and serialize into memory
    let input = GetLinksArgs {
        entry_address: base.clone(),
        tag: tag.into(),
    };
    let allocation_of_input = store_as_json(&mut mem_stack, input)
        .map_err(|err_code| ZomeApiError::Internal(err_code.to_string()))?;
    // Call Ribosome
    let encoded_allocation_of_result: u32 =
        unsafe { hc_get_links(allocation_of_input.encode() as u32) };
    // Deserialize complex result stored in memory
    let result: Result<GetLinksResult, HolochainError> =
        load_json(encoded_allocation_of_result as u32);
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    result.map_err(|hc_err| ZomeApiError::Internal(hc_err.to_string()))
}

/// Returns a list of entries from your local source chain, that match a given type.
/// entry_type_name: Specify type of entry to retrieve
/// limit: Max number of entries to retrieve
pub fn query(entry_type_name: &str, limit: u32) -> ZomeApiResult<QueryResult> {
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };
    // Put args in struct and serialize into memory
    let input = QueryArgs {
        entry_type_name: entry_type_name.to_string(),
        limit: limit,
    };
    let allocation_of_input = store_as_json(&mut mem_stack, input)
        .map_err(|err_code| ZomeApiError::Internal(err_code.to_string()))?;
    let encoded_allocation_of_result: u32 =
        unsafe { hc_query(allocation_of_input.encode() as u32) };
    // Deserialize complex result stored in memory
    let result: Result<QueryResult, HolochainError> =
        load_json(encoded_allocation_of_result as u32);
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    result.map_err(|hc_err| ZomeApiError::Internal(hc_err.to_string()))
}

/// Not Yet Available
pub fn send(_to: HashString, _message: serde_json::Value) -> ZomeApiResult<serde_json::Value> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn start_bundle(_timeout: usize, _user_param: serde_json::Value) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn close_bundle(_action: BundleOnClose) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}

//--------------------------------------------------------------------------------------------------
// Helpers
//--------------------------------------------------------------------------------------------------

pub fn check_for_ribosome_error(encoded_allocation: u32) -> Result<(), ZomeApiError> {
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
