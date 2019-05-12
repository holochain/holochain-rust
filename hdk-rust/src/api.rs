//! developers! Detailed references and examples can be found here for how to use the
//! HDK exposed functions to access powerful Holochain functions.

use crate::error::{ZomeApiError, ZomeApiResult};
use holochain_core_types::{
    cas::content::Address,
    dna::capabilities::CapabilityRequest,
    entry::{
        cap_entries::{CapFunctions, CapabilityType},
        Entry,
    },
    error::{RibosomeEncodedAllocation, RibosomeEncodingBits, ZomeApiInternalResult},
    signature::Provenance,
    time::Timeout,
};
pub use holochain_wasm_utils::api_serialization::validation::*;
use holochain_wasm_utils::{
    api_serialization::{
        capabilities::{CommitCapabilityClaimArgs, CommitCapabilityGrantArgs},
        commit_entry::{CommitEntryArgs, CommitEntryOptions, CommitEntryResult},
        get_entry::{
            EntryHistory, GetEntryArgs, GetEntryOptions, GetEntryResult, GetEntryResultType,
            StatusRequestKind,
        },
        get_links::{GetLinksArgs, GetLinksOptions, GetLinksResult},
        keystore::{
            KeyType, KeystoreDeriveKeyArgs, KeystoreDeriveSeedArgs, KeystoreGetPublicKeyArgs,
            KeystoreListResult, KeystoreNewRandomArgs, KeystoreSignArgs,
        },
        link_entries::LinkEntriesArgs,
        send::{SendArgs, SendOptions},
        sign::{OneTimeSignArgs, SignArgs, SignOneTimeResult},
        verify_signature::VerifySignatureArgs,
        QueryArgs, QueryArgsNames, QueryArgsOptions, QueryResult, UpdateEntryArgs, ZomeApiGlobals,
        ZomeFnCallArgs,
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

/// Call an exposed function from another zome or another (bridged) instance running
/// in the same conductor.
/// Arguments for the called function are passed and resturned as `JsonString`.
/// # Examples
/// Here are two example Zomes, where one performs a `call` into the other.
///
/// This first zome is the "callee"; i.e., the zome that receives the call, and is named `summer`.
/// because the call sums two numbers.
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
/// # use hdk::holochain_core_types::error::RibosomeEncodedValue;
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_query(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_call(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sign(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sign_one_time(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_verify_signature(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_send(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sleep(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_debug(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_links(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_link_entries(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_remove_link(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_list(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_new_random(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_seed(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_sign(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_get_public_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// #[no_mangle]
/// # pub fn hc_commit_capability_grant(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// #[no_mangle]
/// # pub fn hc_commit_capability_claim(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
///
/// # fn main() {
///
/// fn handle_sum(num1: u32, num2: u32) -> JsonString {
///     let sum = num1 + num2;
///     json!({"sum": sum.to_string()}).into()
/// }
///
/// define_zome! {
///     entries: []
///
///     genesis: || {
///         Ok(())
///     }
///
///     functions: [
///             sum: {
///                 inputs: |num1: u32, num2: u32|,
///                 outputs: |sum: JsonString|,
///                 handler: handle_sum
///             }
///     ]
///
///     traits: {
///         hc_public [sum]
///     }
/// }
///
/// # }
/// ```
///
/// This second zome is the "caller" that makes the call into the `summer` Zome.
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
/// # use hdk::holochain_core_types::error::RibosomeEncodedValue;
/// # use hdk::holochain_core_types::cas::content::Address;
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_query(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_call(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sign(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sign_one_time(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_verify_signature(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_send(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sleep(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_debug(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_links(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_link_entries(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_remove_link(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_list(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_new_random(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_seed(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_sign(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_get_public_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// #[no_mangle]
/// # pub fn hc_commit_capability_grant(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// #[no_mangle]
/// # pub fn hc_commit_capability_claim(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
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
///     hdk::call(hdk::THIS_INSTANCE, "summer", Address::from(hdk::PUBLIC_TOKEN.to_string()), "sum", call_input.into())
/// }
///
/// define_zome! {
///     entries: []
///
///     genesis: || {
///         Ok(())
///     }
///
///     functions: [
///             check_sum: {
///                 inputs: |num1: u32, num2: u32|,
///                 outputs: |sum: ZomeApiResult<JsonString>|,
///                 handler: handle_check_sum
///             }
///     ]
///
///     traits: {
///         hc_public [check_sum]
///     }
/// }
///
/// # }
/// ```
pub fn call<S: Into<String>>(
    instance_handle: S,
    zome_name: S,
    cap_token: Address,
    fn_name: S,
    fn_args: JsonString,
) -> ZomeApiResult<JsonString> {
    Dispatch::Call.with_input(ZomeFnCallArgs {
        instance_handle: instance_handle.into(),
        zome_name: zome_name.into(),
        cap_token,
        fn_name: fn_name.into(),
        fn_args: String::from(fn_args),
    })
}

/// Prints a string through the stdout of the running Conductor, and also
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
pub fn debug<J: Into<String>>(msg: J) -> ZomeApiResult<()> {
    let _: ZomeApiResult<()> = Dispatch::Debug.with_input(JsonString::from_json(&msg.into()));
    // internally returns RibosomeEncodedValue::Success which is a zero length allocation
    // return Ok(()) unconditionally instead of the "error" from success
    Ok(())
}

/// Attempts to commit an entry to the local source chain. The entry
/// will also be checked against the defined validation rules for that entry type.
/// If the entry type is defined as public, it will also be published to the DHT.
/// Returns either an address of the committed entry, or an error.
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
    commit_entry_result(entry, CommitEntryOptions::default()).map(|result| result.address())
}

/// Attempts to commit an entry to your local source chain. The entry
/// will have to pass the defined validation rules for that entry type.
/// If the entry type is defined as public, will also publish the entry to the DHT.
///
/// Additional provenances can be added to the commit using the options argument.
/// Returns a CommitEntryResult which contains the address of the committed entry.
pub fn commit_entry_result(
    entry: &Entry,
    options: CommitEntryOptions,
) -> ZomeApiResult<CommitEntryResult> {
    Dispatch::CommitEntry.with_input(CommitEntryArgs {
        entry: entry.clone(),
        options,
    })
}

/// Retrieves latest version of an entry from the local chain or the DHT, by looking it up using
/// the specified address.
/// Returns None if no entry exists at the specified address or
/// if the entry's status is DELETED.  Note that if the entry was updated, the value retrieved
/// may be of the updated entry which will have a different hash value.  If you need
/// to get the original value whatever the status, use [get_entry_initial](fn.get_entry_initial.html), or if you need to know
/// the address of the updated entry use [get_entry_result](fn.get_entry_result.html)
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

/// Returns the Entry at the exact address specified, whatever its status.
/// Returns None if no entry exists at the specified address.
pub fn get_entry_initial(address: &Address) -> ZomeApiResult<Option<Entry>> {
    let entry_result = get_entry_result(
        address,
        GetEntryOptions::new(StatusRequestKind::Initial, true, false, Default::default()),
    )?;
    Ok(entry_result.latest())
}

/// Return an EntryHistory filled with all the versions of the entry from the version at
/// the specified address to the latest.
/// Returns None if no entry exists at the specified address.
pub fn get_entry_history(address: &Address) -> ZomeApiResult<Option<EntryHistory>> {
    let entry_result = get_entry_result(
        address,
        GetEntryOptions::new(StatusRequestKind::All, true, false, Default::default()),
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

/// Adds a named, directed link between two entries on the DHT.
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
///         "authored_posts",
///     )?;
///
///     if let Some(in_reply_to_address) = in_reply_to {
///         // return with Err if in_reply_to_address points to missing entry
///         hdk::get_entry_result(&in_reply_to_address, GetEntryOptions { status_request: StatusRequestKind::All, entry: false, headers: false, timeout: Default::default() })?;
///         hdk::link_entries(&in_reply_to_address, &address, "comments", "comments")?;
///     }
///
///     Ok(address)
///
/// }
/// # }
/// ```
pub fn link_entries<S: Into<String>, TS: Into<String>>(
    base: &Address,
    target: &Address,
    tag: S,
    r#type: TS,
) -> Result<Address, ZomeApiError> {
    Dispatch::LinkEntries.with_input(LinkEntriesArgs {
        base: base.clone(),
        target: target.clone(),
        tag: tag.into(),
        r#type: r#type.into(),
    })
}

/// Commits a LinkRemove entry to your local source chain that marks a link as 'deleted' by setting
/// its status metadata to `Deleted` which gets published to the DHT.
/// Consumes three values, two of which are the addresses of entries, and one of which is a string that removes a
/// relationship between them, called a `tag`. Later, lists of entries.
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
/// pub fn handle_remove_link(content: String, in_reply_to: Option<Address>) -> ZomeApiResult<()> {
///
///     let post_entry = Entry::App("post".into(), Post{
///             content,
///             date_created: "now".into(),
///     }.into());
///
///     let address = hdk::commit_entry(&post_entry)?;
///
///     hdk::remove_link(
///         &AGENT_ADDRESS,
///         &address,
///         "authored_posts",
///         "authored_posts",
///     )?;
///
///
///     Ok(())
///
/// }
/// # }
/// ```
pub fn remove_link<S: Into<String>, TS: Into<String>>(
    base: &Address,
    target: &Address,
    tag: S,
    r#type: TS,
) -> Result<(), ZomeApiError> {
    Dispatch::RemoveLink.with_input(LinkEntriesArgs {
        base: base.clone(),
        target: target.clone(),
        tag: tag.into(),
        r#type: r#type.into(),
    })
}

/// Signs a string payload using the agent's private key.
/// Returns the signature as a string.
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
/// # use holochain_core_types::signature::{Provenance, Signature};
/// # use hdk::error::ZomeApiResult;
/// # fn main() {
/// pub fn handle_sign_message(message: String) -> ZomeApiResult<Signature> {
///    hdk::sign(message).map(Signature::from)
/// }
/// # }
/// ```
pub fn sign<S: Into<String>>(payload: S) -> ZomeApiResult<String> {
    Dispatch::Sign.with_input(SignArgs {
        payload: payload.into(),
    })
}

/// Signs a vector of payloads with a private key that is generated and shredded.
/// Returns the signatures of the payloads and the public key that can be used to verify the signatures.
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
/// # use holochain_core_types::signature::{Provenance, Signature};
/// # use hdk::error::ZomeApiResult;
/// # use hdk::holochain_wasm_utils::api_serialization::sign::{OneTimeSignArgs, SignOneTimeResult};
/// # fn main() {
/// pub fn handle_one_time_sign(key_id: String, message: String) -> ZomeApiResult<Signature> {
///    hdk::sign(message).map(Signature::from)
/// }
/// # }
/// ```
pub fn sign_one_time<S: Into<String>>(payloads: Vec<S>) -> ZomeApiResult<SignOneTimeResult> {
    let mut converted_payloads = Vec::new();
    for p in payloads {
        converted_payloads.push(p.into());
    }
    Dispatch::SignOneTime.with_input(OneTimeSignArgs {
        payloads: converted_payloads,
    })
}

/// Returns a list of the named secrets stored in the keystore.
pub fn keystore_list() -> ZomeApiResult<KeystoreListResult> {
    Dispatch::KeystoreList.without_input()
}

/// Creates a new random "root" Seed secret in the keystore
pub fn keystore_new_random<S: Into<String>>(dst_id: S, size: usize) -> ZomeApiResult<()> {
    Dispatch::KeystoreNewRandom.with_input(KeystoreNewRandomArgs {
        dst_id: dst_id.into(),
        size,
    })
}

/// Creates a new derived seed secret in the keystore, derived from a previously defined seed.
/// Accepts two arguments: the keystore ID of the previously defined seed, and a keystore ID for the newly derived seed.
pub fn keystore_derive_seed<S: Into<String>>(
    src_id: S,
    dst_id: S,
    context: S,
    index: u64,
) -> ZomeApiResult<()> {
    Dispatch::KeystoreDeriveSeed.with_input(KeystoreDeriveSeedArgs {
        src_id: src_id.into(),
        dst_id: dst_id.into(),
        context: context.into(),
        index,
    })
}

/// Creates a new derived key secret in the keystore derived from on a previously defined seed.
/// Accepts two arguments: the keystore ID of the previously defined seed, and a keystore ID for the newly derived key.
pub fn keystore_derive_key<S: Into<String>>(
    src_id: S,
    dst_id: S,
    key_type: KeyType,
) -> ZomeApiResult<String> {
    Dispatch::KeystoreDeriveKey.with_input(KeystoreDeriveKeyArgs {
        src_id: src_id.into(),
        dst_id: dst_id.into(),
        key_type,
    })
}

/// Signs a payload using a private key from the keystore.
/// Accepts one argument: the keystore ID of the desired private key.
pub fn keystore_sign<S: Into<String>>(src_id: S, payload: S) -> ZomeApiResult<String> {
    Dispatch::KeystoreSign.with_input(KeystoreSignArgs {
        src_id: src_id.into(),
        payload: payload.into(),
    })
}

/// Returns the public key of a key secret
/// Accepts one argument: the keystore ID of the desired public key.
/// Fails if the id is a Seed secret.
pub fn keystore_get_public_key<S: Into<String>>(src_id: S) -> ZomeApiResult<String> {
    Dispatch::KeystoreGetPublicKey.with_input(KeystoreGetPublicKeyArgs {
        src_id: src_id.into(),
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
/// # use holochain_core_types::error::RibosomeEncodedValue;
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_query(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_call(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sign(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sign_one_time(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_verify_signature(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_send(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sleep(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_debug(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_links(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_link_entries(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_remove_link(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_list(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_new_random(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_seed(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_sign(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_get_public_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// #[no_mangle]
/// # pub fn hc_commit_capability_grant(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// #[no_mangle]
/// # pub fn hc_commit_capability_claim(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
///
/// # fn main() {
/// fn handle_send_message(to_agent: Address, message: String) -> ZomeApiResult<String> {
///     // because the function signature of hdk::send is the same as the
///     // signature of handle_send_message we can just directly return its' result
///     hdk::send(to_agent, message, 60000.into())
/// }
///
/// define_zome! {
///    entries: []
///
///    genesis: || { Ok(()) }
///
///    receive: |from, payload| {
///        // if you want to serialize data as json to pass, use the json! serde macro
///        json!({
///            "key": "value"
///        }).to_string()
///    }
///
///    functions: [
///            send_message: {
///                inputs: |to_agent: Address, message: String|,
///                outputs: |response: ZomeApiResult<String>|,
///                handler: handle_send_message
///            }
///    ]
///
///     traits: {
///         hc_public [send_message]
///     }
///}
/// # }
/// ```
pub fn send(to_agent: Address, payload: String, timeout: Timeout) -> ZomeApiResult<String> {
    Dispatch::Send.with_input(SendArgs {
        to_agent,
        payload,
        options: SendOptions(timeout),
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

/// Adds a capability grant to the local chain
pub fn commit_capability_grant<S: Into<String>>(
    id: S,
    cap_type: CapabilityType,
    assignees: Option<Vec<Address>>,
    functions: CapFunctions,
) -> ZomeApiResult<Address> {
    Dispatch::CommitCapabilityGrant.with_input(CommitCapabilityGrantArgs {
        id: id.into(),
        cap_type,
        assignees,
        functions,
    })
}

/// Adds a capability claim to the local chain
pub fn commit_capability_claim<S: Into<String>>(
    id: S,
    grantor: Address,
    token: Address,
) -> ZomeApiResult<Address> {
    Dispatch::CommitCapabilityClaim.with_input(CommitCapabilityClaimArgs {
        id: id.into(),
        grantor,
        token,
    })
}
