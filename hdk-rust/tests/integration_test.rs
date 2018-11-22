extern crate holochain_container_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate tempfile;
extern crate test_utils;
#[macro_use]
extern crate serde_json;

use holochain_container_api::*;
use holochain_core_types::{
    cas::content::Address,
    dna::zome::{
        capabilities::{Capability, FnDeclaration, Membrane},
        entry_types::EntryTypeDef,
    },
    error::ZomeApiInternalResult,
    hash::HashString,
    json::JsonString,
};
use std::sync::{Arc, Mutex};
use test_utils::*;

pub fn create_test_cap_with_fn_names(fn_names: Vec<&str>) -> Capability {
    let mut capability = Capability::new();
    capability.cap_type.membrane = Membrane::Public;

    for fn_name in fn_names {
        let mut fn_decl = FnDeclaration::new();
        fn_decl.name = String::from(fn_name);
        capability.functions.push(fn_decl);
    }
    capability
}

fn start_holochain_instance() -> (Holochain, Arc<Mutex<TestLogger>>) {
    // Setup the holochain instance
    let wasm =
        create_wasm_from_file("wasm-test/target/wasm32-unknown-unknown/release/test_globals.wasm");
    let capabability = create_test_cap_with_fn_names(vec![
        "check_global",
        "check_commit_entry",
        "check_commit_entry_macro",
        "check_get_entry_result",
        "check_get_entry",
        "send_tweet",
        "commit_validation_package_tester",
        "link_two_entries",
        "links_roundtrip",
        "check_query",
        "check_app_entry_address",
        "check_sys_entry_address",
        "check_call",
        "check_call_with_args",
        "update_entry_ok",
        "remove_entry_ok",
        "remove_modified_entry_ok",
    ]);
    let mut dna = create_test_dna_with_cap("test_zome", "test_cap", &capabability, &wasm);

    dna.zomes.get_mut("test_zome").unwrap().entry_types.insert(
        String::from("validation_package_tester"),
        EntryTypeDef::new(),
    );

    let (context, test_logger) = test_context_and_logger("alex");
    let mut hc =
        Holochain::new(dna.clone(), context).expect("could not create new Holochain instance.");

    // Run the holochain instance
    hc.start().expect("couldn't start");
    (hc, test_logger)
}

#[test]
fn can_use_globals() {
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the debug API function for printing all GLOBALS
    let result = hc.call("test_zome", "test_cap", "check_global", r#"{}"#);
    assert_eq!(
        result.clone(),
        Ok(JsonString::from(HashString::from(
            "QmU92yJa32rGJYcgDwhxAeBtpHeK7wjLEqZ1bWnDZKTRB8"
        ))),
        "result = {:?}",
        result
    );
}

#[test]
fn can_commit_entry() {
    let (mut hc, _) = start_holochain_instance();

    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry",
        r#"{ "entry_type": "testEntryType", "value": "{\"stuff\": \"non fail\"}" }"#,
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(Address::from(
            "Qmf7HGMHTZSb4zPB2wvrJnkgmURJ9VuTnEi4xG6QguB36v"
        )),
    );
}

#[test]
fn can_commit_entry_macro() {
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        // this works because the macro names the args the same as the SerializedEntry fields
        r#"{ "entry_type": "testEntryType", "value": "{\"stuff\": \"non fail\"}" }"#,
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(Address::from(
            "Qmf7HGMHTZSb4zPB2wvrJnkgmURJ9VuTnEi4xG6QguB36v"
        )),
    );
}

#[test]
fn can_round_trip() {
    let (mut hc, test_logger) = start_holochain_instance();
    let result = hc.call(
        "test_zome",
        "test_cap",
        "send_tweet",
        r#"{ "author": "bob", "content": "had a boring day" }"#,
    );
    assert_eq!(
        result.unwrap(),
        JsonString::from("{\"first\":\"bob\",\"second\":\"had a boring day\"}"),
    );

    let test_logger = test_logger.lock().unwrap();

    println!("{:?}", *test_logger);
}

#[test]
fn can_get_entry() {
    println!("\n can_get_entry\n");
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        r#"{ "entry_type": "testEntryType", "value": "{\"stuff\": \"non fail\"}" }"#,
    );
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(Address::from(
            "Qmf7HGMHTZSb4zPB2wvrJnkgmURJ9VuTnEi4xG6QguB36v"
        )),
    );

    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_get_entry_result",
        &String::from(JsonString::from(json!(
                    {"entry_address": Address::from("Qmf7HGMHTZSb4zPB2wvrJnkgmURJ9VuTnEi4xG6QguB36v")}
                ))),
    );
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(
            "{\"addresses\":[\"Qmf7HGMHTZSb4zPB2wvrJnkgmURJ9VuTnEi4xG6QguB36v\"],\"entries\":[{\"value\":\"{\\\"stuff\\\": \\\"non fail\\\"}\",\"entry_type\":\"testEntryType\"}],\"crud_status\":[{\"bits\":1}],\"crud_links\":{}}"
        )
    );

    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_get_entry",
        &String::from(JsonString::from(json!(
                    {"entry_address": Address::from("Qmf7HGMHTZSb4zPB2wvrJnkgmURJ9VuTnEi4xG6QguB36v")}
                ))),
    );
    println!("\t can_get_entry result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(
            "{\"value\":\"{\\\"stuff\\\": \\\"non fail\\\"}\",\"entry_type\":\"testEntryType\"}"
        )
    );

    // test the case with a bad address
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_get_entry_result",
        &String::from(JsonString::from(json!(
                    {"entry_address": Address::from("QmbC71ggSaEa1oVPTeNN7ZoB93DYhxowhKSF6Yia2Vjxxx")}
                ))),
    );
    println!("\t can_get_entry_result result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(),
               JsonString::from("{\"addresses\":[],\"entries\":[],\"crud_status\":[],\"crud_links\":{}}"));

    // test the case with a bad address
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_get_entry",
        &String::from(JsonString::from(json!(
                    {"entry_address": Address::from("QmbC71ggSaEa1oVPTeNN7ZoB93DYhxowhKSF6Yia2Vjxxx")}
                ))),
    );
    println!("\t can_get_entry result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::null());
}

#[test]
#[cfg(not(windows))] // TODO does not work on windows because of different seperator
fn can_invalidate_invalid_commit() {
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        &String::from(JsonString::from(SerializedEntry::from(Entry::new(
            test_entry_type(),
            JsonString::from("{\"stuff\":\"FAIL\"}"),
        )))),
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from("{\"error\":{\"Internal\":\"{\\\"kind\\\":{\\\"ValidationFailed\\\":\\\"FAIL content is not allowed\\\"},\\\"file\\\":\\\"core/src/nucleus/ribosome/runtime.rs\\\",\\\"line\\\":\\\"84\\\"}\"}}"),
    );
}

#[test]
fn has_populated_validation_data() {
    let (mut hc, _) = start_holochain_instance();

    //
    // Add two entries to chain to have something to check ValidationData on
    //
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        r#"{ "entry_type": "testEntryType", "value": "{\"stuff\":\"non fail\"}" }"#,
    );
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(Address::from(
            "QmSxw5mUkFfc2W95GK2xaNYRp4a8ZXxY8o7mPMDJv9pvJg"
        )),
    );
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        r#"{ "entry_type": "testEntryType", "value": "{\"stuff\":\"non fail\"}" }"#,
    );
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(Address::from(
            "QmSxw5mUkFfc2W95GK2xaNYRp4a8ZXxY8o7mPMDJv9pvJg"
        )),
    );

    //
    // Expect the commit in this zome function to fail with a serialized ValidationData struct
    //
    let result = hc.call(
        "test_zome",
        "test_cap",
        "commit_validation_package_tester",
        r#"{}"#,
    );

    assert!(result.is_ok(), "\t result = {:?}", result);

    //
    // Deactivating this test for now since ordering of contents change non-deterministically
    //
    /*
    assert_eq!(
        JsonString::from("{\"Err\":{\"Internal\":\"{\\\"package\\\":{\\\"chain_header\\\":{\\\"entry_type\\\":{\\\"App\\\":\\\"validation_package_tester\\\"},\\\"entry_address\\\":\\\"QmYQPp1fExXdKfmcmYTbkw88HnCr3DzMSFUZ4ncEd9iGBY\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmSQqKHPpYZbafF7PXPKx31UwAbNAmPVuSHHxcBoDcYsci\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"},\\\"source_chain_entries\\\":[{\\\"value\\\":\\\"\\\\\\\"non fail\\\\\\\"\\\",\\\"entry_type\\\":\\\"testEntryType\\\"},{\\\"value\\\":\\\"\\\\\\\"non fail\\\\\\\"\\\",\\\"entry_type\\\":\\\"testEntryType\\\"},{\\\"value\\\":\\\"alex\\\",\\\"entry_type\\\":\\\"%agent_id\\\"}],\\\"source_chain_headers\\\":[{\\\"entry_type\\\":{\\\"App\\\":\\\"testEntryType\\\"},\\\"entry_address\\\":\\\"QmXxdzM9uHiSfV1xDwUxMm5jX4rVU8jhtWVaeCzjkFW249\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmRHUwiUuFJiMyRmKaA1U49fXEnT8qbZMoj2V9maa4Q3JE\\\",\\\"link_same_type\\\":\\\"QmRHUwiUuFJiMyRmKaA1U49fXEnT8qbZMoj2V9maa4Q3JE\\\",\\\"timestamp\\\":\\\"\\\"},{\\\"entry_type\\\":{\\\"App\\\":\\\"testEntryType\\\"},\\\"entry_address\\\":\\\"QmXxdzM9uHiSfV1xDwUxMm5jX4rVU8jhtWVaeCzjkFW249\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmRYerwRRXYxmYoxq1LTZMVVRfjNMAeqmdELTNDxURtHEZ\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"},{\\\"entry_type\\\":\\\"AgentId\\\",\\\"entry_address\\\":\\\"QmQw3V41bAWkQA9kwpNfU3ZDNzr9YW4p9RV4QHhFD3BkqA\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmQJxUSfJe2QoxTyEwKQX9ypbkcNv3cw1vasGTx1CUpJFm\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"}],\\\"custom\\\":null},\\\"sources\\\":[\\\"<insert your agent key here>\\\"],\\\"lifecycle\\\":\\\"Chain\\\",\\\"action\\\":\\\"Commit\\\"}\"}}"),
        result.unwrap(),
    );
    */}

#[test]
fn can_link_entries() {
    let (mut hc, _) = start_holochain_instance();

    let result = hc.call("test_zome", "test_cap", "link_two_entries", r#"{}"#);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::from(r#"{"Ok":null}"#));
}

#[test]
fn can_roundtrip_links() {
    let (mut hc, _) = start_holochain_instance();
    let result = hc.call("test_zome", "test_cap", "links_roundtrip", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
    let result_string = result.unwrap();

    println!("can_roundtrip_links result_string: {:?}", result_string);
    let expected = JsonString::from("{\"Ok\":[\"QmNgyf5AVG6596qpx83uyPKHU3yehwHFFUNscJzvRfTpVx\",\"QmQbe8uWt8fjE9wRfqnh42Eqj22tHYH6aqfzL7orazQpu3\"]}");
    let ordering1: bool = result_string == expected;

    let expected = JsonString::from("{\"Ok\":[\"QmQbe8uWt8fjE9wRfqnh42Eqj22tHYH6aqfzL7orazQpu3\",\"QmNgyf5AVG6596qpx83uyPKHU3yehwHFFUNscJzvRfTpVx\"]}");
    let ordering2: bool = result_string == expected;

    assert!(ordering1 || ordering2, "result = {:?}", result_string);
}

#[test]
fn can_check_query() {
    let (mut hc, _) = start_holochain_instance();

    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_query",
        r#"{ "entry_type_name": "testEntryType", "limit": "0" }"#,
    );
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(vec![Address::from(
            "QmNgyf5AVG6596qpx83uyPKHU3yehwHFFUNscJzvRfTpVx",
        )]),
    );
}

#[test]
fn can_check_app_entry_address() {
    let (mut hc, _) = start_holochain_instance();

    let result = hc.call("test_zome", "test_cap", "check_app_entry_address", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(Address::from(
            "QmbagHKV6kU89Z4FzQGMHpCYMxpR8WPxnse6KMArQ2wPJa"
        )),
    );
}

#[test]
fn can_check_sys_entry_address() {
    let (mut hc, _) = start_holochain_instance();

    let _result = hc.call("test_zome", "test_cap", "check_sys_entry_address", r#"{}"#);
    // TODO
    //    assert!(result.is_ok(), "result = {:?}", result);
    //    assert_eq!(
    //        result.unwrap(),
    //        r#"{"result":"QmYmZyvDda3ygMhNnEjx8p9Q1TonHG9xhpn9drCptRT966"}"#,
    //    );
}

#[test]
fn can_check_call() {
    let (mut hc, _) = start_holochain_instance();

    let result = hc.call("test_zome", "test_cap", "check_call", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(ZomeApiInternalResult::success(Address::from(
            "QmbagHKV6kU89Z4FzQGMHpCYMxpR8WPxnse6KMArQ2wPJa"
        ))),
    );
}

#[test]
fn can_check_call_with_args() {
    let (mut hc, _) = start_holochain_instance();

    let result = hc.call("test_zome", "test_cap", "check_call_with_args", r#"{}"#);
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(ZomeApiInternalResult::success(Address::from(
            "QmSxw5mUkFfc2W95GK2xaNYRp4a8ZXxY8o7mPMDJv9pvJg"
        ))),
    );
}

#[test]
fn can_remove_entry() {
    let (mut hc, _) = start_holochain_instance();
    let result = hc.call("test_zome", "test_cap", "remove_entry_ok", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from("{\"addresses\":[\"QmSxw5mUkFfc2W95GK2xaNYRp4a8ZXxY8o7mPMDJv9pvJg\"],\"entries\":[{\"value\":\"{\\\"stuff\\\":\\\"non fail\\\"}\",\"entry_type\":\"testEntryType\"}],\"crud_status\":[{\"bits\":4}],\"crud_links\":{}}"),
    );
}

#[test]
fn can_update_entry() {
    let (mut hc, _) = start_holochain_instance();
    let result = hc.call("test_zome", "test_cap", "update_entry_ok", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
}

#[test]
fn can_remove_modified_entry() {
    let (mut hc, _) = start_holochain_instance();
    let result = hc.call("test_zome", "test_cap", "remove_modified_entry_ok", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
}
