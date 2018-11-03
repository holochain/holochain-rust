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
        entry_type::EntryType,
    },
};
use holochain_wasm_utils::api_serialization::get_entry::{GetEntryOptions};
use hdk::holochain_dna::zome::entry_types::Sharing;
use holochain_wasm_utils::holochain_core_types::cas::content::Address;
use holochain_wasm_utils::holochain_core_types::json::default_to_json;

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
    if let Err(hc_err) = result {
        hdk::debug(&format!("ERROR: {:?}", hc_err.to_string())).expect("debug() must work");
        return RibosomeErrorCode::ArgumentDeserializationFailed as u32;
    }

    let serialized_entry: SerializedEntry = result.unwrap();
    hdk::debug(format!("SerializedEntry: {:?}", serialized_entry)).expect("debug() must work");
    let res = hdk::commit_entry(&serialized_entry.into());

    let res_obj: JsonString = match res {
        Ok(hash) => hash.into(),
        Err(e) => e.into(),
    };

    unsafe {
        return store_as_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), res_obj) as u32;
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
struct EntryStruct {
    stuff: String
}

impl From<EntryStruct> for JsonString {
    fn from(v: EntryStruct) -> Self {
        default_to_json(v)
    }
}

fn handle_check_commit_entry_macro(entry_type: String, value: String) -> JsonString {
    let entry = Entry::new(&entry_type.into(), &value.into());
    match hdk::commit_entry(&entry) {
        Ok(hash) => hash.into(),
        Err(e) => e.into(),
    }
}

fn handle_check_get_entry_result(entry_address: Address) -> JsonString {
    match hdk::get_entry_result(entry_address, GetEntryOptions{}) {
        Ok(result) => result.into(),
        Err(e) => e.into(),
    }
}

fn handle_check_get_entry(entry_address: Address) -> JsonString {
    match hdk::get_entry(entry_address) {
        Ok(result) => result.and_then(|entry| Some(entry.serialize())).into(),
        Err(e) => e.into(),
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
    let entry1_hash_result = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &EntryStruct{
        stuff: "entry1".into(),
    }.into()));
    let entry1_hash = match entry1_hash_result {
        Ok(hash) => hash,
        Err(_) => return entry1_hash_result.into(),
    };
    hdk::debug(format!("entry1_hash: {:?}", entry1_hash)).unwrap();

    let entry2_hash_result = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &EntryStruct{
        stuff: "entry2".into(),
    }.into()));
    let entry2_hash = match entry2_hash_result {
        Ok(hash) => hash,
        Err(_) => return entry2_hash_result.into(),
    };
    hdk::debug(format!("entry2_hash: {:?}", entry2_hash)).unwrap();

    let entry3_hash_result = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &EntryStruct{
        stuff: "entry3".into(),
    }.into()));
    let entry3_hash = match entry3_hash_result {
        Ok(hash) => hash,
        Err(_) => return entry3_hash_result.into(),
    };
    hdk::debug(format!("entry3_hash: {:?}", entry3_hash)).unwrap();

    let link_1_result = hdk::link_entries(&entry1_hash, &entry2_hash, "test-tag");
    let link_1 = match link_1_result {
        Ok(link) => link,
        Err(_) => return link_1_result.into(),
    };
    hdk::debug(format!("link_1: {:?}", link_1)).unwrap();

    let link_2_result = hdk::link_entries(&entry1_hash, &entry3_hash, "test-tag");
    let link_2 = match link_2_result {
        Ok(link) => link,
        Err(_) => return link_2_result.into(),
    };
    hdk::debug(format!("link_2: {:?}", link_2)).unwrap();

    hdk::get_links(&entry1_hash, "test-tag").into()
}

fn handle_check_query() -> JsonString {
    // Query DNA entry
    let result = hdk::query(&EntryType::Dna.to_string(), 0);

    assert!(result.is_ok());
    assert!(result.unwrap().addresses.len() == 1);

    // Query AgentId entry
    let result = hdk::query(&EntryType::AgentId.to_string(), 0);
    assert!(result.is_ok());
    assert!(result.unwrap().addresses.len() == 1);

    // Query unknown entry
    let result = hdk::query("bad_type", 0);
    assert!(result.is_ok());
    assert!(result.unwrap().addresses.len() == 0);

    // Query Zome entry
    let _ = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &json!({
        "stuff": "entry1"
    }).into())).unwrap();
    let result = hdk::query("testEntryType", 1);
    assert!(result.is_ok());
    assert!(result.unwrap().addresses.len() == 1);

    // Query Zome entries
    let _ = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &json!({
        "stuff": "entry2"
    }).into())).unwrap();
    let _ = hdk::commit_entry(&Entry::new(&"testEntryType".into(), &json!({
        "stuff": "entry3"
    }).into())).unwrap();

    let result = hdk::query("testEntryType", 0);
    assert!(result.is_ok());
    assert!(result.unwrap().addresses.len() == 3);

    let result = hdk::query("testEntryType", 1);
    assert!(result.is_ok());

    result.unwrap().into()
}

fn handle_check_hash_app_entry() -> JsonString {
    // Setup
    let entry_value = JsonString::from(TestEntryType{stuff: "entry1".into()});
    let entry_type = EntryType::from("testEntryType");
    let entry = Entry::new(&entry_type, &entry_value);

    let commit_result = hdk::commit_entry(&entry);
    if commit_result.is_err() {
        return commit_result.into();
    }

    // Check bad entry type name
    let bad_result = hdk::hash_entry(&Entry::new(&"bad".into(), &entry_value.clone()));
    if !bad_result.is_err() {
        return bad_result.into();
    }

    // Check good entry type name
    let hash_result = hdk::hash_entry(&entry);

    if commit_result == hash_result {
        JsonString::from(hash_result.unwrap())
    } else {
        JsonString::from(
            ZomeApiError::from(
                format!("commit result: {:?} hash result: {:?}", commit_result, hash_result)
            )
        )
    }
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
    match maybe_hash {
        Ok(hash) => hash.into(),
        Err(e) => e.into(),
    }
}

fn handle_check_call_with_args() -> JsonString {
    let args = hdk_test_entry().serialize();
    hdk::debug(format!("args = {:?}", args)).ok();

    let maybe_hash = hdk::call("test_zome", "test_cap", "check_commit_entry_macro", args.into());
    hdk::debug(format!("maybe_hash = {:?}", maybe_hash)).ok();

    match maybe_hash {
        Ok(hash) => hash.into(),
        Err(e) => e.into(),
    }
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

#[derive(Serialize, Deserialize, Debug)]
struct TestEntryType {
    stuff: String,
}

fn hdk_test_entry_type() -> EntryType {
    EntryType::from("testEntryType")
}

fn hdk_test_entry_value() -> TestEntryType {
    TestEntryType {stuff: "non fail".into()}
}

fn hdk_test_entry() -> Entry {
    Entry::new(&hdk_test_entry_type(), &hdk_test_entry_value().into())
}

impl From<TestEntryType> for JsonString {
    fn from(v: TestEntryType) -> Self {
        default_to_json(v)
    }
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
                inputs: |entry_address: Address|,
                outputs: |result: JsonString|,
                handler: handle_check_get_entry
            }

            check_get_entry_result: {
                inputs: |entry_address: Address|,
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
