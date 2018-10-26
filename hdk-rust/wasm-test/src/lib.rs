#[macro_use]
extern crate hdk;
extern crate holochain_wasm_utils;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate boolinator;

use boolinator::Boolinator;
use hdk::{globals::G_MEM_STACK, error::ZomeApiError};
use holochain_wasm_utils::{
<<<<<<< HEAD
    holochain_core_types::error::RibosomeErrorCode,
    memory_serialization::*, memory_allocation::*
};
use hdk::RibosomeError;
use holochain_wasm_utils::holochain_core_types::json::JsonString;
use holochain_wasm_utils::holochain_core_types::entry::SerializedEntry;
use holochain_wasm_utils::holochain_core_types::cas::content::Address;
=======
    memory_allocation::*, memory_serialization::*,
    holochain_core_types::{
        error::RibosomeErrorCode,
        hash::HashString,
        entry_type::EntryType,
    },
};
use holochain_wasm_utils::api_serialization::get_entry::{GetEntryOptions, GetResultStatus};
use hdk::holochain_dna::zome::entry_types::Sharing;
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6

#[no_mangle]
pub extern "C" fn check_global(encoded_allocation_of_input: u32) -> u32 {
    hdk::global_fns::init_global_memory(encoded_allocation_of_input);
    #[allow(unused_must_use)]
    {
<<<<<<< HEAD
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
=======
        hdk::debug(&hdk::DNA_NAME);
        hdk::debug(&hdk::DNA_HASH.to_string());
        hdk::debug(&hdk::AGENT_ID_STR);
        hdk::debug(&hdk::AGENT_ADDRESS.to_string());
        hdk::debug(&hdk::AGENT_INITIAL_HASH.to_string());
        hdk::debug(&hdk::AGENT_LATEST_HASH.to_string());
    }
    return 0;
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
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
<<<<<<< HEAD
=======
    #[derive(Deserialize, Default)]
    struct CommitInputStruct {
        entry_type_name: String,
        entry_content: String,
    }
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6

    unsafe {
        G_MEM_STACK =
            Some(SinglePageStack::from_encoded_allocation(encoded_allocation_of_input).unwrap());
    }

    // Deserialize and check for an encoded error
    let result = load_json(encoded_allocation_of_input as u32);
<<<<<<< HEAD
    if let Err(e) = result {
        hdk::debug(format!("ERROR ArgumentDeserializationFailed: {:?}", e)).expect("debug() must work");
=======
    if let Err(err_str) = result {
        hdk::debug(&format!("ERROR: {:?}", err_str)).expect("debug() must work");
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
        return RibosomeErrorCode::ArgumentDeserializationFailed as u32;
    }

    let serialized_entry: SerializedEntry = result.unwrap();
    hdk::debug(format!("SerializedEntry: {:?}", serialized_entry)).expect("debug() must work");
    let res = hdk::commit_entry(&serialized_entry);

    let res_obj = match res {
<<<<<<< HEAD
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
=======
        Ok(hash_str) => CommitOutputStruct {
            address: hash_str.to_string(),
        },
        Err(ZomeApiError::Internal(err_str)) => unsafe {
            return store_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), err_str) as u32;
        },
        Err(_) => unreachable!(),
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
    };
    unsafe {
        return store_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), res_obj) as u32;
    }
}

#[derive(Deserialize, Serialize, Default)]
struct EntryStruct {
    stuff: String
}

//
<<<<<<< HEAD
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
=======

fn handle_check_commit_entry_macro(entry_type_name: String, entry_content: String) -> serde_json::Value {
    let entry_content = serde_json::from_str::<serde_json::Value>(&entry_content);
    let res = hdk::commit_entry(&entry_type_name, entry_content.unwrap());
    match res {
        Ok(hash_str) => json!({ "address": hash_str }),
        Err(ZomeApiError::ValidationFailed(msg)) => json!({ "validation failed": msg}),
        Err(ZomeApiError::Internal(err_str)) => json!({ "error": err_str}),
        Err(_) => unreachable!(),
    }
}

fn handle_check_get_entry_result(entry_hash: HashString) -> serde_json::Value {
    let res = hdk::get_entry_result(entry_hash,GetEntryOptions{});
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

fn handle_check_get_entry(entry_hash: HashString) -> serde_json::Value {
    let result : Result<Option<EntryStruct>,ZomeApiError> = hdk::get_entry(entry_hash);
    match result {
        Ok(e) => match e {
            Some(entry_value) => json!(entry_value),
            None => json!(null),
        },
        Err(err) => json!({"get entry Err": err.to_string()}),
    }
}

fn handle_commit_validation_package_tester() -> serde_json::Value {
    let res = hdk::commit_entry("validation_package_tester", json!({
        "stuff": "test"
    }));
    match res {
        Ok(hash_str) => json!({ "address": hash_str }),
        Err(ZomeApiError::ValidationFailed(msg)) => json!({ "validation failed": msg}),
        Err(ZomeApiError::Internal(err_str)) => json!({ "error": err_str}),
        Err(_) => unreachable!(),
    }
}

fn handle_link_two_entries()-> serde_json::Value {
    let entry1 = hdk::commit_entry("testEntryType", json!({
        "stuff": "entry1"
    }));
    let entry2 = hdk::commit_entry("testEntryType", json!({
        "stuff": "entry2"
    }));
    if entry1.is_err() {
        return json!({"error": &format!("Could not commit entry: {}", entry1.err().unwrap().to_string())})
    }
    if entry2.is_err() {
        return json!({"error": &format!("Could not commit entry: {}", entry2.err().unwrap().to_string())})
    }

    match hdk::link_entries(&entry1.unwrap(), &entry2.unwrap(), "test-tag") {
        Ok(()) => json!({"ok": true}),
        Err(error) => json!({"error": error.to_string()}),
    }
}

fn handle_links_roundtrip() -> serde_json::Value {
    let entry1_hash = hdk::commit_entry("testEntryType", json!({
        "stuff": "entry1"
    })).unwrap();
    let entry2_hash = hdk::commit_entry("testEntryType", json!({
        "stuff": "entry2"
    })).unwrap();
    let entry3_hash = hdk::commit_entry("testEntryType", json!({
        "stuff": "entry3"
    })).unwrap();


    hdk::link_entries(&entry1_hash, &entry2_hash, "test-tag").expect("Can't link?!");
    hdk::link_entries(&entry1_hash, &entry3_hash, "test-tag").expect("Can't link?!");

    match hdk::get_links(&entry1_hash, "test-tag") {
        Ok(result) => json!({"links": result.links}),
        Err(error) => json!({"error": error}),
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
    }
}

fn handle_check_query() -> serde_json::Value {
    // Query DNA entry
    let result = hdk::query(&EntryType::Dna.to_string(), 0);
    assert!(result.is_ok());
    assert!(result.unwrap().len() == 1);

    // Query AgentId entry
    let result = hdk::query(&EntryType::AgentId.to_string(), 0);
    assert!(result.is_ok());
    assert!(result.unwrap().len() == 1);

    // Query Zome entry
    let _ = hdk::commit_entry("testEntryType", json!({
        "stuff": "entry1"
    })).unwrap();
    let result = hdk::query("testEntryType", 1);
    assert!(result.is_ok());
    assert!(result.unwrap().len() == 1);

    // Query Zome entries
    let _ = hdk::commit_entry("testEntryType", json!({
        "stuff": "entry2"
    })).unwrap();
    let _ = hdk::commit_entry("testEntryType", json!({
        "stuff": "entry3"
    })).unwrap();

    let result = hdk::query("testEntryType", 0);
    assert!(result.is_ok());
    assert!(result.unwrap().len() == 3);

    let result = hdk::query("testEntryType", 1);
    assert!(result.is_ok());

    json!(result.unwrap())
}

fn handle_check_hash_app_entry() -> serde_json::Value {
    // Setup
    let entry_value = json!({
        "stuff": "entry1"
    });
    let commit_hash = hdk::commit_entry("testEntryType", entry_value.clone()).unwrap();
    // Check bad entry type name
    let result = hdk::hash_entry("bad", entry_value.clone());
    assert!(result.is_err());
    // Check good entry type name
    let good_hash = hdk::hash_entry("testEntryType", entry_value).unwrap();
    assert!(commit_hash == good_hash);
    json!(good_hash)
}

fn handle_check_hash_sys_entry() -> serde_json::Value {
    // TODO
    json!({"result": "FIXME"})
}

fn handle_check_call() -> serde_json::Value {
    let empty_dumpty = json!({});
    hdk::debug(&format!("empty_dumpty = {:?}", empty_dumpty)).ok();
    let maybe_hash = hdk::call("test_zome", "test_cap", "check_hash_app_entry", empty_dumpty);
    hdk::debug(&format!("maybe_hash = {:?}", maybe_hash)).ok();
    let tmp = maybe_hash.unwrap();
    let hash: &str = serde_json::from_str(&tmp).unwrap();
    hdk::debug(&format!("hash = {}", hash)).ok();
    json!(hash)
}

#[derive(Serialize, Deserialize, Debug)]
struct HashStruct {
    address: String,
}

fn handle_check_call_with_args() -> serde_json::Value {
    let arg_str = r#"{ "entry_type_name": "testEntryType", "entry_content": "{\"stuff\": \"non fail\"}" }"#;
    let args = serde_json::from_str::<serde_json::Value>(arg_str).unwrap();
    // let args =  json!(arg_str);
    hdk::debug(&format!("args = {:?}", args)).ok();
    let maybe_hash = hdk::call("test_zome", "test_cap", "check_commit_entry_macro", args);
    hdk::debug(&format!("maybe_hash = {:?}", maybe_hash)).ok();
    let tmp = maybe_hash.unwrap();
    let hash: HashStruct = serde_json::from_str(&tmp).unwrap();
    hdk::debug(&format!("hash = {:?}", hash)).ok();
    json!(hash)
}


#[derive(Serialize, Deserialize)]
struct TweetResponse {
    first: String,
    second: String,
}

<<<<<<< HEAD
impl From<TweetResponse> for JsonString {
    fn from(tweet_response: TweetResponse) -> JsonString {
        JsonString::from(serde_json::to_string(&tweet_response).expect("could not Jsonify TweetResponse"))
    }
}

zome_functions! {
    send_tweet: |author: String, content: String| {
        TweetResponse { first: author,  second: content}
    }
=======

fn handle_send_tweet(author: String, content: String) -> TweetResponse {
    TweetResponse { first: author,  second: content}
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
}

#[derive(Serialize, Deserialize)]
// struct TestEntryType {
//     stuff: String,
// }
struct TestEntryType(String);

// #[derive(Serialize, Deserialize)]
// struct TestEntryTypeB(String);

<<<<<<< HEAD
validations! {
    [ENTRY] validate_testEntryType {
        [hdk::ValidationPackage::Entry]
        |entry: TestEntryType, _ctx: hdk::ValidationData| {
            (entry.0 != "FAIL")
                .ok_or_else(|| "FAIL content is not allowed".to_string())
=======
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
            check_commit_entry_macro: {
                inputs: |entry_type_name: String, entry_content: String|,
                outputs: |result: serde_json::Value|,
                handler: handle_check_commit_entry_macro
            }

            check_get_entry: {
                inputs: |entry_hash: HashString|,
                outputs: |result: serde_json::Value|,
                handler: handle_check_get_entry
            }

            check_get_entry_result: {
                inputs: |entry_hash: HashString|,
                outputs: |result: serde_json::Value|,
                handler: handle_check_get_entry_result
            }

            commit_validation_package_tester: {
                inputs: | |,
                outputs: |result: serde_json::Value|,
                handler: handle_commit_validation_package_tester
            }

            link_two_entries: {
                inputs: | |,
                outputs: |result: serde_json::Value|,
                handler: handle_link_two_entries
            }

            links_roundtrip: {
                inputs: | |,
                outputs: |result: serde_json::Value|,
                handler: handle_links_roundtrip
            }

            check_call: {
                inputs: | |,
                outputs: |result: serde_json::Value|,
                handler: handle_check_call
            }

            check_call_with_args: {
                inputs: | |,
                outputs: |result: serde_json::Value|,
                handler: handle_check_call_with_args
            }

            check_hash_app_entry: {
                inputs: | |,
                outputs: |result: serde_json::Value|,
                handler: handle_check_hash_app_entry
            }

            check_query: {
                inputs: | |,
                outputs: |result: serde_json::Value|,
                handler: handle_check_query
            }

            check_hash_sys_entry: {
                inputs: | |,
                outputs: |result: serde_json::Value|,
                handler: handle_check_hash_sys_entry
            }

            send_tweet: {
                inputs: |author: String, content: String|,
                outputs: |response: TweetResponse|,
                handler: handle_send_tweet
            }
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
        }
    }
}
