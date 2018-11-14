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
#[macro_use]
extern crate holochain_core_types_derive;

use boolinator::Boolinator;
use hdk::{
    error::{ZomeApiError, ZomeApiResult},
    globals::G_MEM_STACK,
    holochain_dna::zome::entry_types::Sharing,
};
use holochain_wasm_utils::{
    api_serialization::get_entry::GetEntryOptions,
    holochain_core_types::{
        cas::content::Address,
        entry::{Entry, SerializedEntry},
        entry_type::EntryType,
        error::{HolochainError, RibosomeErrorCode},
        json::{JsonString, RawString},
    },
    memory_allocation::*,
    memory_serialization::*,
};

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
            serde_json::to_string(&commit_output_struct)
                .expect("could not Jsonify CommitOutputStruct"),
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
        hdk::debug(format!("ERROR: {:?}", hc_err.to_string())).expect("debug() must work");
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

#[derive(Deserialize, Serialize, Default, Debug, DefaultJson)]
struct EntryStruct {
    stuff: String,
}

fn handle_check_commit_entry_macro(entry_type: String, value: String) -> JsonString {
    let entry = Entry::new(entry_type.into(), value);
    match hdk::commit_entry(&entry) {
        Ok(address) => address.into(),
        Err(e) => e.into(),
    }
}

fn handle_check_get_entry_result(entry_address: Address) -> JsonString {
    match hdk::get_entry_result(entry_address, GetEntryOptions {}) {
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
    hdk::commit_entry(&Entry::new(
        "validation_package_tester".into(),
        RawString::from("test"),
    )).into()
}

fn handle_link_two_entries() -> JsonString {
    let entry1_result = hdk::commit_entry(&Entry::new(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry1".into(),
        },
    ));

    if entry1_result.is_err() {
        return entry1_result.into();
    }

    let entry2_result = hdk::commit_entry(&Entry::new(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry2".into(),
        },
    ));

    if entry2_result.is_err() {
        return entry2_result.into();
    }

    hdk::link_entries(&entry1_result.unwrap(), &entry2_result.unwrap(), "test-tag").into()
}

fn handle_links_roundtrip() -> JsonString {
    let entry1_hash_result = hdk::commit_entry(&Entry::new(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry1".into(),
        },
    ));
    let entry1_address = match entry1_hash_result {
        Ok(hash) => hash,
        Err(_) => return entry1_hash_result.into(),
    };
    hdk::debug(format!("entry1_address: {:?}", entry1_address)).unwrap();

    let entry2_hash_result = hdk::commit_entry(&Entry::new(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry2".into(),
        },
    ));
    let entry2_address = match entry2_hash_result {
        Ok(hash) => hash,
        Err(_) => return entry2_hash_result.into(),
    };
    hdk::debug(format!("entry2_address: {:?}", entry2_address)).unwrap();

    let entry3_hash_result = hdk::commit_entry(&Entry::new(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry3".into(),
        },
    ));
    let entry3_address = match entry3_hash_result {
        Ok(hash) => hash,
        Err(_) => return entry3_hash_result.into(),
    };
    hdk::debug(format!("entry3_address: {:?}", entry3_address)).unwrap();

    let link_1_result = hdk::link_entries(&entry1_address, &entry2_address, "test-tag");
    let link_1 = match link_1_result {
        Ok(link) => link,
        Err(_) => return link_1_result.into(),
    };
    hdk::debug(format!("link_1: {:?}", link_1)).unwrap();

    let link_2_result = hdk::link_entries(&entry1_address, &entry3_address, "test-tag");
    let link_2 = match link_2_result {
        Ok(link) => link,
        Err(_) => return link_2_result.into(),
    };
    hdk::debug(format!("link_2: {:?}", link_2)).unwrap();

    hdk::get_links(&entry1_address, "test-tag").into()
}

fn handle_check_query() -> JsonString {
    fn err(s: &str) -> ZomeApiResult<Address> {
        Err(ZomeApiError::Internal(s.to_owned()))
    }

    // Query DNA entry
    let addresses = hdk::query(&EntryType::Dna.to_string(), 0, 0).unwrap();

    if !addresses.len() == 1 {
        return err("Dna Addresses not length 1").into();
    }

    // Query AgentId entry
    let addresses = hdk::query(&EntryType::AgentId.to_string(), 0, 0).unwrap();

    if !addresses.len() == 1 {
        return err("AgentId Addresses not length 1").into();
    }

    // Query unknown entry
    let addresses = hdk::query("bad_type", 0, 0).unwrap();

    if !addresses.len() == 0 {
        return err("bad_type Addresses not length 1").into();
    }

    // Query Zome entry
    let _ = hdk::commit_entry(&Entry::new(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry1".into(),
        },
    )).unwrap();
    let addresses = hdk::query("testEntryType", 0, 1).unwrap();

    if !addresses.len() == 1 {
        return err("testEntryType Addresses not length 1").into();
    }

    // Query Zome entries
    let _ = hdk::commit_entry(&Entry::new(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry2".into(),
        },
    )).unwrap();
    let _ = hdk::commit_entry(&Entry::new(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry3".into(),
        },
    )).unwrap();

    let addresses = hdk::query("testEntryType", 0, 0).unwrap();

    if !addresses.len() == 3 {
        return err("testEntryType Addresses not length 3").into();
    }

    hdk::query("testEntryType", 0, 1).unwrap().into()
}

fn handle_check_app_entry_address() -> JsonString {
    // Setup
    let entry_value = JsonString::from(TestEntryType {
        stuff: "entry1".into(),
    });
    let entry_type = EntryType::from("testEntryType");
    let entry = Entry::new(entry_type, entry_value.clone());

    let commit_result = hdk::commit_entry(&entry);
    if commit_result.is_err() {
        return commit_result.into();
    }

    // Check bad entry type name
    let bad_result = hdk::entry_address(&Entry::new("bad".into(), entry_value.clone()));
    if !bad_result.is_err() {
        return bad_result.into();
    }

    // Check good entry type name
    let entry_address_result = hdk::entry_address(&entry);

    if commit_result == entry_address_result {
        JsonString::from(entry_address_result.unwrap())
    } else {
        JsonString::from(ZomeApiError::from(format!(
            "commit result: {:?} hash result: {:?}",
            commit_result, bad_result
        )))
    }
}

fn handle_check_sys_entry_address() -> JsonString {
    // TODO
    json!({"result": "FIXME"}).into()
}

fn handle_check_call() -> JsonString {
    let empty_dumpty = json!({});
    hdk::debug(format!("empty_dumpty = {:?}", empty_dumpty)).ok();
    let maybe_hash = hdk::call(
        "test_zome",
        "test_cap",
        "check_app_entry_address",
        empty_dumpty.into(),
    );
    hdk::debug(format!("maybe_hash = {:?}", maybe_hash)).ok();
    match maybe_hash {
        Ok(hash) => hash.into(),
        Err(e) => e.into(),
    }
}

fn handle_check_call_with_args() -> JsonString {
    let args = hdk_test_entry().serialize();
    hdk::debug(format!("args = {:?}", args)).ok();

    let maybe_address = hdk::call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        args.into(),
    );
    hdk::debug(format!("maybe_address = {:?}", maybe_address)).ok();

    match maybe_address {
        Ok(address) => address.into(),
        Err(e) => e.into(),
    }
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct TweetResponse {
    first: String,
    second: String,
}

fn handle_send_tweet(author: String, content: String) -> JsonString {
    TweetResponse {
        first: author,
        second: content,
    }.into()
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct TestEntryType {
    stuff: String,
}

fn hdk_test_entry_type() -> EntryType {
    EntryType::from("testEntryType")
}

fn hdk_test_entry_value() -> TestEntryType {
    TestEntryType {
        stuff: "non fail".into(),
    }
}

fn hdk_test_entry() -> Entry {
    Entry::new(hdk_test_entry_type(), hdk_test_entry_value())
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

            check_app_entry_address: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_check_app_entry_address
            }

            check_query: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_check_query
            }

            check_sys_entry_address: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_check_sys_entry_address
            }

            send_tweet: {
                inputs: |author: String, content: String|,
                outputs: |response: JsonString|,
                handler: handle_send_tweet
            }
        }
    }
}
