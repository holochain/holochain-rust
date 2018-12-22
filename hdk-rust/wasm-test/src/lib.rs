#![feature(try_from)]
#[macro_use]
extern crate hdk;
extern crate holochain_wasm_utils;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate boolinator;
#[macro_use]
extern crate holochain_core_types_derive;

pub mod handle_crud;

use boolinator::Boolinator;
use handle_crud::{
    handle_remove_entry_ok, handle_remove_modified_entry_ok, handle_update_entry_ok,
};
use hdk::{
    error::{ZomeApiError, ZomeApiResult},
    globals::G_MEM_STACK,
};
use holochain_wasm_utils::{
    api_serialization::{
        get_entry::{GetEntryOptions, GetEntryResult},
        get_links::GetLinksResult,
        query::QueryArgsNames,
    },
    holochain_core_types::{
        cas::content::{Address, AddressableContent},
        dna::entry_types::Sharing,
        entry::{
            entry_type::{AppEntryType, EntryType},
            AppEntryValue, Entry,
        },
        error::{HolochainError, RibosomeErrorCode},
        json::{JsonString, RawString},
    },
    memory_allocation::*,
    memory_serialization::*,
};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct TestEntryType {
    stuff: String,
}

#[derive(Deserialize, Serialize, Default, Debug, DefaultJson)]
struct CommitOutputStruct {
    address: String,
}

#[derive(Deserialize, Serialize, Default, Debug, DefaultJson)]
struct EntryStruct {
    stuff: String,
}

#[no_mangle]
pub extern "C" fn handle_check_global() -> Address {
    hdk::AGENT_LATEST_HASH.clone()
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

    let entry: Entry = result.unwrap();
    hdk::debug(format!("Entry: {:?}", entry)).expect("debug() must work");
    let res = hdk::commit_entry(&entry.into());

    let res_obj: JsonString = match res {
        Ok(hash) => hash.into(),
        Err(e) => e.into(),
    };

    unsafe {
        return store_as_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), res_obj) as u32;
    }
}

fn handle_check_commit_entry_macro(entry: Entry) -> ZomeApiResult<Address> {
    hdk::commit_entry(&entry)
}

fn handle_check_get_entry_result(entry_address: Address) -> ZomeApiResult<GetEntryResult> {
    hdk::get_entry_result(entry_address, GetEntryOptions::default())
}

fn handle_check_get_entry(entry_address: Address) -> ZomeApiResult<Option<Entry>> {
    hdk::get_entry(entry_address)
}

fn handle_commit_validation_package_tester() -> ZomeApiResult<Address> {
    hdk::commit_entry(&Entry::App(
        "validation_package_tester".into(),
        RawString::from("test").into(),
    ))
}

fn handle_link_two_entries() -> ZomeApiResult<()> {
    let entry_1 = Entry::App(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry1".into(),
        }
        .into(),
    );
    hdk::commit_entry(&entry_1)?;

    let entry_2 = Entry::App(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry2".into(),
        }
        .into(),
    );

    hdk::commit_entry(&entry_2)?;

    hdk::link_entries(&entry_1.address(), &entry_2.address(), "test-tag")
}

fn handle_links_roundtrip_create() -> ZomeApiResult<Address> {
    let entry_1 = Entry::App(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry1".into(),
        }
        .into(),
    );
    hdk::commit_entry(&entry_1)?;

    let entry_2 = Entry::App(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry2".into(),
        }
        .into(),
    );
    hdk::commit_entry(&entry_2)?;

    let entry_3 = Entry::App(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry3".into(),
        }
        .into(),
    );
    hdk::commit_entry(&entry_3)?;

    hdk::link_entries(&entry_1.address(), &entry_2.address(), "test-tag")?;
    hdk::link_entries(&entry_1.address(), &entry_3.address(), "test-tag")?;
    Ok(entry_1.address())
}

fn handle_links_roundtrip_get(address: Address) -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&address, "test-tag")
}

fn handle_check_query() -> ZomeApiResult<Vec<Address>> {
    println!("handle_check_query");
    fn err(s: &str) -> ZomeApiResult<Vec<Address>> {
        Err(ZomeApiError::Internal(s.to_owned()))
    }

    // Query DNA entry; EntryTypes will convert into the appropriate single-name enum type
    let addresses = hdk::query(EntryType::Dna.into(), 0, 0).unwrap();

    if !addresses.len() == 1 {
        return err("Dna Addresses not length 1");
    }

    // Query AgentId entry
    let addresses = hdk::query(QueryArgsNames::QueryList(vec![EntryType::AgentId.to_string()]), 0, 0).unwrap();

    if !addresses.len() == 1 {
        return err("AgentId Addresses not length 1");
    }

    // Query unknown entry; An &str will convert to a QueryArgsNames::QueryName
    let addresses = hdk::query("bad_type".into(), 0, 0).unwrap();

    if !addresses.len() == 0 {
        return err("bad_type Addresses not length 1");
    }

    // Query Zome entry
    let _ = hdk::commit_entry(&Entry::App(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry1".into(),
        }
        .into(),
    ))
    .unwrap();
    let addresses = hdk::query(QueryArgsNames::QueryName("testEntryType".to_string()), 0, 1).unwrap();

    if !addresses.len() == 1 {
        return err("testEntryType Addresses not length 1");
    }

    // Query Zome entries
    let _ = hdk::commit_entry(&Entry::App(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry2".into(),
        }
        .into(),
    ))
    .unwrap();
    let _ = hdk::commit_entry(&Entry::App(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry3".into(),
        }
        .into(),
    ))
    .unwrap();

    let addresses = hdk::query("testEntryType".into(), 0, 0).unwrap();

    if !addresses.len() == 3 {
        return err("testEntryType Addresses not length 3");
    }

    // See if we can get all System EntryTypes, and then System + testEntryType
    let addresses = hdk::query("[%]*".into(), 0, 0).unwrap();
    if !addresses.len() == 2 {
        return err("System Addresses not length 3");
    }
    let addresses = hdk::query(vec!["[%]*","testEntryType"].into(), 0, 0).unwrap();
    if !addresses.len() == 5 {
        return err("System Addresses not length 3");
    }

    hdk::query(QueryArgsNames::QueryName("testEntryType".to_string()), 0, 1)
}

fn handle_check_app_entry_address() -> ZomeApiResult<Address> {
    // Setup
    let entry_value = AppEntryValue::from(TestEntryType {
        stuff: "entry1".into(),
    });
    let entry_type = AppEntryType::from("testEntryType");
    let entry = Entry::App(entry_type, entry_value.clone());

    let commit_result = hdk::commit_entry(&entry);
    if commit_result.is_err() {
        return commit_result.into();
    }

    // Check bad entry type name
    let bad_result = hdk::entry_address(&Entry::App("bad".into(), entry_value.clone()));
    if !bad_result.is_err() {
        return bad_result.into();
    }

    // Check good entry type name
    let entry_address_result = hdk::entry_address(&entry);

    if commit_result == entry_address_result {
        entry_address_result
    } else {
        Err(ZomeApiError::from(format!(
            "commit result: {:?} hash result: {:?}",
            commit_result, bad_result
        )))
    }
}

// fn handle_check_sys_entry_address() -> JsonString {
//     // TODO
//     json!({"result": "FIXME"}).into()
// }

fn handle_check_call() -> ZomeApiResult<JsonString> {
    let empty_dumpty = JsonString::empty_object();
    hdk::debug(format!("empty_dumpty = {:?}", empty_dumpty))?;

    let maybe_hash = hdk::call(
        hdk::THIS_INSTANCE,
        "test_zome",
        "test_cap",
        "test_token",
        "check_app_entry_address",
        empty_dumpty,
    );
    hdk::debug(format!("maybe_hash = {:?}", maybe_hash))?;

    maybe_hash
}

fn handle_check_call_with_args() -> ZomeApiResult<JsonString> {
    #[derive(Serialize, Deserialize, Debug, DefaultJson)]
    struct CommitEntryInput {
        entry: Entry,
    }

    hdk::call(
        hdk::THIS_INSTANCE,
        "test_zome",
        "test_cap",
        "test_token",
        "check_commit_entry_macro",
        JsonString::from(CommitEntryInput {
            entry: hdk_test_entry(),
        }),
    )
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct TweetResponse {
    first: String,
    second: String,
}

fn handle_send_tweet(author: String, content: String) -> TweetResponse {
    TweetResponse {
        first: author,
        second: content,
    }
}

fn handle_link_validation(stuff1: String, stuff2: String) -> JsonString {
    let app_entry_type = AppEntryType::from("link_validator");
    let entry_value1 = JsonString::from(TestEntryType { stuff: stuff1 });
    let entry_value2 = JsonString::from(TestEntryType { stuff: stuff2 });
    let entry1 = Entry::App(app_entry_type.clone(), entry_value1.clone());
    let entry2 = Entry::App(app_entry_type.clone(), entry_value2.clone());

    let _ = hdk::commit_entry(&entry1);
    let _ = hdk::commit_entry(&entry2);

    JsonString::from(hdk::link_entries(
        &entry1.address(),
        &entry2.address(),
        "longer",
    ))
}

fn hdk_test_app_entry_type() -> AppEntryType {
    AppEntryType::from("testEntryType")
}

fn hdk_test_entry_value() -> AppEntryValue {
    TestEntryType {
        stuff: "non fail".into(),
    }
    .into()
}

fn hdk_test_entry() -> Entry {
    Entry::App(hdk_test_app_entry_type(), hdk_test_entry_value())
}

fn handle_send_message(to_agent: Address, message: String) -> ZomeApiResult<String>  {
    hdk::send(to_agent, message)
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
            },

            links: [
                to!(
                    "testEntryType",
                    tag: "test-tag",
                    validation_package: || {
                        hdk::ValidationPackageDefinition::ChainFull
                    },
                    validation: |source: Address, target: Address, ctx: hdk::ValidationData | {
                        Ok(())
                    }
                )
            ]
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
        ),

        entry!(
            name: "link_validator",
            description: "asdfda",
            sharing: Sharing::Public,
            native_type: TestEntryType,

            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },

            validation: |_entry: TestEntryType, ctx: hdk::ValidationData| {
                Ok(())
            },

            links: [
                to!(
                    "link_validator",
                    tag: "longer",
                    validation_package: || {
                        hdk::ValidationPackageDefinition::Entry
                    },
                    validation: |base: Address, target: Address, ctx: hdk::ValidationData | {
                        let base = match hdk::get_entry(base)? {
                            Some(entry) => match entry {
                                Entry::App(_, test_entry) => TestEntryType::try_from(test_entry)?,
                                _ => Err("System entry found")?
                            },
                            None => Err("Base not found")?,
                        };

                        let target = match hdk::get_entry(target)? {
                            Some(entry) => match entry {
                                Entry::App(_, test_entry) => TestEntryType::try_from(test_entry)?,
                                _ => Err("System entry found")?,
                            }
                            None => Err("Target not found")?,
                        };

                        (target.stuff.len() > base.stuff.len())
                            .ok_or("Target stuff is not longer".to_string())
                    }

                )
            ]
        )
    ]

    genesis: || { Ok(()) }

    receive: |payload| {
        format!("Received: {}", payload)
    }

    functions: {
        test (Public) {
            check_global: {
                inputs: | |,
                outputs: |agent_latest_hash: Address|,
                handler: handle_check_global
            }

            check_commit_entry_macro: {
                inputs: |entry: Entry|,
                outputs: |result: ZomeApiResult<Address>|,
                handler: handle_check_commit_entry_macro
            }

            check_get_entry: {
                inputs: |entry_address: Address|,
                outputs: |result: ZomeApiResult<Option<Entry>>|,
                handler: handle_check_get_entry
            }

            check_get_entry_result: {
                inputs: |entry_address: Address|,
                outputs: |result: ZomeApiResult<GetEntryResult>|,
                handler: handle_check_get_entry_result
            }

            commit_validation_package_tester: {
                inputs: | |,
                outputs: |result: ZomeApiResult<Address>|,
                handler: handle_commit_validation_package_tester
            }

            link_two_entries: {
                inputs: | |,
                outputs: |result: ZomeApiResult<()>|,
                handler: handle_link_two_entries
            }

            links_roundtrip_create: {
                inputs: | |,
                outputs: |result: ZomeApiResult<Address>|,
                handler: handle_links_roundtrip_create
            }

            links_roundtrip_get: {
                inputs: |address: Address|,
                outputs: |result: ZomeApiResult<GetLinksResult>|,
                handler: handle_links_roundtrip_get
            }

            link_validation: {
                inputs: |stuff1: String, stuff2: String|,
                outputs: |result: JsonString|,
                handler: handle_link_validation
            }

            check_call: {
                inputs: | |,
                outputs: |result: ZomeApiResult<JsonString>|,
                handler: handle_check_call
            }

            check_call_with_args: {
                inputs: | |,
                outputs: |result: ZomeApiResult<JsonString>|,
                handler: handle_check_call_with_args
            }

            check_app_entry_address: {
                inputs: | |,
                outputs: |result: ZomeApiResult<Address>|,
                handler: handle_check_app_entry_address
            }

            check_query: {
                inputs: | |,
                outputs: |result: ZomeApiResult<Vec<Address>>|,
                handler: handle_check_query
            }

            // check_sys_entry_address: {
            //     inputs: | |,
            //     outputs: |result: JsonString|,
            //     handler: handle_check_sys_entry_address
            // }

            update_entry_ok: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_update_entry_ok
            }

            remove_entry_ok: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_remove_entry_ok
            }

            remove_modified_entry_ok: {
                inputs: | |,
                outputs: |result: JsonString|,
                handler: handle_remove_modified_entry_ok
            }

            send_tweet: {
                inputs: |author: String, content: String|,
                outputs: |response: TweetResponse|,
                handler: handle_send_tweet
            }

            send_message: {
                inputs: |to_agent: Address, message: String|,
                outputs: |response: ZomeApiResult<String>|,
                handler: handle_send_message
            }
        }
    }
}
