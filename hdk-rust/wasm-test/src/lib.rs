#[macro_use]
extern crate hdk;
extern crate holochain_wasm_utils;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate boolinator;

use boolinator::Boolinator;
use hdk::globals::G_MEM_STACK;
use holochain_wasm_utils::{
    holochain_core_types::error::RibosomeErrorCode,
    memory_serialization::*, memory_allocation::*
};
use hdk::RibosomeError;
use holochain_wasm_utils::holochain_core_types::json::JsonString;
use holochain_wasm_utils::holochain_core_types::entry::SerializedEntry;
use holochain_wasm_utils::holochain_core_types::cas::content::Address;

#[no_mangle]
pub extern "C" fn check_global(encoded_allocation_of_input: u32) -> u32 {
    unsafe {
        G_MEM_STACK = Some(SinglePageStack::from_encoded_allocation(encoded_allocation_of_input).unwrap());
    }
    #[allow(unused_must_use)]
    {
        hdk::debug(hdk::APP_NAME.to_owned());
        hdk::debug(hdk::APP_DNA_HASH.to_owned());
        hdk::debug(hdk::APP_AGENT_ID_STR.to_owned());
        hdk::debug(hdk::APP_AGENT_KEY_HASH.to_owned());
        hdk::debug(hdk::APP_AGENT_INITIAL_HASH.to_owned());
        hdk::debug(hdk::APP_AGENT_LATEST_HASH.to_owned());
    }

    unsafe {
        return store_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), hdk::APP_AGENT_LATEST_HASH.clone()) as u32;
    }
}


#[derive(Deserialize, Serialize, Default)]
struct CommitOutputStruct {
    address: String,
}

impl From<CommitOutputStruct> for JsonString {
    fn from(commit_output_struct: CommitOutputStruct) -> JsonString {
        JsonString::from(
            serde_json::to_string(&commit_output_struct).expect("could not Jsonify CommitOutputStruct")
        )
    }
}

#[no_mangle]
pub extern "C" fn check_commit_entry(encoded_allocation_of_input: u32) -> u32 {

    unsafe {
        G_MEM_STACK = Some(SinglePageStack::from_encoded_allocation(encoded_allocation_of_input).unwrap());
    }

    // Deserialize and check for an encoded error
    let result = load_json(encoded_allocation_of_input as u32);
    if let Err(e) = result {
        hdk::debug(format!("ERROR ArgumentDeserializationFailed: {:?}", e)).expect("debug() must work");
        return RibosomeErrorCode::ArgumentDeserializationFailed as u32;
    }

    let serialized_entry: SerializedEntry = result.unwrap();
    hdk::debug(format!("SerializedEntry: {:?}", serialized_entry)).expect("debug() must work");
    let res = hdk::commit_entry(&serialized_entry);

    let res_obj = match res {
        Ok(hash_str) => {
            hdk::debug(format!("SUCCESS: {:?}", hash_str.clone().to_string())).expect("debug() must work");
            CommitOutputStruct {address: hash_str.to_string()}
        },
        Err(RibosomeError::RibosomeFailed(err_str)) => {
            hdk::debug(format!("ERROR RibosomeFailed: {:?}", err_str)).expect("debug() must work");
            unsafe {
                return store_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), err_str) as u32;
            }
        },
       Err(e) => {
           hdk::debug(format!("ERROR unknown: {:?}", e)).expect("debug() must work");
           unreachable!();
       }
    };
    unsafe {
        return store_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), res_obj) as u32;
    }
}

//
zome_functions! {
    check_commit_entry_macro: |entry_type: String, value: String| {
        let serialized_entry = SerializedEntry::new(&entry_type, &value);
        let res = hdk::commit_entry(&serialized_entry);
        hdk::debug(format!("res: {:?}", res)).expect("debug() must work");
        res
    }
}

zome_functions! {
    check_get_entry: |entry_address: Address| {
        hdk::get_entry(entry_address)
    }
}


#[derive(Serialize, Deserialize)]
struct TweetResponse {
    first: String,
    second: String,
}

impl From<TweetResponse> for JsonString {
    fn from(tweet_response: TweetResponse) -> JsonString {
        JsonString::from(serde_json::to_string(&tweet_response).expect("could not Jsonify TweetResponse"))
    }
}

zome_functions! {
    send_tweet: |author: String, content: String| {
        TweetResponse { first: author,  second: content}
    }
}

#[derive(Serialize, Deserialize)]
// struct TestEntryType {
//     stuff: String,
// }
struct TestEntryType(String);

// #[derive(Serialize, Deserialize)]
// struct TestEntryTypeB(String);

validations! {
    [ENTRY] validate_testEntryType {
        [hdk::ValidationPackage::Entry]
        |entry: TestEntryType, _ctx: hdk::ValidationData| {
            (entry.0 != "FAIL")
                .ok_or_else(|| "FAIL content is not allowed".to_string())
        }
    }
}
