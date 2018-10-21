#[macro_use]
extern crate hdk;
extern crate holochain_wasm_utils;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate boolinator;

use boolinator::Boolinator;
use hdk::{globals::G_MEM_STACK, error::ZomeApiError};
use holochain_wasm_utils::{
    memory_allocation::*, memory_serialization::*,
    holochain_core_types::{
        error::RibosomeErrorCode,
        hash::HashString,
    },
};
use holochain_wasm_utils::api_serialization::get_entry::{GetEntryOptions, GetResultStatus};

#[no_mangle]
pub extern "C" fn check_global(encoded_allocation_of_input: u32) -> u32 {
    hdk::global_fns::init_global_memory(encoded_allocation_of_input);
    #[allow(unused_must_use)]
    {
        hdk::debug(&hdk::APP_NAME);
        hdk::debug(&hdk::APP_DNA_HASH.to_string());
        hdk::debug(&hdk::APP_AGENT_ID_STR);
        hdk::debug(&hdk::APP_AGENT_KEY_HASH.to_string());
        hdk::debug(&hdk::APP_AGENT_INITIAL_HASH.to_string());
        hdk::debug(&hdk::APP_AGENT_LATEST_HASH.to_string());
    }
    return 0;
}

#[derive(Deserialize, Serialize, Default)]
struct CommitOutputStruct {
    address: String,
}

#[no_mangle]
pub extern "C" fn check_commit_entry(encoded_allocation_of_input: u32) -> u32 {
    #[derive(Deserialize, Default)]
    struct CommitInputStruct {
        entry_type_name: String,
        entry_content: String,
    }

    unsafe {
        G_MEM_STACK =
            Some(SinglePageStack::from_encoded_allocation(encoded_allocation_of_input).unwrap());
    }

    // Deserialize and check for an encoded error
    let result = load_json(encoded_allocation_of_input as u32);
    if let Err(err_str) = result {
        hdk::debug(&format!("ERROR: {:?}", err_str)).expect("debug() must work");
        return RibosomeErrorCode::ArgumentDeserializationFailed as u32;
    }

    let input: CommitInputStruct = result.unwrap();
    let entry_content = serde_json::from_str::<serde_json::Value>(&input.entry_content);
    let entry_content = entry_content.unwrap();
    let res = hdk::commit_entry(&input.entry_type_name, entry_content);

    let res_obj = match res {
        Ok(hash_str) => CommitOutputStruct {
            address: hash_str.to_string(),
        },
        Err(ZomeApiError::Internal(err_str)) => unsafe {
            return store_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), err_str) as u32;
        },
        Err(_) => unreachable!(),
    };
    unsafe {
        return store_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), res_obj) as u32;
    }
}

//
zome_functions! {
    check_commit_entry_macro: |entry_type_name: String, entry_content: String| {
        let entry_content = serde_json::from_str::<serde_json::Value>(&entry_content);
        let res = hdk::commit_entry(&entry_type_name, entry_content.unwrap());
        match res {
            Ok(hash_str) => json!({ "address": hash_str }),
            Err(ZomeApiError::ValidationFailed(msg)) => json!({ "validation failed": msg}),
            Err(ZomeApiError::Internal(err_str)) => json!({ "error": err_str}),
            Err(_) => unreachable!(),
        }
    }

    check_get_entry: |entry_hash: HashString| {
        let res = hdk::get_entry(entry_hash,GetEntryOptions{});
        match res {
            Ok(result) => match result.status {
                GetResultStatus::Found => {
                    let maybe_entry_value : Result<serde_json::Value, _> = serde_json::from_str(&result.entry);
                    match maybe_entry_value {
                        Ok(entry_value) => entry_value,
                        Err(err) => json!({"error trying deserialize entry": err.to_string()}),
                    }
                },
                GetResultStatus::NotFound => json!({"got back no entry": true}),
            }
            Err(ZomeApiError::Internal(err_str)) => json!({"get entry Err": err_str}),
            Err(_) => unreachable!(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TweetResponse {
    first: String,
    second: String,
}

zome_functions! {
    send_tweet: |author: String, content: String| {

        TweetResponse { first: author,  second: content}
    }
}

#[derive(Serialize, Deserialize)]
struct TestEntryType {
    stuff: String,
}

validations! {
    [ENTRY] validate_testEntryType {
        [hdk::ValidationPackage::Entry]
        |entry: TestEntryType, _ctx: hdk::ValidationData| {
            (entry.stuff != "FAIL")
                .ok_or_else(|| "FAIL content is not allowed".to_string())
        }
    }
}
