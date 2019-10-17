//! developers! Detailed references and examples can be found here for how to use the
//! HDK exposed functions to access powerful Holochain functions.

use crate::error::{ZomeApiError, ZomeApiResult};
use holochain_json_api::json::{default_to_json, JsonString, RawString};
use holochain_persistence_api::{cas::content::Address, hash::HashString};

use holochain_core_types::{
    dna::capabilities::CapabilityRequest,
    error::{RibosomeEncodedAllocation, RibosomeEncodingBits, ZomeApiInternalResult},
};
pub use holochain_wasm_utils::api_serialization::validation::*;
use holochain_wasm_utils::{
    api_serialization::ZomeApiGlobals,
    memory::{ribosome::load_ribosome_encoded_json, stack::WasmStack},
};

use init_globals::init_globals;
use std::convert::{TryFrom, TryInto};

mod bundle;
mod call;
mod capability;
mod commit_entry;
mod debug;
mod decrypt;
mod emit_signal;
mod encrypt;
mod entry_address;
mod entry_type_properties;
mod get_entry;
mod get_links;
mod keystore;
mod link_entries;
mod property;
mod query;
mod remove_link;
mod send;
mod sign;
mod sleep;
mod update_remove;
mod version;

pub use self::{
    bundle::{close_bundle, start_bundle},
    call::call,
    capability::{commit_capability_claim, commit_capability_grant},
    commit_entry::{commit_entry, commit_entry_result},
    debug::debug,
    decrypt::decrypt,
    emit_signal::emit_signal,
    encrypt::encrypt,
    entry_address::entry_address,
    entry_type_properties::entry_type_properties,
    get_entry::{get_entry, get_entry_history, get_entry_initial, get_entry_result},
    get_links::{
        get_links, get_links_and_load, get_links_count, get_links_count_with_options,
        get_links_result, get_links_with_options,
    },
    keystore::{
        keystore_derive_key, keystore_derive_seed, keystore_get_public_key, keystore_list,
        keystore_new_random, keystore_sign,
    },
    link_entries::link_entries,
    property::property,
    query::{query, query_result},
    remove_link::remove_link,
    send::send,
    sign::{sign, sign_one_time, verify_signature},
    sleep::sleep,
    update_remove::{remove_entry, update_agent, update_entry},
    version::version,
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
                        .map_err(|_| ZomeApiError::from(format!("Failed to deserialize return value: {}", result.value)))
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
    hc_crypto,Crypto;
    hc_sign_one_time, SignOneTime;
    hc_verify_signature, VerifySignature;
    hc_link_entries, LinkEntries;
    hc_remove_link, RemoveLink;
    hc_get_links, GetLinks;
    hc_get_links_count,GetLinksCount;
    hc_sleep, Sleep;
    hc_meta,Meta;
    hc_keystore_list, KeystoreList;
    hc_keystore_new_random, KeystoreNewRandom;
    hc_keystore_derive_seed, KeystoreDeriveSeed;
    hc_keystore_derive_key, KeystoreDeriveKey;
    hc_keystore_sign, KeystoreSign;
    hc_keystore_get_public_key, KeystoreGetPublicKey;
    hc_commit_capability_grant, CommitCapabilityGrant;
    hc_commit_capability_claim, CommitCapabilityClaim;
    hc_emit_signal, EmitSignal;
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
    pub static ref CAPABILITY_REQ: &'static Option<CapabilityRequest> = &GLOBALS.cap_request;

    /// The json string from the DNA top level properties field.
    /// Deserialize this into a serde_json::Value or a zome specific struct to access the fields
    pub static ref PROPERTIES: &'static JsonString = &GLOBALS.properties;

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
        default_to_json(*cap_request)
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
