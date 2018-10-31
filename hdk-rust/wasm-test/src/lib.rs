#![feature(try_from)]
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
use hdk::globals::G_MEM_STACK;
use holochain_wasm_utils::holochain_core_types::error::ZomeApiError;
use holochain_wasm_utils::{memory_allocation::*, memory_serialization::*};
use holochain_wasm_utils::holochain_core_types::json::JsonString;
use holochain_wasm_utils::holochain_core_types::json::RawString;
use holochain_wasm_utils::holochain_core_types::entry::SerializedEntry;
use holochain_wasm_utils::holochain_core_types::entry::Entry;
use holochain_wasm_utils::{
    holochain_core_types::{
        error::RibosomeErrorCode,
        hash::HashString,
        entry_type::EntryType,
    },
};
use holochain_wasm_utils::holochain_core_types::json::default_try_from_json;
use holochain_wasm_utils::holochain_core_types::error::HolochainError;
use std::convert::TryFrom;
use holochain_wasm_utils::api_serialization::get_entry::{GetEntryOptions, GetResultStatus};
use hdk::holochain_dna::zome::entry_types::Sharing;
use holochain_wasm_utils::holochain_core_types::json::default_to_json;
use holochain_wasm_utils::holochain_core_types::cas::content::Address;

#[no_mangle]
pub extern "C" fn handle_check_global() -> JsonString {
    hdk::AGENT_LATEST_HASH.clone().into()
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
        G_MEM_STACK =
            Some(SinglePageStack::from_encoded_allocation(encoded_allocation_of_input).unwrap());
    }

    // Deserialize and check for an encoded error
    let result = load_json(encoded_allocation_of_input as u32);
    if let Err(err_str) = result {
        hdk::debug(format!("ERROR ArgumentDeserializationFailed: {:?}", err_str)).expect("debug() must work");
        return RibosomeErrorCode::ArgumentDeserializationFailed as u32;
    }

    let serialized_entry: SerializedEntry = result.unwrap();
    hdk::debug(format!("SerializedEntry: {:?}", serialized_entry)).expect("debug() must work");
    let res = hdk::commit_entry(&serialized_entry.into());

    let res_obj = match res {
        Ok(hash_str) => {
            hdk::debug(format!("SUCCESS: {:?}", hash_str.clone().to_string())).expect("debug() must work");
            CommitOutputStruct {address: hash_str.to_string()}
        },
        Err(ZomeApiError::Internal(err_str)) => unsafe {
            hdk::debug(format!("ERROR ZomeApiError: {:?}", err_str)).expect("debug() must work");
            return store_as_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), err_str) as u32;
        },
        Err(e) => {
            hdk::debug(format!("ERROR unknown: {:?}", e)).expect("debug() must work");
            unreachable!();
        }
    };
    unsafe {
        return store_as_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), res_obj) as u32;
    }
}

#[derive(Deserialize, Serialize, Default)]
struct EntryStruct {
    stuff: String
}

fn handle_check_commit_entry_macro(entry_type: String, value: String) -> JsonString {
    let entry = Entry::new(&entry_type.into(), &value.into());
    let res = hdk::commit_entry(&entry);
    hdk::debug(format!("res: {:?}", res)).expect("debug() must work");

    JsonString::from(match res {
        Ok(hash_str) => json!({ "address": hash_str }),
        Err(ZomeApiError::ValidationFailed(msg)) => json!({ "validation failed": msg}),
        Err(ZomeApiError::Internal(err_str)) => json!({ "error": err_str}),
        Err(_) => unreachable!(),
    })
}

fn handle_check_get_entry_result(entry_hash: HashString) -> JsonString {
    let res = hdk::get_entry_result(entry_hash,GetEntryOptions{});
    match res {
        Ok(result) => match result.status {
            GetResultStatus::Found => {
                match result.maybe_serialized_entry {
                    Some(serialized_entry) => serialized_entry.into(),
                    None => unreachable!(),
                }
            },
            GetResultStatus::NotFound => json!({"got back no entry": true}).into(),
        }
        Err(ZomeApiError::Internal(err_str)) => json!({"get entry Err": err_str}).into(),
        Err(_) => unreachable!(),
    }
}

fn handle_check_get_entry(entry_hash: HashString) -> JsonString {
    let result : Result<Option<Entry>,ZomeApiError> = hdk::get_entry(entry_hash);
    match result {
        Ok(e) => match e {
            Some(entry) => entry.serialize().into(),
            None => JsonString::null(),
        },
        Err(err) => json!({"get entry Err": err.to_string()}).into(),
    }
}

fn handle_commit_validation_package_tester() -> JsonString {
    hdk::commit_entry(&Entry::new(&"validation_package_tester".into(), &RawString::from("test").into())).into()
}

fn handle_link_two_entries()-> JsonString {
    let entry1_result = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &json!({
        "stuff": "entry1"
    }).into()));

    if entry1_result.is_err() {
        return entry1_result.into()
    }

    let entry2_result = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &json!({
        "stuff": "entry2"
    }).into()));

    if entry2_result.is_err() {
        return entry2_result.into()
    }

    hdk::link_entries(&entry1_result.unwrap(), &entry2_result.unwrap(), "test-tag").into()
}

fn handle_links_roundtrip() -> JsonString {
    let entry1_hash = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &json!({
        "stuff": "entry1"
    }).into())).unwrap();
    let entry2_hash = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &json!({
        "stuff": "entry2"
    }).into())).unwrap();
    let entry3_hash = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &json!({
        "stuff": "entry3"
    }).into())).unwrap();


    hdk::link_entries(&entry1_hash, &entry2_hash, "test-tag").expect("Can't link?!");
    hdk::link_entries(&entry1_hash, &entry3_hash, "test-tag").expect("Can't link?!");

    JsonString::from(match hdk::get_links(&entry1_hash, "test-tag") {
        Ok(result) => format!("{{\"links\": {}}}", JsonString::from(result.links)),
        Err(error) => format!("{{\"error\": {}}}", JsonString::from(error)),
    })
}

fn handle_check_query() -> JsonString {
    // Query DNA entry
    let result = hdk::query(&EntryType::Dna.to_string(), 0);
    assert!(result.is_ok());
    assert!(result.unwrap().len() == 1);

    // Query AgentId entry
    let result = hdk::query(&EntryType::AgentId.to_string(), 0);
    assert!(result.is_ok());
    assert!(result.unwrap().len() == 1);

    // Query Zome entry
    let _ = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &json!({
        "stuff": "entry1"
    }).into())).unwrap();
    let result = hdk::query("testEntryType", 1);
    assert!(result.is_ok());
    assert!(result.unwrap().len() == 1);

    // Query Zome entries
    let _ = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &json!({
        "stuff": "entry2"
    }).into())).unwrap();
    let _ = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &json!({
        "stuff": "entry3"
    }).into())).unwrap();

    let result = hdk::query("testEntryType", 0);
    assert!(result.is_ok());
    assert!(result.unwrap().len() == 3);

    let result = hdk::query("testEntryType", 1);
    assert!(result.is_ok());

    result.unwrap().into()
}

fn handle_check_hash_app_entry() -> JsonString {
    // Setup
    let entry_value = JsonString::from(json!({
        "stuff": "entry1"
    }));
    let commit_hash = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &entry_value.clone())).unwrap();
    // Check bad entry type name
    let result = hdk::hash_entry(&Entry::new(&"bad".into(), &entry_value.clone()));
    assert!(result.is_err());
    // Check good entry type name
    let good_hash = hdk::hash_entry(&Entry::new(&"testEntryType".into(), &entry_value)).unwrap();
    assert!(commit_hash == good_hash);
    good_hash.into()
}

fn handle_check_hash_sys_entry() -> JsonString {
    // TODO
    json!({"result": "FIXME"}).into()
}

fn handle_check_call() -> JsonString {
    let empty_dumpty = json!({});
    hdk::debug(format!("empty_dumpty = {:?}", empty_dumpty)).ok();
    let maybe_hash = hdk::call("test_zome", "test_cap", "check_hash_app_entry", empty_dumpty.into());
    hdk::debug(format!("maybe_hash = {:?}", maybe_hash)).ok();
    let tmp = maybe_hash.unwrap();
    let hash = Address::try_from(tmp).unwrap();
    hdk::debug(format!("hash = {}", hash)).ok();
    hash.into()
}

#[derive(Serialize, Deserialize, Debug)]
struct HashStruct {
    address: String,
}

impl From<HashStruct> for JsonString {
    fn from(v: HashStruct) -> Self {
        default_to_json(v)
    }
}

impl TryFrom<JsonString> for HashStruct {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> Result<Self, Self::Error> {
        default_try_from_json(j)
    }
}

fn handle_check_call_with_args() -> JsonString {
    let arg_str = JsonString::from(r#"{ "entry_type_name": "testEntryType", "entry_content": "{\"stuff\": \"non fail\"}" }"#);
    let args = SerializedEntry::try_from(arg_str).unwrap();
    // let args =  json!(arg_str);
    hdk::debug(format!("args = {:?}", args)).ok();
    let maybe_hash = hdk::call("test_zome", "test_cap", "check_commit_entry_macro", args.into());
    hdk::debug(format!("maybe_hash = {:?}", maybe_hash)).ok();
    let tmp = maybe_hash.unwrap();
    let hash = HashStruct::try_from(tmp).unwrap();
    hdk::debug(format!("hash = {:?}", hash)).ok();
    hash.into()
}


#[derive(Serialize, Deserialize, Debug)]
struct TweetResponse {
    first: String,
    second: String,
}

impl From<TweetResponse> for JsonString {
    fn from(v: TweetResponse) -> Self {
        default_to_json(v)
    }
}

fn handle_send_tweet(author: String, content: String) -> JsonString {
    TweetResponse { first: author,  second: content}.into()
}

#[derive(Serialize, Deserialize)]
struct TestEntryType {
    stuff: String,
}

define_zome! {
    entries: [
        entry!(
            name: "testEntryType",
            description: "asdfda",
            sharing: Sharing::Public,
            native_type: TestEntryType,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainFull
            },

            validation: |entry: TestEntryType, _ctx: hdk::ValidationData| {
                (entry.stuff != "FAIL")
                    .ok_or_else(|| "FAIL content is not allowed".to_string())
            }
        ),

        entry!(
            name: "validation_package_tester",
            description: "asdfda",
            sharing: Sharing::Public,
            native_type: TestEntryType,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainFull
            },

            validation: |_entry: TestEntryType, ctx: hdk::ValidationData| {
                Err(serde_json::to_string(&ctx).unwrap())
            }
        )
    ]

    genesis: || { Ok(()) }

    functions: {
        test (Public) {
            check_global: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_check_global
            }

            check_commit_entry_macro: {
                inputs: |entry_type: String, value: String|,
                outputs: |result: JsonString|,
                handler: handle_check_commit_entry_macro
            }

            check_get_entry: {
                inputs: |entry_hash: HashString|,
                outputs: |result: JsonString|,
                handler: handle_check_get_entry
            }

            check_get_entry_result: {
                inputs: |entry_hash: HashString|,
                outputs: |result: JsonString|,
                handler: handle_check_get_entry_result
            }

            commit_validation_package_tester: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_commit_validation_package_tester
            }

            link_two_entries: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_link_two_entries
            }

            links_roundtrip: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_links_roundtrip
            }

            check_call: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_check_call
            }

            check_call_with_args: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_check_call_with_args
            }

            check_hash_app_entry: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_check_hash_app_entry
            }

            check_query: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_check_query
            }

            check_hash_sys_entry: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_check_hash_sys_entry
            }

            send_tweet: {
                inputs: |author: String, content: String|,
                outputs: |response: JsonString|,
                handler: handle_send_tweet
            }
        }
    }
}
