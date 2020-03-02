use hdk::prelude::*;
use hdk::{
    holochain_core_types::{
        signature::{Provenance, Signature},
    },
    holochain_wasm_types::keystore::KeyType,
};

pub fn handle_sign_message(key_id: String, message: String) -> ZomeApiResult<Signature> {
    if key_id == "" {
        hdk::sign(message).map(Signature::from)
    } else {
        hdk::keystore_sign(key_id, message).map(Signature::from)
    }
}

pub fn handle_verify_message(message: String, provenance: Provenance) -> ZomeApiResult<bool> {
    hdk::verify_signature(provenance, message)
}

pub fn handle_add_key(src_id: String, dst_id: String) -> ZomeApiResult<JsonString> {
    let key_str = hdk::keystore_derive_key(src_id, dst_id, KeyType::Signing)?;
    Ok(JsonString::from_json(&key_str))
}

pub fn handle_get_pubkey(src_id: String) -> ZomeApiResult<JsonString> {
    let key_str = hdk::keystore_get_public_key(src_id)?;
    Ok(JsonString::from_json(&key_str))
}

pub fn handle_add_seed(src_id: String, dst_id: String, index: u64) -> ZomeApiResult<()> {
    hdk::keystore_derive_seed(src_id, dst_id, "mycntext".to_string(), index)
}

pub fn handle_list_secrets() -> ZomeApiResult<Vec<String>> {
    hdk::keystore_list().map(|keystore_ids| keystore_ids.ids)
}

define_zome! {
    entries: []

    init: || {
        {
            hdk::keystore_new_random("app_root_seed", 32)
                .map_err(|err|
                    hdk::debug(format!("ignoring new seed generation because of error: {}",err))
                ).unwrap_or(());
            Ok(())
        }
    }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
        Ok(())
    }

    functions: [
        sign_message: {
            inputs: |key_id: String, message: String|,
            outputs: |result: ZomeApiResult<Signature>|,
            handler: handle_sign_message
        }

        verify_message: {
            inputs: |message: String, provenance: Provenance|,
            outputs: |result: ZomeApiResult<bool>|,
            handler: handle_verify_message
        }

        add_seed: {
            inputs: |src_id: String, dst_id: String, index: u64|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_add_seed
        }

        add_key: {
            inputs: |src_id: String, dst_id: String|,
            outputs: |result: ZomeApiResult<JsonString>|,
            handler: handle_add_key
        }

        get_pubkey: {
            inputs: |src_id: String|,
            outputs: |result: ZomeApiResult<JsonString>|,
            handler: handle_get_pubkey
        }

        list_secrets: {
            inputs: | |,
            outputs: |result: ZomeApiResult<Vec<String>>|,
            handler: handle_list_secrets
        }

    ]

    traits: {
        hc_public [sign_message, verify_message, add_key, add_seed, list_secrets, get_pubkey]
    }
}
