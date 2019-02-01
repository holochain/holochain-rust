use basic_workflows::setup_normal;
use constants::*;
use holochain_net_connection::{json_protocol::JsonProtocol, NetResult};
use p2p_node::P2pNode;

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
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: empty_publish_entry_list_test()");
    setup_normal(alex, billy, can_connect)?;
    // Alex replies an empty list to the initial HandleGetPublishingEntryList
    alex.reply_to_first_HandleGetPublishingEntryList();
    // Billy asks for unpublished data.
    let fetch_data = billy.request_entry(ENTRY_ADDRESS_1.clone());
    // Alex sends back a failureResult response to the network
    alex.reply_to_HandleFetchEntry(&fetch_data)?;
    // Billy should receive the failureResult back
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FailureResult(_))))
        .unwrap();
    println!("got result: {:?}", result);
    // Done
    Ok(())
}

/// Return some data in publish_list request
#[cfg_attr(tarpaulin, skip)]
pub fn publish_entry_list_test(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: publish_entry_list_test()");
    setup_normal(alex, billy, can_connect)?;
    // author an entry without publishing it
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, false)?;
    // Reply to the publish_list request received from network module
    alex.reply_to_first_HandleGetPublishingEntryList();
    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);
    // billy might receive HandleStoreEntry
    let _ = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchEntry(_))), 2000);
    // billy asks for reported published data.
    billy.request_entry(ENTRY_ADDRESS_1.clone());
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    if !has_received {
        let has_received = billy.wait_HandleFetchEntry_and_reply();
        assert!(has_received);
    }
    // Billy should receive the entry data
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchEntryResult(_))))
        .unwrap();
    println!("got result: {:?}", result);
    // Done
    Ok(())
}

/// Reply some data in publish_meta_list
#[cfg_attr(tarpaulin, skip)]
pub fn publish_meta_list_test(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: publish_meta_list_test()");
    setup_normal(alex, billy, can_connect)?;
    // Author meta and reply to HandleGetPublishingMetaList
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, true)?;
    alex.author_meta(
        &ENTRY_ADDRESS_1,
        META_ATTRIBUTE.into(),
        &META_CONTENT_1,
        false,
    )?;
    alex.reply_to_first_HandleGetPublishingMetaList();
    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    assert!(has_received);
    // billy might receive HandleDhtStore
    let _ = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))), 2000);
    // billy asks for reported published data.
    billy.request_meta(ENTRY_ADDRESS_1.clone(), META_ATTRIBUTE.into());
    // Alex or billy should receive HandleFetchMeta request
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if !has_received {
        billy.wait_HandleFetchMeta_and_reply();
    }
    // Billy should receive the data
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))))
        .unwrap();
    println!("got result: {:?}", result);
    // Done
    Ok(())
}

/// Reply with some data in hold_list
#[cfg_attr(tarpaulin, skip)]
pub fn hold_entry_list_test(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: hold_entry_list_test()");
    setup_normal(alex, billy, can_connect)?;
    // Have alex hold some data
    alex.hold_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1);
    // Alex: Look for the hold_list request received from network module and reply
    alex.reply_to_first_HandleGetHoldingEntryList();
    // Might receive a HandleFetchEntry request from network module:
    // hackmode would want the data right away
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    if has_received {
        // billy might receive HandleDhtStore
        let _ = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchEntry(_))), 2000);
    }
    // Have billy request that data
    billy.request_entry(ENTRY_ADDRESS_1.clone());
    // Alex or billy might receive HandleFetchEntry request as this moment
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleFetchEntry_and_reply();
    }
    // Billy should receive the data
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchEntryResult(_))))
        .unwrap();
    println!("\t got result: {:?}", result);
    // Done
    Ok(())
}

/// Reply with some meta in hold_meta_list
#[cfg_attr(tarpaulin, skip)]
pub fn hold_meta_list_test(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: hold_meta_list_test()");
    setup_normal(alex, billy, can_connect)?;
    // Have alex hold some data
    alex.hold_meta(&ENTRY_ADDRESS_1, META_ATTRIBUTE, &META_CONTENT_1);
    // Alex: Look for the hold_list request received from network module and reply
    alex.reply_to_first_HandleGetHoldingMetaList();
    // Might receive a HandleFetchMeta request from network module:
    // hackmode would want the data right away
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if has_received {
        // billy might receive HandleStoreMeta
        let _ = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))), 2000);
    }
    // Have billy request that metadata
    billy.request_meta(ENTRY_ADDRESS_1.clone(), META_ATTRIBUTE.into());
    // Alex might receive HandleFetchMeta request as this moment
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleFetchMeta_and_reply();
    }
    // Billy should receive the data
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))))
        .unwrap();
    println!("got result: {:?}", result);
    // Done
    Ok(())
}

/// Return some data in publish_list request
#[cfg_attr(tarpaulin, skip)]
pub fn double_publish_entry_list_test(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    println!("Testing: double_publish_entry_list_test()");
    setup_normal(alex, billy, can_connect)?;
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
        let has_received = billy.wait_HandleFetchEntry_and_reply();
        assert!(has_received);
    }
    // Billy should receive the entry data back
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchEntryResult(_))))
        .unwrap();
    println!("got result: {:?}", result);
    // Done
    Ok(())
}

/// Reply some data in publish_meta_list
#[cfg_attr(tarpaulin, skip)]
pub fn double_publish_meta_list_test(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: double_publish_meta_list_test()");
    setup_normal(alex, billy, can_connect)?;
    // Author meta and reply to HandleGetPublishingMetaList
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, true)?;
    alex.author_meta(
        &ENTRY_ADDRESS_1,
        META_ATTRIBUTE.into(),
        &META_CONTENT_1,
        true,
    )?;
    alex.reply_to_first_HandleGetPublishingMetaList();
    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    assert!(!has_received);
    // billy might receive HandleDhtStore
    let _ = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))), 2000);
    // billy asks for reported published data.
    billy.request_meta(ENTRY_ADDRESS_1.clone(), META_ATTRIBUTE.into());
    // Alex or billy should receive HandleFetchMeta request
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if !has_received {
        billy.wait_HandleFetchMeta_and_reply();
    }
    // Billy should receive the data
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))))
        .unwrap();
    println!("got result: {:?}", result);
    // Done
    Ok(())
}

//#[cfg_attr(tarpaulin, skip)]
//pub fn publish_same_entry_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
//    // Setup
//    println!("Testing: publish_same_entry_test()");
//    setup_normal(alex, billy, can_connect)?;
//
//    // author an entry without publishing it
//    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, true)?;
//    billy.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, true)?;
//
//    // Done
//    Ok(())
//}
