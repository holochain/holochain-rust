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

use boolinator::Boolinator;
use hdk::{
    error::{ZomeApiError, ZomeApiResult},
};
use holochain_wasm_utils::{
    api_serialization::{
        get_entry::{GetEntryOptions, GetEntryResult},
        get_links::GetLinksResult,
        query::{ QueryArgsNames, QueryArgsOptions, QueryResult },
    },
    holochain_core_types::{
        cas::content::{Address, AddressableContent},
        dna::entry_types::Sharing,
        entry::{
            entry_type::{AppEntryType, EntryType},
            AppEntryValue, Entry,
        },
        error::{
            HolochainError,
            RibosomeErrorCode,
        },
        json::{JsonString,RawString},
    },
};
use holochain_wasm_utils::holochain_core_types::{validation::{LinkValidationData,EntryValidationData},error::RibosomeEncodingBits};
use holochain_wasm_utils::memory::ribosome::load_ribosome_encoded_json;
use holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result;
use holochain_wasm_utils::memory::allocation::WasmAllocation;
use hdk::global_fns::init_global_memory;
use holochain_wasm_utils::holochain_core_types::error::RibosomeEncodedValue;
use std::convert::TryFrom;
use std::time::Duration;
use hdk::api::G_MEM_STACK;

#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
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
pub extern "C" fn check_commit_entry(encoded_allocation_of_input: RibosomeEncodingBits) -> RibosomeEncodingBits {

    let allocation = match WasmAllocation::try_from_ribosome_encoding(encoded_allocation_of_input) {
        Ok(allocation) => allocation,
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    };

    let memory_init_result = init_global_memory(allocation);
    if memory_init_result.is_err() {
        return return_code_for_allocation_result(memory_init_result).into();
    }

    // Deserialize and check for an encoded error
    let entry: Entry = match load_ribosome_encoded_json(encoded_allocation_of_input) {
        Ok(entry) => entry,
        Err(hc_err) => {
            hdk::debug(format!("ERROR: {:?}", hc_err.to_string())).ok();
            return RibosomeEncodedValue::Failure(RibosomeErrorCode::ArgumentDeserializationFailed).into();
        },
    };

    hdk::debug(format!("Entry: {:?}", entry)).ok();

    let res = hdk::commit_entry(&entry.into());

    let res_obj: JsonString = match res {
        Ok(hash) => hash.into(),
        Err(e) => e.into(),
    };

    let mut wasm_stack = match unsafe { G_MEM_STACK } {
        Some(wasm_stack) => wasm_stack,
        None => return RibosomeEncodedValue::Failure(RibosomeErrorCode::OutOfMemory).into(),
    };

    return_code_for_allocation_result(
        wasm_stack.write_json(res_obj)
    ).into()

}

fn handle_check_commit_entry_macro(entry: Entry) -> ZomeApiResult<Address> {
    hdk::commit_entry(&entry)
}

fn handle_check_get_entry_result(entry_address: Address) -> ZomeApiResult<GetEntryResult> {
    hdk::get_entry_result(&entry_address, GetEntryOptions::default())
}

fn handle_check_get_entry(entry_address: Address) -> ZomeApiResult<Option<Entry>> {
    hdk::get_entry(&entry_address)
}

fn handle_commit_validation_package_tester() -> ZomeApiResult<Address> {
    hdk::commit_entry(&Entry::App(
        "validation_package_tester".into(),
        RawString::from("test").into(),
    ))
}

fn handle_link_two_entries() -> ZomeApiResult<Address> {
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

    hdk::link_entries(&entry_1.address(), &entry_2.address(), "test-tag", "test-type")
}

fn handle_remove_link() -> ZomeApiResult<()> {
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
    hdk::link_entries(&entry_1.address(), &entry_2.address(), "test-tag", "test-type")?;
    hdk::remove_link(&entry_1.address(), &entry_2.address(), "test-tag", "test-type")

}

/// Commit 3 entries
/// Commit a "test-tag" link from entry1 to entry2
/// Commit a "test-tag" link from entry1 to entry3
/// return entry1 address
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

    hdk::link_entries(&entry_1.address(), &entry_2.address(), "test-tag", "test-type")?;
    hdk::link_entries(&entry_1.address(), &entry_3.address(), "test-tag", "test-type")?;
    Ok(entry_1.address())
}

fn handle_links_roundtrip_get(address: Address) -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&address, "test-tag")
}

fn handle_links_roundtrip_get_and_load(
    address: Address,
) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
    hdk::get_links_and_load(&address, "test-tag")
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
    let addresses = hdk::query(
        QueryArgsNames::QueryList(vec![EntryType::AgentId.to_string()]),
        0,
        0,
    )
    .unwrap();

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
    let addresses =
        hdk::query(QueryArgsNames::QueryName("testEntryType".to_string()), 0, 1).unwrap();

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
    let addresses = hdk::query(vec!["[%]*", "testEntryType"].into(), 0, 0).unwrap();
    if !addresses.len() == 5 {
        return err("System + testEntryType Addresses not length 5");
    }

    // Confirm same results via hdk::query_result
    let addresses = match hdk::query_result(vec!["[%]*","testEntryType"].into(),
                                            QueryArgsOptions::default()).unwrap() {
        QueryResult::Addresses(av) => av,
        _ => return err("Unexpected hdk::query_result"),
    };
    if !addresses.len() == 5 {
        return err("System + testEntryType Addresses enum not length 5");
    };
    let headers = match hdk::query_result(vec!["[%]*","testEntryType"].into(),
                                          QueryArgsOptions{ headers: true,
                                                            ..Default::default()}).unwrap() {
        QueryResult::Headers(hv) => hv,
        _ => return err("Unexpected hdk::query_result"),
    };
    if !headers.len() == 5 {
        return err("System + testEntryType Headers enum not length 5");
    };

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
        Address::from(hdk::PUBLIC_TOKEN.to_string()),
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
        Address::from(hdk::PUBLIC_TOKEN.to_string()),
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

fn handle_send_message(to_agent: Address, message: String) -> ZomeApiResult<String> {
    hdk::send(to_agent, message, 60000.into())
}

fn handle_sleep() -> ZomeApiResult<()> {
    hdk::sleep(Duration::from_millis(10))
}

define_zome! {
    entries: [
        entry!(
            name: "testEntryType",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainFull
            },

            validation: |valida: hdk::EntryValidationData<TestEntryType>| {
                match valida
                {
                    EntryValidationData::Create{entry:test_entry,validation_data:_} =>
                    {
                        (test_entry.stuff != "FAIL").ok_or_else(|| "FAIL content is not allowed".to_string())

                    },
                    _=> Ok(()),

                }

            },

            links: [
                to!(
                    "testEntryType",
                    tag: "test-tag",
                    r#type: "test-type",
                    validation_package: || {
                        hdk::ValidationPackageDefinition::ChainFull
                    },
                    validation: |validation_data: hdk::LinkValidationData | {
                        Ok(())
                    }
                )
            ]
        ),

        entry!(
            name: "validation_package_tester",
            description: "asdfda",
            sharing: Sharing::Public,
            validation_package: || {
                hdk::ValidationPackageDefinition::ChainFull
            },

            validation: |validation_data: hdk::EntryValidationData<TestEntryType>| {
                match validation_data
                {
                    EntryValidationData::Create{entry:test_entry,validation_data:_} =>
                    {

                        Err(serde_json::to_string(&test_entry).unwrap())

                    },
                _ => Ok(())
                }
            }
        ),

        entry!(
            name: "link_validator",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },

            validation: |validation_data: hdk::EntryValidationData<TestEntryType>| {
                Ok(())
            },

            links: [
                to!(
                    "link_validator",
                    tag: "longer",
                    r#type: "longer",
                    validation_package: || {
                        hdk::ValidationPackageDefinition::Entry
                    },
                    validation: |validation_data: hdk::LinkValidationData | {
                        let link = match validation_data
                        {
                            LinkValidationData::LinkAdd{link,validation_data:_} => link.clone(),
                            LinkValidationData::LinkRemove{link,validation_data:_} => link.clone()
                        };
                        let base = link.link().base();
                        let target = link.link().target();
                        let base = match hdk::get_entry(&base)? {
                            Some(entry) => match entry {
                                Entry::App(_, test_entry) => TestEntryType::try_from(test_entry)?,
                                _ => Err("System entry found")?
                            },
                            None => Err("Base not found")?,
                        };

                        let target = match hdk::get_entry(&target)? {
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

    receive: |_from, payload| {
        {
            let entry = Entry::App(
                "testEntryType".into(),
                EntryStruct {
                    stuff: payload.clone(),
                }
                .into(),
            );
            match hdk::commit_entry(&entry) {
                Ok(address) => format!("Committed: '{}' / address: {}", payload, address.to_string()),
                Err(error) => format!("Error committing in receive: '{}'", error.to_string()),
            }
        }
    }

    functions: [
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
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_link_two_entries
        }

        remove_link: {
            inputs: | |,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_remove_link
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

        links_roundtrip_get_and_load: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<Vec<ZomeApiResult<Entry>>>|,
            handler: handle_links_roundtrip_get_and_load
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

        sleep: {
            inputs: | |,
            outputs: |response: ZomeApiResult<()>|,
            handler: handle_sleep
        }
    ]

    traits: {}
}
