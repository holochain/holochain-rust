extern crate holochain_conductor_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_json_api;
extern crate holochain_persistence_api;
extern crate tempfile;
extern crate test_utils;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate hdk;
extern crate holochain_wasm_utils;
#[macro_use]
extern crate holochain_json_derive;

use hdk::error::ZomeApiError;
use hdk::error::ZomeApiResult;


use holochain_core_types::{
    crud_status::CrudStatus,
    dna::{
        fn_declarations::{FnDeclaration, TraitFns},
        zome::{ZomeFnDeclarations, ZomeTraits},
    },
    entry::{
        entry_type::test_app_entry_type,
        Entry,
    },
    
    error::{HolochainError, RibosomeEncodedValue, RibosomeEncodingBits},
};


use holochain_persistence_api::{
    cas::content::{AddressableContent},
    hash::HashString,
};
#[cfg(not(windows))]
use holochain_core_types::{error::CoreError};


use holochain_wasm_utils::{
    api_serialization::{
        get_entry::{GetEntryResult, StatusRequestKind},
        get_links::{GetLinksResult, LinksResult},
    },
};

use test_utils::{start_holochain_instance,make_test_call,TestEntry,wait_for_zome_result};

//
// These empty function definitions below are needed for the windows linker
//
#[no_mangle]
pub fn hc_init_globals(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_commit_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_get_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_entry_address(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_query(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_update_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_remove_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_send(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_encrypt(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_property(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_debug(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_call(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_crypto(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_meta(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_sign_one_time(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_verify_signature(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_link_entries(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_get_links(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_get_links_count(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_start_bundle(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_close_bundle(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_sleep(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn zome_setup(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn __list_traits(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn __list_functions(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_remove_link(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_list(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_new_random(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_derive_seed(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_derive_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_sign(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_get_public_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_commit_capability_grant(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_commit_capability_claim(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_emit_signal(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}


#[test]
pub fn test_invalid_target_link()
{
    let (mut hc, _,_signal_receiver) = start_holochain_instance("test_invalid_target_link", "alice");
    let result = make_test_call(&mut hc, "link_tag_validation", r#"{"stuff1" : "first","stuff2":"second","tag":"muffins"}"#);
    
    let expected_result : ZomeApiResult<()> = serde_json::from_str::<ZomeApiResult<()>>(&result.clone().unwrap().to_string()).unwrap();
    assert_eq!(expected_result.unwrap_err(),ZomeApiError::Internal(r#"{"kind":{"ValidationFailed":"invalid tag"},"file":"core\\src\\nucleus\\ribosome\\runtime.rs","line":"225"}"#.to_string()));

}

#[test]
pub fn test_bad_links()
{
    let (mut hc, _,_signal_receiver) = start_holochain_instance("test_bad_links", "alice");
    let result = make_test_call(&mut hc, "create_and_link_tagged_entry_bad_link", r#"{"content" : "message","tag":"maiffins"}"#);

    let expected_result : ZomeApiResult<()> = serde_json::from_str::<ZomeApiResult<()>>(&result.clone().unwrap().to_string()).unwrap();
    assert_eq!(expected_result.unwrap_err(),ZomeApiError::Internal(r#"{"kind":{"ErrorGeneric":"Base for link not found"},"file":"core\\src\\nucleus\\ribosome\\runtime.rs","line":"225"}"#.to_string()));

}

#[test]
pub fn test_links_with_immediate_timeout()
{
    let (mut hc, _,_signal_receiver) = start_holochain_instance("test_links_with_immediate_timeout", "alice");
    make_test_call(&mut hc, "create_and_link_tagged_entry", r#"{"content": "message me","tag":"tag me"}"#);

    let result = make_test_call(&mut hc, "my_entries_immediate_timeout", r#"{}"#);
    let expected_result : ZomeApiResult<()> = serde_json::from_str::<ZomeApiResult<()>>(&result.clone().unwrap().to_string()).unwrap();
    assert_eq!(expected_result.unwrap_err(),ZomeApiError::Internal(r#"{"kind":"Timeout","file":"core\\src\\nucleus\\ribosome\\runtime.rs","line":"225"}"#.to_string()));
}

#[test]
pub fn test_links_with_load()
{
    let (mut hc, _,_signal_receiver) = start_holochain_instance("test_links_with_load", "alice");
    let result = make_test_call(&mut hc, "create_and_link_tagged_entry", r#"{"content": "message me","tag":"tag me"}"#);
    assert!(result.is_ok(), "result = {:?}", result);
   
    let _result = make_test_call(&mut hc, "my_entries_with_load", r#"{}"#);

    let expected_result  = wait_for_zome_result::<Vec<TestEntry>>(&mut hc,"my_entries_with_load",r#"{}"#,|cond|cond.len()==1,6);
    let expected_links = expected_result.expect("Could not get links for test");
    assert_eq!(expected_links[0].stuff,"message me".to_string());

    let result = make_test_call(&mut hc, "delete_link_tagged_entry", r#"{"content": "message me","tag":"tag me"}"#);
    assert!(result.is_ok(), "result = {:?}", result);

    //query for deleted links
    let expected_result = wait_for_zome_result::<GetLinksResult>(&mut hc,"get_my_entries_by_tag",r#"{"tag" : "tag me","status":"Deleted"}"#,|cond|cond.links().len()==1,6);
    let expected_links = expected_result.unwrap().clone();
    assert_eq!(expected_links.links().len(),1);
    
    //try get links and load with nothing, not sure of necessary more of a type system check
    let expected_result = wait_for_zome_result::<Vec<TestEntry>>(&mut hc,"my_entries_with_load",r#"{}"#,|cond|cond.len()==0,6);
    let expected_links = expected_result.unwrap().clone();

    assert_eq!(expected_links.len(),0);
   
}

#[test]
#[cfg(not(windows))]
fn can_validate_links() {
    let (mut hc, _,_) = start_holochain_instance("can_validate_links", "alice");
    let params_ok = r#"{"stuff1": "a", "stuff2": "aa"}"#;
    let result = make_test_call(&mut hc, "link_validation", params_ok);
    assert!(result.is_ok(), "result = {:?}", result);

    let params_not_ok = r#"{"stuff1": "aaa", "stuff2": "aa"}"#;
    let result = make_test_call(&mut hc, "link_validation", params_not_ok);
    assert!(result.is_ok(), "result = {:?}", result);
    // Yep, the zome call is ok but what we got back should be a ValidationFailed error,
    // wrapped in a CoreError, wrapped in a ZomeApiError, wrapped in a Result,
    // serialized to JSON :D
    let zome_result: Result<(), ZomeApiError> =
        serde_json::from_str(&result.unwrap().to_string()).unwrap();
    assert!(zome_result.is_err());
    if let ZomeApiError::Internal(error) = zome_result.err().unwrap() {
        let core_error: CoreError = serde_json::from_str(&error).unwrap();
        assert_eq!(
            core_error.kind,
            HolochainError::ValidationFailed("Target stuff is not longer".to_string()),
        );
    } else {
        assert!(false);
    }
}

#[test]
fn create_tag_and_retrieve()
{
    let (mut hc, _,_signal_receiver) = start_holochain_instance("create_tag_and_retrieve", "alice");
    let result = make_test_call(&mut hc, "create_and_link_tagged_entry", r#"{"content": "message me","tag":"tag me"}"#);
    assert!(result.is_ok(), "result = {:?}", result);

    let result = make_test_call(&mut hc, "create_and_link_tagged_entry", r#"{"content": "message me once","tag":"tag another me"}"#);
    assert!(result.is_ok(), "result = {:?}", result);
 
    let expected_result = wait_for_zome_result::<GetLinksResult>(&mut hc,"get_my_entries_by_tag",r#"{"tag" : "tag another me"}"#,|cond|cond.links().len()==1,6);
    let expected_links = expected_result.unwrap().clone();
    assert!(expected_links.links().iter().any(|s| s.tag=="tag another me"));
    assert!(expected_links.links().iter().any(|s|s.address ==HashString::from("QmeuyJUoXHnU9GJT2LxnnNMmjDbvq1GGsa99pjmo1gPo4Y")));

    let expected_result = wait_for_zome_result::<GetLinksResult>(&mut hc,"get_my_entries_by_tag",r#"{"tag" : "tag me"}"#,|cond|cond.links().len()==1,6);
    let expected_links = expected_result.unwrap().clone();
    assert!(expected_links.links().iter().any(|s| s.tag=="tag me"));
    assert!(expected_links.links().iter().any(|s| s.address ==HashString::from("QmPdCLGkzp9daTcwbKePno9SySameXGRqdM4TfTGkju6Mo")));
    
    let expected_result = wait_for_zome_result::<GetLinksResult>(&mut hc,"get_my_entries_by_tag",r#"{}"#,|cond|cond.links().len()==2,6);
    let expected_links = expected_result.unwrap().clone();
    assert!(expected_links.links().iter().any(|s| s.tag=="tag another me"));
    assert!(expected_links.links().iter().any(|s| s.tag=="tag me"));

}