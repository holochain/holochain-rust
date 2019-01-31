use basic_workflows::setup_normal;
use constants::*;
use holochain_net_connection::{
    json_protocol::JsonProtocol,
    NetResult,
};
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

    // Look for the publish_list request received from network module and reply
    alex.reply_to_first_HandleGetPublishingEntryList();

    // billy asks for unpublished data.
    let fetch_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // Alex sends a failureResult back to the network
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
    println!("\n");

    // author an entry without publishing it
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, false)?;

    // Look for the publish_list request received from network module and reply
    let request = alex
        .find_recv_msg(
            0,
            Box::new(one_is!(JsonProtocol::HandleGetPublishingEntryList(_))),
        )
        .expect("Did not receive a HandleGetPublishingDataList request");
    let get_list_data = unwrap_to!(request => JsonProtocol::HandleGetPublishingEntryList);
    alex.reply_to_HandleGetPublishingEntryList(&get_list_data)?;

    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);

    // billy might receive HandleDhtStore
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

    // author an entry
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, true)?;
    // author a meta without publishing it
    alex.author_meta(
        &ENTRY_ADDRESS_1,
        META_ATTRIBUTE.into(),
        &META_CONTENT_1,
        false,
    )?;

    // Look for the publish_meta_list request received from network module
    let request = alex
        .find_recv_msg(
            0,
            Box::new(one_is!(JsonProtocol::HandleGetPublishingMetaList(_))),
        )
        .expect("Did not receive a HandleGetPublishingMetaList request");
    let get_list_data = unwrap_to!(request => JsonProtocol::HandleGetPublishingMetaList);
    // reply with publish_meta_list
    alex.reply_to_HandleGetPublishingMetaList(&get_list_data)?;

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
pub fn hold_entry_list_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: hold_list_test()");
    setup_normal(alex, billy, can_connect)?;

    // Have alex hold some data
    alex.hold_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1);

    // Alex: Look for the hold_list request received from network module and reply
    let request = alex
        .find_recv_msg(
            0,
            Box::new(one_is!(JsonProtocol::HandleGetHoldingEntryList(_))),
        )
        .expect("Did not receive a HandleGetHoldingDataList request");
    // extract request data
    let get_list_data = unwrap_to!(request => JsonProtocol::HandleGetHoldingEntryList);
    // reply
    alex.reply_to_HandleGetHoldingEntryList(&get_list_data)?;

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
    println!("got result: {:?}", result);

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
    let request = alex
        .find_recv_msg(
            0,
            Box::new(one_is!(JsonProtocol::HandleGetHoldingMetaList(_))),
        )
        .expect("Did not receive a HandleGetHoldingMetaList request");
    // extract request data
    let get_list_data = unwrap_to!(request => JsonProtocol::HandleGetHoldingMetaList);
    // reply
    alex.reply_to_HandleGetHoldingMetaList(&get_list_data)?;

    // Might receive a HandleFetchMeta request from network module:
    // hackmode would want the data right away
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if has_received {
        // billy might receive HandleStoreMeta
        let _ = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))), 2000);
    }

    // Have billy request that metadata
    billy.request_meta(ENTRY_ADDRESS_1.clone(), META_ATTRIBUTE.into());

    // Alex might receive HandleFetchDhtData request as this moment
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleFetchMeta_and_reply();
    }

    // Billy shoudl receive the data
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))))
        .unwrap();
    println!("got result: {:?}", result);

    // Done
    Ok(())
}


/// Reply with some data in hold_list
#[cfg_attr(tarpaulin, skip)]
pub fn double_hold_list_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: double_hold_list_test()");
    setup_normal(alex, billy, can_connect)?;

    // Have alex hold some data
    alex.hold_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1);

    // Alex: Look for the hold_list request received from network module and reply
    let request = alex
        .find_recv_msg(
            0,
            Box::new(one_is!(JsonProtocol::HandleGetHoldingEntryList(_))),
        )
        .expect("Did not receive a HandleGetHoldingDataList request");
    // extract request data
    let get_list_data = unwrap_to!(request => JsonProtocol::HandleGetHoldingEntryList);
    // reply
    alex.reply_to_HandleGetHoldingEntryList(&get_list_data)?;

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
    println!("got result: {:?}", result);

    // Done
    Ok(())
}


/// Reply with some data in hold_list
#[cfg_attr(tarpaulin, skip)]
pub fn publish_same_entry_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: publish_same_entry_test()");
    setup_normal(alex, billy, can_connect)?;

    // author an entry without publishing it
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, false)?;
    // notify network module of our data
    alex.reply_to_first_HandleGetPublishingEntryList();

    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);
    // billy might receive HandleStoreEntry
    let _ = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchEntry(_))), 2000);

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
    println!("got result: {:?}", result);

    // Done
    Ok(())
}