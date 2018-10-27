use error::{ZomeApiError, ZomeApiResult};
use globals::*;
pub use holochain_wasm_utils::api_serialization::validation::*;
use holochain_wasm_utils::{
    api_serialization::{
        commit::{CommitEntryArgs, CommitEntryResult},
        get_entry::{GetEntryArgs, GetEntryOptions, GetEntryResult, GetResultStatus},
        get_links::{GetLinksArgs, GetLinksResult},
        link_entries::{LinkEntriesArgs, LinkEntriesResult},
        HashEntryArgs, QueryArgs, QueryResult, ZomeFnCallArgs,
    },
    holochain_core_types::hash::HashString,
    memory_allocation::*,
    memory_serialization::*,
};
use serde::de::DeserializeOwned;
use serde_json;

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

<<<<<<< HEAD
// HC.HashNotFound
#[derive(Clone, Debug, PartialEq)]
pub enum RibosomeError {
    RibosomeFailed(String),
    FunctionNotImplemented,
    HashNotFound,
    ValidationFailed(String),
}

impl RibosomeError {
    pub fn to_json(&self) -> serde_json::Value {
        json!({ "error": self.description() })
    }
}

impl fmt::Display for RibosomeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // @TODO seems weird to use debug for display
        // replacing {:?} with {} gives a stack overflow on to_string() (there's a test for this)
        // what is the right way to do this?
        // @see https://github.com/holochain/holochain-rust/issues/223
        write!(f, "{:?}", self)
    }
}

impl Error for RibosomeError {
    fn description(&self) -> &str {
        match self {
            RibosomeFailed(error_desc) => error_desc,
            FunctionNotImplemented => "Function not implemented",
            HashNotFound => "Hash not found",
            ValidationFailed(failure_desc) => failure_desc,
        }
    }
}

impl PartialEq<String> for RibosomeError {
    fn eq(&self, failure_msg: &String) -> bool {
        match self {
            RibosomeFailed(msg) => {
                if msg == failure_msg {
                    return true;
                }
                false
            }
            _ => false,
        }
    }
}

=======
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
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

    if let Err(err_str) = result {
        return Err(ZomeApiError::Internal(err_str));
    }
    let output = result.unwrap();

    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    Ok(output)
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

    // Call WASMI-able commit
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_commit_entry(allocation_of_input.encode() as u32);
    }
    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result = load_json(encoded_allocation_of_result as u32);

    if let Err(err_str) = result {
        return Err(ZomeApiError::Internal(err_str));
    }
    let output: CommitEntryResult = result.unwrap();

    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    if output.validation_failure.len() > 0 {
        Err(ZomeApiError::ValidationFailed(output.validation_failure))
    } else {
        Ok(HashString::from(output.address))
    }
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

    // Call WASMI-able get_entry
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_get_entry(allocation_of_input.encode() as u32);
    }
    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result = load_json(encoded_allocation_of_result as u32);
    if let Err(err_str) = result {
        return Err(ZomeApiError::Internal(err_str));
    }
    let result: GetEntryResult = result.unwrap();

    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    Ok(result)
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

    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result: LinkEntriesResult = load_json(encoded_allocation_of_result as u32)
        .map_err(|err_str| ZomeApiError::Internal(err_str))?;

    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    if result.ok {
        Ok(())
    } else {
        Err(ZomeApiError::Internal(result.error))
    }
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
    let result = load_string(encoded_allocation_of_result as u32)
        .map_err(|err_str| ZomeApiError::Internal(err_str))?;
    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    Ok(HashString::from(result))
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

/// Consumes two values, the first of which is the address of an entry, `base`, and the second of which is a string, `tag`,
/// used to describe the relationship between the `base` and other entries you wish to lookup. Returns a list of addresses of other
/// entries which matched as being linked by the given `tag`. Links are created in the first place using the Zome API function `link_entries`.
pub fn get_links<S: Into<String>>(base: &HashString, tag: S) -> ZomeApiResult<GetLinksResult> {
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };

    // Put args in struct and serialize into memory
    let input = GetLinksArgs {
        entry_address: base.clone(),
        tag: tag.into(),
    };

    let allocation_of_input = store_as_json(&mut mem_stack, input)
        .map_err(|err_code| ZomeApiError::Internal(err_code.to_string()))?;

    let encoded_allocation_of_result: u32 =
        unsafe { hc_get_links(allocation_of_input.encode() as u32) };

    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result: GetLinksResult = load_json(encoded_allocation_of_result as u32)
        .map_err(|err_str| ZomeApiError::Internal(err_str))?;

    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    if result.ok {
        Ok(result)
    } else {
        Err(ZomeApiError::Internal(result.error))
    }
}

/// Returns a list of entries from your local source chain, that match a given type.
/// entry_type_name: Specify type of entry to retrieve
/// limit: Max number of entries to retrieve
pub fn query(entry_type_name: &str, limit: u32) -> ZomeApiResult<Vec<HashString>> {
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
    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result: QueryResult = load_json(encoded_allocation_of_result as u32)
        .map_err(|err_str| ZomeApiError::Internal(err_str))?;
    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    Ok(result.hashes)
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
/****
//--------------------------------------------------------------------------------------------------
// UNIT TESTS
//--------------------------------------------------------------------------------------------------

/// Unit tests
#[cfg(test)]
mod test {
    use super::*;

    /**
     * Ribosome error handling unit tests
     */

    #[test]
    /// test that we can convert an error to a string
    fn test_to_string() {
        let err = RibosomeError::FunctionNotImplemented.to_string();
        assert_eq!(r#"FunctionNotImplemented"#, err)
    }

    #[test]
    /// test that we can get the description for an error
    fn test_description() {
        let err = RibosomeError::FunctionNotImplemented;
        assert_eq!("Function not implemented", err.description())
    }

    /**
     * property() unit tests
     */

    #[test]
    /// test that property() returns HashNotFound error for null key
    fn test_property_invalid() {
        // check whether function implemented
        let result = property("Name");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        // test empty property key parameter
        let result = property("");
        assert_ne!(Some(RibosomeError::HashNotFound), result.err());

        // test unknown property key parameter
        assert_eq!(
            r#"HashNotFound"#,
            property("unknown").err().unwrap().to_string()
        );
    }

    #[test]
    /// test that property() returns value for known key
    fn test_property_valid() {
        // check whether function implemented
        let result = property("Name");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME
        // test known property key parameter
        let result = property("Name");
        assert!(result.is_ok())
        ***/
    }

    /**
     * make_hash() unit tests
     */

    #[test]
    /// test that make_hash() returns value for array entry data
    fn test_make_hash_invalid() {
        // check whether function implemented
        let result = make_hash("", json!("test_data"));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        // test empty entry type parameter
        // TODO: is this the right error?
        let result = make_hash("", json!("test_data"));
        assert!(result.is_err());
        assert_eq!(Some(RibosomeError::HashNotFound), result.err());
    }

    #[test]
    /// test that make_hash() returns value for valid entry data
    fn test_make_hash_valid() {
        // check whether function implemented
        let result = make_hash("", json!("test_data"));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        //
        // test various data w/ valid (non-empty) entry type
        //
        let result = make_hash("test", json!(""));
        assert!(result.is_ok());

        let result = make_hash("test", json!("test"));
        assert!(result.is_ok());

        let result = make_hash("test", json!(1));
        assert!(result.is_ok());

        let result = make_hash("test", json!([1, 2, 3]));
        assert!(result.is_ok());

        let result = make_hash("test", serde_json::Value::Null);
        assert!(result.is_ok());

        let result = make_hash("test", serde_json::Value::Bool(true));
        assert!(result.is_ok());

        let result = make_hash("test", json!({"a": [1, 2, 3], "b": true}));
        assert!(result.is_ok());
    }

    /**
     * debug() unit tests
     */

    #[test]
    /// test that debug() returns ok for valid arguments
    fn test_debug() {
        /*** FIXME
        // check whether function implemented
        let result = debug("");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        let result = debug("test debug");
        assert!(result.is_ok());
        ***/
    }

    /**
     * call() unit tests
     */

    #[test]
    /// test that call() returns error for invalid arguments
    fn test_call_invalid() {
        // check whether function implemented
        let result = call("", "", "", json!(""));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        // test empty zome name parameter
        let result = call("", "test", "", json!("test"));
        assert!(result.is_err());
        // FIXME with proper error value
        // assert_eq!(Some( ?? ), result.err());

        // test empty capability name parameter
        let result = call("", "test", "", json!("test"));
        assert!(result.is_err());
        // FIXME with proper error value
        // assert_eq!(Some( ?? ), result.err());

        // test empty function name parameter
        let result = call("test", "", "", json!("test"));
        assert!(result.is_err());
        // FIXME with proper error value
        // assert_eq!(Some( ?? ), result.err());
    }

    #[test]
    /// test that call() returns value for valid arguments
    fn test_call_valid() {
        // check whether function implemented
        let result = call("", "", "", json!(""));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME
        // test valid zome, function, and argument(s) parameters
        // FIXME - need valid call arguments & expected return value
        let result = call("??", "??", json!("??"));
        assert!(result.is_ok());
        assert_eq!(Some(json!("??")), result.ok());
        ***/
    }

    /**
     * sign() unit tests
     */

    #[test]
    /// test that sign() returns value for valid arguments
    fn test_sign() {
        // check whether function implemented
        let result = sign("");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        // test sign empty data parameter
        let result = sign("");
        assert!(result.is_ok());
        assert_ne!("", result.ok().unwrap());

        // test sign non-empty data parameter
        let result = sign("test data");
        assert!(result.is_ok());
        assert_ne!(Some(String::from("")), result.ok());
    }

    /**
     * verify_signature() unit tests
     */

    #[test]
    /// test that verify_signature() returns error for invalid arguments
    fn test_verify_signature_invalid() {
        // check whether function implemented
        let result = verify_signature("", "", "");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        // test invalid (i.e., empty string) parameters
        // FIXME with proper error value
        let result = verify_signature("", "", "");
        assert!(result.is_err());

        /*** FIXME
        // get agent's own public key for verification
        let key_entry = get_entry(APP_AGENT_KEY_HASH.clone());
        assert!(key_entry.is_ok());
        let pub_key = key_entry.unwrap();
        assert_ne!(None, pub_key);
        let pub_key = pub_key.unwrap();

        // sign test data
        let data = "test data".to_string();
        let signed = sign(data.clone());
        assert!(signed.is_ok());
        let signature = signed.unwrap();

        // test invalid public key parameter
        // FIXME with proper error value
        let verified = verify_signature(signature.clone(), data.clone(), "bad key".to_string());
        assert!(verified.is_err());
        // FIXME with proper error value
        assert_eq!(Some(RibosomeError::??), verified.err());

        // test invalid signature parameter
        // FIXME with proper error value
        let verified = verify_signature("bad signature".to_string(), data, pub_key);
        assert!(verified.is_err());
        // FIXME with proper error value
        assert_eq!(Some(RibosomeError::??), verified.err());
        ***/
    }

    #[test]
    /// test that verify_signature() returns value for valid arguments
    fn test_verify_signature_valid() {
        // check whether function implemented
        let result = verify_signature("", "", "");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME
        // get agent's own public key for verification
        let key_entry = get_entry(APP_AGENT_KEY_HASH.clone());
        assert!(key_entry.is_ok());
        let pub_key = key_entry.unwrap();
        assert_ne!(None, pub_key);
        let pub_key = pub_key.unwrap();
       
        // sign test data
        let data = "test data".to_string();
        let signed = sign(data.clone());
        assert!(signed.is_ok());
        let signature = signed.unwrap();
       
        // test valid public key parameter
        let verified = verify_signature(signature.clone(), data.clone(), pub_key);
        assert!(verified.is_ok());
        assert_eq!(Some(true), verified.ok());
        ***/
    }

    /**
     * commit_entry() unit tests
     */

    #[test]
    /// test that commit_entry() returns error for invalid arguments
    fn test_commit_entry_invalid() {
        /*** FIXME
        // check whether function implemented
        let result = commit_entry("", json!(""));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        // invalid (i.e., empty string) arguments
        let result = commit_entry("", json!(""));
        assert!(result.is_err());
        // FIXME with proper error value
        assert_eq!(Some(RibosomeError::??), verified.err());
        ***/
    }

    #[test]
    /// test that commit_entry() returns ok for valid arguments
    fn test_commit_entry_valid() {
        /*** FIXME
        // check whether function implemented
        let result = commit_entry("", json!(""));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());
       
        // invalid (i.e., empty string) arguments
        let result = commit_entry("test", json!("test data"));
        assert!(result.is_ok());
        assert_ne!(None, result.ok());
        assert_ne!(Some(HashString::from(""), result.ok());
        ***/
    }

    /**
     * update_entry() unit tests
     */

    #[test]
    /// test that update_entry() returns error for invalid arguments
    fn test_update_entry_invalid() {
        // check whether function implemented
        let result = update_entry("test", json!(""), HashString::from(""));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        // test invalid invalid entry hash
        let result = update_entry("test", json!(""), HashString::from(""));
        assert!(result.is_err());
        assert_eq!(Some(RibosomeError::HashNotFound), result.err());

        /*** FIXME
        // test invalid entry type
        let test_entry = commit_entry("test", json!("test_data"));
        assert!(test_entry.is_ok());
        let test_entry = test_entry.ok();
        assert_ne!(None, test_entry);
        let result = update_entry("test", json!("test_data"), test_entry.unwrap());
        assert!(result.is_err());
        // FIXME with proper error value
        assert_eq!(Some(RibosomeError::??), result.err());
        ***/
    }

    #[test]
    /// test that update_entry() returns ok for valid arguments
    fn test_update_entry_valid() {
        // check whether function implemented
        let result = update_entry("test", json!(""), HashString::from(""));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME
        let test_entry = commit_entry("test", json!("test_data"));
        assert!(test_entry.is_ok());
        let test_entry = test_entry.ok();
        assert_ne!(None, test_entry);
        let test_entry = test_entry.unwrap();

        let result = update_entry("test", json!("update data"), test_entry.clone());
        assert!(result.is_ok());
        let result = result.ok();
        assert_ne!(None, result.clone());
        assert_ne!(Some(HashString::from(test_entry.clone())), result);
        ***/
    }

    /**
     * update_agent() unit tests
     */

    #[test]
    /// test that update_agent() returns ok
    fn test_update_agent() {
        // check whether function implemented
        let result = update_agent();
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        // test update agent
        let result = update_agent();
        assert!(result.is_ok());
        let result = result.ok();
        assert_ne!(Some(HashString::from("")), result);
    }

    //
    // remove_entry() unit tests
    //

    #[test]
    /// test that remove_entry() returns error for invalid arguments
    fn test_remove_entry_invalid() {
        // check whether function implemented
        let result = remove_entry(HashString::from(""), "");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME
        // test invalid (i.e., empty hash string) parameters
        let result = remove_entry(HashString::from(""), "remove_entry_invalid");
        assert!(result.is_err());
        assert_eq!(Some(RibosomeError::HashNotFound), result.err());

        // test invalid (i.e., empty message string) parameters
        let result = remove_entry(HashString::from(""), "");
        assert!(result.is_err());
        // FIXME with proper error value
        // assert_eq!(Some(RibosomeError::??), result.err());
        ***/
    }

    #[test]
    /// test that remove_entry() returns ok for valid arguments
    fn test_remove_entry_valid() {
        // check whether function implemented
        let result = remove_entry(HashString::from(""), "");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME
        // commit test entry
        let test_entry = commit_entry("test", json!("test data"));
        assert!(test_entry.is_ok());
        let test_entry = test_entry.unwrap();

        // test invalid (i.e., empty hash string) parameters
        let result = remove_entry(test_entry, "remove_entry_valid");
        assert!(result.is_ok());
        let result = result.ok();
        assert_ne!(Some(HashString::from("")), result);
        ***/
    }

    /**
     * get_entry() unit tests
     */

    #[test]
    /// test that get_entry() returns ok for valid arguments
    fn test_get_entry_valid() {
        /*** FIXME
        let result = get_entry(HashString::from(""));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());
        
        // commit test entry
        let test_entry = commit_entry("test", json!("test data"));
        assert!(test_entry.is_ok());
        let test_entry = test_entry.unwrap();

        // test get test entry
        let result = get_entry(test_entry);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(Some(String::from("test data")), result);
        ***/
    }

    #[test]
    /// test that get_entry() returns error for valid arguments
    fn test_get_entry_invalid() {
        /*** FIXME
        // check whether function implemented
        let result = get_entry(HashString::from(""));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        // test null entry hash parameter
        // FIXME with proper error value
        let result = get_entry(HashString::from(""));
        assert!(result.is_err());
        assert_eq!(Some(RibosomeError::HashNotFound), result.err());

        // commit and then remove test entry
         let test_entry = commit_entry("test", json!("test data"));
        assert!(test_entry.is_ok());
        let test_entry = test_entry.unwrap();
        
        let removed = remove_entry(test_entry, "remove test entry");
        assert!(removed.is_ok());
        let removed = removed.unwrap();

        // test get on removed test entry
        let result = get_entry(removed);
        assert!(result.is_err());
        assert_eq!(Some(RibosomeError::HashNotFound), result.err());
        ***/
    }

    /**
     * link_entries() unit tests
     */

    #[test]
    /// test that link_entries() returns error for invalid arguments
    fn test_link_entries_invalid() {
        // check whether function implemented
        let result = link_entries(HashString::from(""), HashString::from(""), "");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME
        // commit test entry 1
        let test_entry_1 = commit_entry("test", json!("test data 1"));
        assert!(test_entry_1.is_ok());
        let test_entry_1 = test_entry_1.unwrap();

        // test link w/ bad base argument
        let result = link_entries(HashString::from(""), test_entry_1.clone(), "test link bad base");
        assert!(result.is_err());
        assert_eq!(Some(RibosomeError::HashNotFound), result.err());

        // test link w/ bad target argument
        let result = link_entries(test_entry_1.clone(), HashString::from(""), "test link bad target");
        assert!(result.is_err());
        assert_eq!(Some(RibosomeError::HashNotFound), result.err());
        
        // commit test entry 2
        let test_entry_2 = commit_entry("test", json!("test data 2"));
        assert!(test_entry_2.is_ok());
        let test_entry_2 = test_entry_2.unwrap();

        // test link w/ bad tag argument
        let result = link_entries(test_entry_1.clone(), test_entry_2.clone(), "");
        assert!(result.is_err());
        assert_eq!(Some(RibosomeError::HashNotFound), result.err());
        ***/
    }

    #[test]
    /// test that link_entries() returns ok for valid arguments
    fn test_link_entries_valid() {
        // check whether function implemented
        let result = link_entries(HashString::from(""), HashString::from(""), "");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME
        // commit test entry 1
        let test_entry_1 = commit_entry("test", json!("test data 1"));
        assert!(test_entry_1.is_ok());
        let test_entry_1 = test_entry_1.unwrap();

        // commit test entry 2
        let test_entry_2 = commit_entry("test", json!("test data 2"));
        assert!(test_entry_2.is_ok());
        let test_entry_2 = test_entry_2.unwrap();

        let result = link_entries(test_entry_1, test_entry_2, "link entries");
        assert!(result.is_ok());
        ***/
    }

    //
    // get_links() unit tests
    //

    #[test]
    /// test that get_links() returns error for invalid arguments
    fn test_get_links_invalid() {
        // check whether function implemented
        let result = link_entries(HashString::from(""), HashString::from(""), "");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME
        // commit test entry 1
        let test_entry_1 = commit_entry("test", json!("test data 1"));
        assert!(test_entry_1.is_ok());
        let test_entry_1 = test_entry_1.unwrap();

        // commit test entry 2
        let test_entry_2 = commit_entry("test", json!("test data 2"));
        assert!(test_entry_2.is_ok());
        let test_entry_2 = test_entry_2.unwrap();

        // link test entries
        let linked = link_entries(test_entry_1.clone(), test_entry_2.clone(), "link entries");
        assert!(linked.is_ok());

        // test get links w/ null hash argument
        // FIXME with proper error value
        let result = get_links(HashString::from(""), "test link");
        assert!(result.is_err());
        // FIXME with proper error value
        // assert_eq!(Some(RibosomeError::??), result.err());

        // test get links w/ null tag argument
        assert!(get_links(test_entry_1.clone(), "").is_err());
        // FIXME with proper error value
        // assert_eq!(Some(RibosomeError::??), result.err());
        ***/
    }

    #[test]
    /// test that get_links() returns ok for valid arguments
    fn test_get_links_valid() {
        // check whether function implemented
        let result = link_entries(HashString::from(""), HashString::from(""), "");
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME
        // commit test entry 1
        let test_entry_1 = commit_entry("test", json!("test data 1"));
        assert!(test_entry_1.is_ok());
        let test_entry_1 = test_entry_1.unwrap();

        // commit test entry 2
        let test_entry_2 = commit_entry("test", json!("test data 2"));
        assert!(test_entry_2.is_ok());
        let test_entry_2 = test_entry_2.unwrap();

        // link test entries
        let linked = link_entries(test_entry_1.clone(), test_entry_2.clone(), "link entries");
        assert!(linked.is_ok());

        // test get links w/ null hash argument
        // FIXME with proper error value
        let result = get_links(HashString::from(""), "test link");
        assert!(result.is_ok());
        ***/
    }

    /**
     * query() unit tests
     */

    #[test]
    /// test query() returns error for invalid arguments
    fn test_query() {
        // check whether function implemented
        let result = query();
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME */
    }

    /**
     * send() unit tests
     */

    #[test]
    /// test send() returns error for invalid parameters
    fn test_send_invalid() {
        // check whether function implemented
        let result = send(HashString::from(""), json!(""));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        // test null destination hash argument
        let result = send(HashString::from(""), json!("test message"));
        assert!(result.is_err());
        // assert_eq!(Some(RibosomeError::??), result.err());
    }

    #[test]
    /// test send() returns ok for valid parameters
    fn test_send_valid() {
        // check whether function implemented
        let result = send(HashString::from(""), json!(""));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());

        /*** FIXME
        // test using own agent as destination hash argument
        let result = send(APP_AGENT_KEY_HASH.clone(), json!("test message"));
        assert!(result.is_ok());
        let result = result.ok();
        assert_ne!(Some(json!("")), result);
        ***/
    }

    /**
     * start_bundle() unit tests
     */

    #[test]
    /// test start_bundle() returns error for invalid parameters
    fn test_start_bundle() {
        /*** FIXME
        // check whether function implemented
        let result = start_bundle(0, json!(""));
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());
        ***/
    }

    /**
     * close_bundle() unit tests
     */

    #[test]
    /// test close_bundle() returns error for invalid parameters
    fn test_close_bundle() {
        /*** FIXME
        // check whether function implemented
        let result = close_bundle(BundleOnClose::Discard);
        assert_ne!(Some(RibosomeError::FunctionNotImplemented), result.err());
        ***/
    }
}
****/
