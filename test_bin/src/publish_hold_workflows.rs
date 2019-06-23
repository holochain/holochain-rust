use basic_workflows::setup_two_nodes;
use constants::*;
use holochain_net::{
    connection::{json_protocol::JsonProtocol, NetResult},
    tweetlog::*,
};
use p2p_node::test_node::TestNode;

/// Test the following workflow after normal setup:
/// sequenceDiagram
/// participant a as Alex
/// participant net as P2P Network
/// participant b as Billy
/// a->>net: HandleFetchPublishedDataListResult(list:[])
/// b->>net: FetchDhtData(xyz_addr)
/// net->>a: HandleFetchData(xyz_addr)
/// a-->>net: FailureResult
/// net->>b: FailureResult
#[cfg_attr(tarpaulin, skip)]
pub fn empty_publish_entry_list_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: empty_publish_entry_list_test()");
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    // Alex replies an empty list to the initial HandleGetPublishingEntryList
    alex.reply_to_first_HandleGetPublishingEntryList();
    // Billy asks for unpublished data.
    let fetch_data = billy.request_entry(ENTRY_ADDRESS_1.clone());
    // Alex sends back a failureResult response to the network
    alex.reply_to_HandleFetchEntry(&fetch_data)?;
    // Billy should receive the failureResult back
    let result = billy
        .wait_json(Box::new(one_is!(JsonProtocol::FailureResult(_))))
        .unwrap();
    log_i!("got result: {:?}", result);
    // Done
    Ok(())
}

/// Return some data in publish_list request
#[cfg_attr(tarpaulin, skip)]
pub fn publish_entry_list_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: publish_entry_list_test()");
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    // author an entry without publishing it
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, false)?;
    // Reply to the publish_list request received from network module
    alex.reply_to_first_HandleGetPublishingEntryList();
    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);
    // billy might receive HandleStoreEntry
    let _ =
        billy.wait_json_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchEntry(_))), 2000);
    // billy asks for reported published data.
    billy.request_entry(ENTRY_ADDRESS_1.clone());
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleFetchEntry_and_reply();
    }
    // Billy should receive the entry data
    let mut result =
        billy.find_recv_json_msg(0, Box::new(one_is!(JsonProtocol::FetchEntryResult(_))));
    if result.is_none() {
        result = billy.wait_json(Box::new(one_is!(JsonProtocol::FetchEntryResult(_))))
    }
    let json = result.unwrap();
    log_i!("got result: {:?}", json);
    // Done
    Ok(())
}

/// Reply some data in publish_meta_list
#[cfg_attr(tarpaulin, skip)]
pub fn publish_meta_list_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: publish_meta_list_test()");
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    // Author meta and reply to HandleGetPublishingMetaList
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, true)?;
    alex.author_meta(
        &ENTRY_ADDRESS_1,
        META_LINK_ATTRIBUTE.into(),
        &META_LINK_CONTENT_1,
        false,
    )?;
    alex.reply_to_first_HandleGetPublishingMetaList();
    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    assert!(has_received);
    // billy might receive HandleFetchMeta
    let _ = billy.wait_json_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))), 2000);
    // billy asks for reported published data.
    billy.request_meta(ENTRY_ADDRESS_1.clone(), META_LINK_ATTRIBUTE.into());
    // Alex or billy should receive HandleFetchMeta request
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleFetchMeta_and_reply();
    }
    // Billy should receive the data
    let mut result =
        billy.find_recv_json_msg(0, Box::new(one_is!(JsonProtocol::FetchMetaResult(_))));
    if result.is_none() {
        result = billy.wait_json(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))));
    }
    let json = result.unwrap();
    log_i!("got result: {:?}", json);
    // Done
    Ok(())
}

/// Reply with some meta in hold_meta_list
#[cfg_attr(tarpaulin, skip)]
pub fn hold_meta_list_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: hold_meta_list_test()");
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    // Have alex hold some data
    alex.hold_meta(&ENTRY_ADDRESS_1, META_LINK_ATTRIBUTE, &META_LINK_CONTENT_1);
    // Alex: Look for the hold_list request received from network module and reply
    alex.reply_to_first_HandleGetHoldingMetaList();
    // Might receive a HandleFetchMeta request from network module:
    // hackmode would want the data right away
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if has_received {
        // billy might receive HandleStoreMeta
        let _ =
            billy.wait_json_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))), 2000);
    }
    // Have billy request that metadata
    billy.request_meta(ENTRY_ADDRESS_1.clone(), META_LINK_ATTRIBUTE.into());
    // Alex might receive HandleFetchMeta request as this moment
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleFetchMeta_and_reply();
    }
    // Billy should receive the data
    let mut result =
        billy.find_recv_json_msg(0, Box::new(one_is!(JsonProtocol::FetchMetaResult(_))));
    if result.is_none() {
        result = billy.wait_json(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))));
    }
    let json = result.unwrap();
    log_i!("got result: {:?}", json);
    // Done
    Ok(())
}

/// Return some data in publish_list request
#[cfg_attr(tarpaulin, skip)]
pub fn double_publish_entry_list_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    println!("Testing: double_publish_entry_list_test()");
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, true)?;
    alex.reply_to_first_HandleGetPublishingEntryList();
    // Should NOT receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(!has_received);
    // billy asks for reported published data.
    billy.request_entry(ENTRY_ADDRESS_1.clone());
    // Alex or Billy receives and replies to a HandleFetchEntry
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleFetchEntry_and_reply();
    }
    // Billy should receive the entry data back
    let mut result =
        billy.find_recv_json_msg(0, Box::new(one_is!(JsonProtocol::FetchEntryResult(_))));
    if result.is_none() {
        result = billy.wait_json(Box::new(one_is!(JsonProtocol::FetchEntryResult(_))))
    }
    let json = result.unwrap();
    log_i!("got result: {:?}", json);
    // Done
    Ok(())
}

/// Reply some data in publish_meta_list
#[cfg_attr(tarpaulin, skip)]
pub fn double_publish_meta_list_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: double_publish_meta_list_test()");
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;

    // Author meta and reply to HandleGetPublishingMetaList
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, true)?;
    alex.author_meta(
        &ENTRY_ADDRESS_1,
        META_LINK_ATTRIBUTE.into(),
        &META_LINK_CONTENT_1,
        true,
    )?;
    alex.reply_to_first_HandleGetPublishingMetaList();
    // Should NOT receive a HandleFetchMeta request from network module
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    assert!(!has_received);
    // billy might receive HandleFetchMeta
    let _ = billy.wait_json_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))), 2000);
    // billy asks for reported published data.
    billy.request_meta(ENTRY_ADDRESS_1.clone(), META_LINK_ATTRIBUTE.into());
    // Alex or billy should receive HandleFetchMeta request
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleFetchMeta_and_reply();
    }
    // Billy should receive the data
    let mut result =
        billy.find_recv_json_msg(0, Box::new(one_is!(JsonProtocol::FetchMetaResult(_))));
    if result.is_none() {
        result = billy.wait_json(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))));
    }
    let json = result.unwrap();
    log_i!("got result: {:?}", json);
    // Done
    Ok(())
}

/// Reply some data in publish_meta_list
#[cfg_attr(tarpaulin, skip)]
pub fn many_meta_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: many_meta_test()");
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    // Author meta and reply to HandleGetPublishingMetaList
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, true)?;
    log_d!("entry authored");

    alex.author_meta(
        &ENTRY_ADDRESS_1,
        META_LINK_ATTRIBUTE.into(),
        &META_LINK_CONTENT_1,
        true,
    )?;
    log_d!("META_LINK_CONTENT_1 authored");
    alex.author_meta(
        &ENTRY_ADDRESS_1,
        META_CRUD_ATTRIBUTE.into(),
        &META_CRUD_CONTENT,
        true,
    )?;
    log_d!("META_CRUD_CONTENT authored");
    alex.author_meta(
        &ENTRY_ADDRESS_1,
        META_LINK_ATTRIBUTE.into(),
        &META_LINK_CONTENT_2,
        false,
    )?;
    log_d!("META_LINK_CONTENT_2 authored");
    alex.author_meta(
        &ENTRY_ADDRESS_1,
        META_LINK_ATTRIBUTE.into(),
        &META_LINK_CONTENT_3,
        false,
    )?;
    log_d!("META_LINK_CONTENT_3 authored");
    alex.reply_to_first_HandleGetPublishingMetaList();

    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    assert!(has_received);

    // billy might receive HandleFetchMeta
    let _ = billy.wait_json_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))), 2000);
    log_d!("alex has_received done");

    // billy asks for reported published data.
    let request_meta_1 = billy.request_meta(ENTRY_ADDRESS_1.clone(), META_LINK_ATTRIBUTE.into());

    // Alex or billy should receive HandleFetchMeta request
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if !has_received {
        billy.wait_HandleFetchMeta_and_reply();
    }
    log_d!("node has_received HandleFetchMeta 1 = {}", has_received);

    // Alex or billy should receive HandleFetchMeta request
    let mut has_received = alex.wait_HandleFetchMeta_and_reply();
    if !has_received {
        has_received = billy.wait_HandleFetchMeta_and_reply();
    }
    log_d!("node has_received HandleFetchMeta 2 = {}", has_received);

    // Billy should receive the data
    let mut result =
        billy.find_recv_json_msg(0, Box::new(one_is!(JsonProtocol::FetchMetaResult(_))));
    if result.is_none() {
        result = billy.wait_json(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))));
    }
    let result = result.unwrap();
    log_i!("got result 1: {:?}", result);
    let meta_data = unwrap_to!(result => JsonProtocol::FetchMetaResult);
    assert_eq!(meta_data.request_id, request_meta_1.request_id);
    assert_eq!(meta_data.entry_address, ENTRY_ADDRESS_1.clone());
    assert_eq!(meta_data.attribute, META_LINK_ATTRIBUTE.clone());
    assert_eq!(meta_data.content_list.len(), 3);

    // billy asks for reported published data.
    let request_meta_2 = billy.request_meta(ENTRY_ADDRESS_1.clone(), META_CRUD_ATTRIBUTE.into());
    // Alex or billy should receive HandleFetchMeta request
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleFetchMeta_and_reply();
    }
    // Billy should receive the data
    let mut result =
        billy.find_recv_json_msg(1, Box::new(one_is!(JsonProtocol::FetchMetaResult(_))));
    if result.is_none() {
        result = billy.wait_json(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))));
    }
    let json = result.unwrap();
    log_i!("got result 2: {:?}", json);
    let meta_data = unwrap_to!(json => JsonProtocol::FetchMetaResult);
    assert_eq!(meta_data.request_id, request_meta_2.request_id);
    assert_eq!(meta_data.entry_address, ENTRY_ADDRESS_1.clone());
    assert_eq!(meta_data.attribute, META_CRUD_ATTRIBUTE.clone());
    assert_eq!(meta_data.content_list.len(), 1);
    assert_eq!(meta_data.content_list[0], META_CRUD_CONTENT.clone());
    // Done
    Ok(())
}
