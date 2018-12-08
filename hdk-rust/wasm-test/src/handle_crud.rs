use holochain_wasm_utils::{
    api_serialization::{
        get_entry::GetEntryOptions,
    },
    holochain_core_types::{
        entry::Entry,
        json::JsonString,
        crud_status::CrudStatus,
    },
};
use hdk_test_entry;
use hdk_test_app_entry_type;
use TestEntryType;

//
pub(crate) fn handle_update_entry_ok() -> JsonString {
    // Commit v1 entry
    hdk::debug("**** Commit v1 entry").ok();
    let res = hdk::commit_entry(&hdk_test_entry());
    let addr_v1 = res.unwrap();
    // get it
    hdk::debug("**** Get it").ok();
    let res = hdk::get_entry(addr_v1.clone());
    let entry_v1 = res.unwrap().unwrap();

    // update it to v2
    hdk::debug("**** update it to v2").ok();
    let entry_v2 =
        Entry::App(hdk_test_app_entry_type(), JsonString::from(TestEntryType { stuff: "v2".into() }));
    let res = hdk::update_entry(entry_v2.clone(), addr_v1.clone());
    let addr_v2 = res.unwrap();
    // get latest from latest
    hdk::debug("**** get latest from latest").ok();
    let res = hdk::get_entry(addr_v2.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v2.clone());
    // get latest from initial
    hdk::debug("**** get latest from initial").ok();
    let res = hdk::get_entry(addr_v1.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v2.clone());
    // get initial from latest
    hdk::debug("**** get initial from latest").ok();
    let res = hdk::get_entry_initial(addr_v2.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v2.clone());
    // get initial from initial
    hdk::debug("**** get initial from initial").ok();
    let res = hdk::get_entry_initial(addr_v1.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v1.clone());

    // update it again from v1
    hdk::debug("**** update it again from v1").ok();
    let entry_v3 = Entry::App(
        hdk_test_app_entry_type(),
        JsonString::from(TestEntryType { stuff: "v3".into() }));
    let res = hdk::update_entry(entry_v3.clone(), addr_v1.clone());
    let addr_v3 = res.unwrap();
    // get latest from v1
    hdk::debug("**** get latest from v1").ok();
    let res = hdk::get_entry(addr_v1.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v3.clone());
    // get latest from v2
    hdk::debug("**** get latest from v2").ok();
    let res = hdk::get_entry(addr_v2.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v3.clone());

    // update it again from v3
    let entry_v4 = Entry::App(
        hdk_test_app_entry_type(),
        JsonString::from(TestEntryType { stuff: "v4".into() }),
    );
    let res = hdk::update_entry(entry_v4.clone(), addr_v3.clone());
    let addr_v4 = res.unwrap();
    // get latest from v1
    let res = hdk::get_entry(addr_v1.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v4.clone());
    // get latest from v2
    let res = hdk::get_entry(addr_v2.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v4.clone());
    // get latest from v3
    let res = hdk::get_entry(addr_v3.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v4.clone());
    // get latest from v4
    let res = hdk::get_entry(addr_v4.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v4.clone());
    // get initial from v1
    let res = hdk::get_entry_initial(addr_v1.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v1.clone());
    // get initial from v2
    let res = hdk::get_entry_initial(addr_v2.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v2.clone());
    // get initial from v3
    let res = hdk::get_entry_initial(addr_v3.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v3.clone());
    // get initial from v4
    hdk::debug("**** get initial from v4").ok();
    let res = hdk::get_entry_initial(addr_v4.clone());
    let entry_res = res.unwrap().unwrap();
    assert_eq!(entry_res, entry_v4.clone());

    // get history from latest
    hdk::debug("**** get history from latest").ok();
    let res = hdk::get_entry_history(addr_v4.clone());
    let latest = res.unwrap().unwrap();
    assert_eq!(latest.entries.len(), 1);
    assert_eq!(latest.entries[0], entry_v4.clone());
    assert_eq!(latest.addresses[0], addr_v4.clone());
    assert_eq!(latest.crud_status[0], CrudStatus::LIVE);
    assert_eq!(latest.crud_links.len(), 0);

    // get history from initial
    hdk::debug("**** get history from initial").ok();
    let res = hdk::get_entry_history(addr_v1.clone());
    let history = res.unwrap().unwrap();

    assert_eq!(history.entries.len(), 4);
    assert_eq!(history.entries[0], entry_v1.clone());
    assert_eq!(history.addresses[0], addr_v1.clone());
    assert_eq!(history.crud_status[0], CrudStatus::MODIFIED);
    assert_eq!(history.crud_links[&addr_v1.clone()], addr_v2.clone());

    assert_eq!(history.entries[1], entry_v2.clone());
    assert_eq!(history.addresses[1], addr_v2.clone());
    assert_eq!(history.crud_status[1], CrudStatus::MODIFIED);
    assert_eq!(history.crud_links[&addr_v2.clone()], addr_v3.clone());

    assert_eq!(history.entries[2], entry_v3.clone());
    assert_eq!(history.addresses[2], addr_v3.clone());
    assert_eq!(history.crud_status[2], CrudStatus::MODIFIED);
    assert_eq!(history.crud_links[&addr_v3.clone()], addr_v4.clone());

    assert_eq!(history.entries[3], entry_v4.clone());
    assert_eq!(history.addresses[3], addr_v4.clone());
    assert_eq!(history.crud_status[3], CrudStatus::LIVE);
    assert_eq!(history.crud_links.get(&addr_v4.clone()), None);

    JsonString::from(history)
}

//
pub fn handle_remove_entry_ok() -> JsonString {
    // Commit v1 entry
    hdk::debug("**** Commit v1 entry").ok();
    let entry_v1 = hdk_test_entry();
    let res = hdk::commit_entry(&entry_v1);
    let addr_v1 = res.unwrap();
    // Get it
    hdk::debug("**** Get it").ok();
    let res = hdk::get_entry(addr_v1.clone());
    let entry_test = res.unwrap().unwrap();
    assert_eq!(entry_test, entry_v1);
    // Delete it
    hdk::debug("**** Delete it").ok();
    let res = hdk::remove_entry(addr_v1.clone());
    assert!(res.is_ok());
    // Get it should fail
    hdk::debug("**** Get it should fail").ok();
    let res = hdk::get_entry(addr_v1.clone());
    assert_eq!(res.unwrap(), None);
    // Get initial should work
    hdk::debug("**** Get initial should work").ok();
    let res = hdk::get_entry_initial(addr_v1.clone());
    assert_eq!(res.unwrap(), Some(entry_v1));
    // Delete it again should fail
    hdk::debug("**** Delete it again should fail").ok();
    let res = hdk::remove_entry(addr_v1.clone());
    assert!(res.is_err());
    // Get entry_result
    match hdk::get_entry_result(addr_v1, GetEntryOptions::default()) {
        Ok(result) => result.into(),
        Err(e) => e.into(),
    }
}

//
pub fn handle_remove_modified_entry_ok() -> JsonString {
    // Commit entry v1
    hdk::debug("**** commit v1 entry").ok();
    let entry_v1 = hdk_test_entry();
    let res = hdk::commit_entry(&entry_v1);
    let addr_v1 = res.unwrap();
    // Get it
    hdk::debug("**** get it").ok();
    let res = hdk::get_entry(addr_v1.clone());
    let entry_test = res.unwrap().unwrap();
    assert_eq!(entry_test, entry_v1);
    // Update it to v2
    hdk::debug("**** update it to v2").ok();
    let entry_v2 = Entry::App(
        hdk_test_app_entry_type(),
        JsonString::from(TestEntryType { stuff: "v2".into() }),
    );
    let res = hdk::update_entry(entry_v2.clone(), addr_v1.clone());
    let addr_v2 = res.unwrap();
    // Get v2
    hdk::debug("**** get v2").ok();
    let res = hdk::get_entry(addr_v1.clone());
    let entry_test = res.unwrap().unwrap();
    assert_eq!(entry_test, entry_v2);
    // Delete it
    hdk::debug("**** delete it").ok();
    let res = hdk::remove_entry(addr_v1.clone());
    assert!(res.is_ok());
    // Get v2 should fail
    hdk::debug("**** get v2 should fail").ok();
    let res = hdk::get_entry(addr_v2.clone());
    assert_eq!(res.unwrap(), None);
    // Get v1 should fail
    hdk::debug("**** get v1 should fail").ok();
    let res = hdk::get_entry(addr_v1.clone());
    assert_eq!(res.unwrap(), None);
    // Get initial should work
    hdk::debug("**** get initial should work").ok();
    let res = hdk::get_entry_initial(addr_v1.clone());
    assert_eq!(res.unwrap(), Some(entry_v1.clone()));
    // Delete v2 again should fail
    hdk::debug("**** delete v2 again should fail").ok();
    let res = hdk::remove_entry(addr_v2.clone());
    assert!(res.is_err());
    // Delete v1 again should fail
    hdk::debug("**** delete v1 again should fail").ok();
    let res = hdk::remove_entry(addr_v1.clone());
    assert!(res.is_err());

    // Get history from initial
    hdk::debug("**** get history from initial").ok();
    let res = hdk::get_entry_history(addr_v1.clone());
    let history = res.unwrap().unwrap();

    assert_eq!(history.entries.len(), 2);
    assert_eq!(history.entries[0], entry_v1.clone());
    assert_eq!(history.addresses[0], addr_v1.clone());
    assert_eq!(history.crud_status[0], CrudStatus::MODIFIED);
    assert_eq!(history.crud_links[&addr_v1.clone()], addr_v2.clone());

    assert_eq!(history.entries[1], entry_v2.clone());
    assert_eq!(history.addresses[1], addr_v2.clone());
    assert_eq!(history.crud_status[1], CrudStatus::DELETED);
    assert!(history.crud_links.get(&addr_v2.clone()).is_some());

    JsonString::from(history)
}