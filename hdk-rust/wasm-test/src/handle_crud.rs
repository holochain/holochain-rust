
use holochain_wasm_utils::{
    api_serialization::{
        get_entry::{GetEntryOptions, GetEntryResult, StatusRequestKind, GetEntryArgs},
    },
    holochain_core_types::{
        entry::{Entry, SerializedEntry},
        json::JsonString,
        crud_status::CrudStatus,
    },
};

use hdk_test_entry;
use hdk_test_entry_type;
use TestEntryType;


pub(crate) fn handle_update_entry_ok() -> JsonString {
    // Commit v1 entry
    hdk::debug("\n **** Commit v1 entry:").ok();
    let res = hdk::commit_entry(&hdk_test_entry());
    let addr_v1 = res.unwrap();

    // get it
    hdk::debug("\n **** Get it:").ok();
    let res = hdk::get_entry(addr_v1.clone());
    let entry_v1 = res.unwrap().unwrap();

    // update it to v2
    hdk::debug("\n **** update it to v2:").ok();
    let entry_v2 =
        Entry::new(hdk_test_entry_type(), TestEntryType { stuff: "v2".into() });
    let res = hdk::update_entry(entry_v2.clone(), addr_v1.clone());
    hdk::debug("\t res?").ok();
    let addr_v2 = res.unwrap();

    hdk::debug(entry_v2.to_string()).ok();
    JsonString::from(entry_v2.to_string())

//    // get latest from latest
//    let res = hdk::get_entry(addr_v2.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v2.clone());
//    // get latest from initial
//    let res = hdk::get_entry(addr_v1.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v2.clone());
//    // get initial from latest
//    let res = hdk::get_entry_initial(addr_v2.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v2.clone());
//    // get initial from initial
//    let res = hdk::get_entry_initial(addr_v1.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v1.clone());
//
//    // update it again from v1
//    let entry_v3 =
//        Entry::new(hdk_test_entry_type(), TestEntryType { stuff: "v3".into() });
//    let res = hdk::update_entry(entry_v3.clone(), addr_v1.clone());
//    let addr_v3 = res.unwrap();
//    // get latest from v1
//    let res = hdk::get_entry(addr_v1.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v3.clone());
//    // get latest from v2
//    let res = hdk::get_entry(addr_v2.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v3.clone());
//
//    // update it again from v3
//    let entry_v4 =
//        Entry::new(hdk_test_entry_type(), TestEntryType { stuff: "v4".into() });
//    let res = hdk::update_entry(entry_v4.clone(), addr_v3.clone());
//    let addr_v4 = res.unwrap();
//    // get latest from v1
//    let res = hdk::get_entry(addr_v1.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v4.clone());
//    // get latest from v2
//    let res = hdk::get_entry(addr_v2.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v4.clone());
//    // get latest from v3
//    let res = hdk::get_entry(addr_v3.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v4.clone());
//    // get latest from v4
//    let res = hdk::get_entry(addr_v4.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v4.clone());
//    // get initial from v1
//    let res = hdk::get_entry_initial(addr_v1.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v1.clone());
//    // get initial from v2
//    let res = hdk::get_entry_initial(addr_v2.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v2.clone());
//    // get initial from v3
//    let res = hdk::get_entry_initial(addr_v3.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v3.clone());
//    // get initial from v4
//    let res = hdk::get_entry_initial(addr_v4.clone());
//    let entry_res = res.unwrap().unwrap();
//    assert_eq!(entry_res, entry_v4.clone());
//
//    // get history from latest
//    let res = hdk::get_entry_history(addr_v4.clone());
//    let latest = res.unwrap().unwrap();
//    assert_eq!(latest.entries.len(), 1);
//    assert_eq!(latest.entries[0], SerializedEntry::from(entry_v4));
//    assert_eq!(latest.addresses[0], addr_v4.clone());
//    assert_eq!(latest.crud_status[0], CrudStatus::LIVE);
//    assert_eq!(latest.crud_links.len(), 0);
//
//    // get history from initial
//    let res = hdk::get_entry_history(addr_v1.clone());
//    let history = res.unwrap().unwrap();
//    assert_eq!(history.entries.len(), 4);
//    assert_eq!(history.entries[0], SerializedEntry::from(entry_v1.clone()));
//    assert_eq!(history.addresses[0], addr_v1.clone());
//    assert_eq!(history.crud_status[0], CrudStatus::MODIFIED);
//    assert_eq!(history.crud_links[&addr_v1.clone()], addr_v2.clone());
//
//    assert_eq!(history.entries[1], SerializedEntry::from(entry_v1.clone()));
//    assert_eq!(history.addresses[1], addr_v1.clone());
//    assert_eq!(history.crud_status[1], CrudStatus::MODIFIED);
//    assert_eq!(history.crud_links[&addr_v2.clone()], addr_v3.clone());
//
//    assert_eq!(history.entries[2], SerializedEntry::from(entry_v1.clone()));
//    assert_eq!(history.addresses[2], addr_v1.clone());
//    assert_eq!(history.crud_status[2], CrudStatus::MODIFIED);
//    assert_eq!(history.crud_links[&addr_v3.clone()], addr_v4.clone());
//
//    assert_eq!(history.entries[3], SerializedEntry::from(entry_v1.clone()));
//    assert_eq!(history.addresses[3], addr_v1.clone());
//    assert_eq!(history.crud_status[3], CrudStatus::LIVE);
//    assert_eq!(history.crud_links.get(&addr_v4.clone()), None);
//
//    JsonString::from(history)
}

//
pub fn handle_remove_entry_ok() -> JsonString {
    // Commit v1 entry
    let entry_test = hdk_test_entry();
    let res = hdk::commit_entry(&entry_test);
    let addr_v1 = res.unwrap();
    // get it
    hdk::debug("\n get it:\n");
    let res = hdk::get_entry(addr_v1.clone());
    let entry_v1 = res.unwrap().unwrap();
    assert_eq!(entry_test, entry_v1);
    // Delete it
    hdk::debug("\n Delete it:\n");
    let res = hdk::remove_entry(addr_v1.clone());
    assert!(res.is_ok());
    // get it should fail
    hdk::debug("\n get it should fail:\n");
    let res = hdk::get_entry(addr_v1.clone());
    assert_eq!(res.unwrap(), None);
    // Delete it again should fail
    hdk::debug("\n Delete it again should fail:\n");
    let res = hdk::remove_entry(addr_v1.clone());
    assert!(res.is_err());
    // get entry_result
    match hdk::get_entry_result(addr_v1, GetEntryOptions::default()) {
        Ok(result) => result.into(),
        Err(e) => e.into(),
    }
}


//
//pub fn handle_crud_err() -> JsonString {
//    // Commit an entry
//    // get it
//    // update it
//    // get initial
//    // get latest
//    // update it again
//    // get latest
//    // get all
//    // delete it
//    // get latest
//}