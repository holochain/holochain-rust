//! developers! Detailed references and examples can be found here for how to use the
//! HDK exposed functions to access powerful Holochain functions.
use bitflags::bitflags;
use holochain_json_api::json::{default_to_json, JsonString, RawString};
use holochain_persistence_api::{cas::content::Address, hash::HashString};
use lazy_static::lazy_static;

use holochain_core_types::{dna::capabilities::CapabilityRequest};
pub use holochain_wasm_types::validation::*;
use holochain_wasm_types::ZomeApiGlobals;

use crate::init_globals::init_globals;

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
    version::{version, version_hash},
};
use holochain_wasmer_guest::*;

holochain_wasmer_guest::memory_externs!();

holochain_wasmer_guest::host_externs!(
    hc_debug,
    hc_commit_entry,
    hc_get_entry,
    hc_update_entry,
    hc_remove_entry,
    hc_init_globals,
    hc_call,
    hc_link_entries,
    hc_get_links,
    hc_get_links_count,
    hc_query,
    hc_entry_address,
    hc_send,
    hc_sleep,
    hc_remove_link,
    hc_sign,
    hc_encrypt,
    hc_decrypt,
    hc_sign_one_time,
    hc_verify_signature,
    hc_keystore_list,
    hc_keystore_new_random,
    hc_keystore_derive_seed,
    hc_keystore_derive_key,
    hc_keystore_sign,
    hc_keystore_get_public_key,
    hc_commit_capability_grant,
    hc_commit_capability_claim,
    hc_emit_signal,
    hc_meta,
    hc_timestamp
);

//--------------------------------------------------------------------------------------------------
// ZOME API GLOBAL VARIABLES
//--------------------------------------------------------------------------------------------------

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
