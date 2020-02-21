#![feature(proc_macro_hygiene)]

use hdk::prelude::*;
use hdk_proc_macros::zome;

use hdk::{
    holochain_core_types::{
        signature::{Provenance, Signature},
    },
    holochain_wasm_utils::api_serialization::keystore::{KeyType, SeedType},
};

#[zome]
pub mod converse {

    #[init]
    pub fn init() {
        hdk::keystore_new_random("app_root_seed", 32)
            .map_err(|err|
                hdk::debug(format!("ignoring new seed generation because of error: {}",err))
            ).unwrap_or(());
        Ok(())
    }

    #[validate_agent]
    pub fn validate_agent(validation_data: EntryValidationData<AgentId>) {
        Ok(())
    }

    #[zome_fn("hc_public")]
    pub fn sign_message(key_id: String, message: String) -> ZomeApiResult<Signature> {
        if key_id == "" {
            hdk::sign(message).map(Signature::from)
        } else {
            hdk::keystore_sign(key_id, message).map(Signature::from)
        }
    }

    #[zome_fn("hc_public", "other")]
    pub fn verify_message(message: String, provenance: Provenance) -> ZomeApiResult<bool> {
        hdk::verify_signature(provenance, message)
    }

    #[zome_fn("hc_public")]
    pub fn add_key(src_id: String, dst_id: String) -> ZomeApiResult<JsonString> {
        let key_str = hdk::keystore_derive_key(src_id, dst_id, KeyType::Signing)?;
        Ok(JsonString::from_json(&key_str))
    }

    #[zome_fn("hc_public")]
    pub fn get_pubkey(src_id: String) -> ZomeApiResult<JsonString> {
        let key_str = hdk::keystore_get_public_key(src_id)?;
        Ok(JsonString::from_json(&key_str))
    }

    #[zome_fn("hc_public")]
    pub fn add_seed(src_id: String, dst_id: String, index: u64) -> ZomeApiResult<()> {
        hdk::keystore_derive_seed(src_id, dst_id, "mycntext".to_string(), index, SeedType::OneShot)
    }

    #[zome_fn("hc_public")]
    pub fn list_secrets() -> ZomeApiResult<Vec<String>> {
        hdk::keystore_list().map(|keystore_ids| keystore_ids.ids)
    }

}
