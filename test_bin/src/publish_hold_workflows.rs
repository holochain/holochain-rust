use basic_workflows::setup_normal;
use constants::*;
use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    json_protocol::{
        ConnectData, DhtMetaData, EntryData, EntryListData, FailureResultData, FetchEntryData,
        FetchEntryResultData, FetchMetaData, GetListData, JsonProtocol, MessageData, TrackDnaData,
    },
    net_connection::NetSend,
    NetResult,
};
use p2p_node::P2pNode;

use std::{thread, time};

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
pub fn empty_publish_data_list_test(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: empty_publish_data_list_test()");
    setup_normal(alex, billy, can_connect)?;

    // Look for the publish_list request received from network module and reply
    let request = alex
        .find_recv_msg(
            0,
            Box::new(one_is!(JsonProtocol::HandleGetPublishingEntryList(_))),
        )
        .expect("Did not receive a HandleGetPublishingDataList request");

    let get_list_data = unwrap_to!(request => JsonProtocol::HandleGetPublishingEntryList);

    alex.reply_get_publish_data_list(&get_list_data)?;

    // billy asks for unpublished data.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let fetch_data = FetchEntryData {
        request_id         : FETCH_ENTRY_1_ID.into(),
        dna_address        : DNA_ADDRESS.clone(),
        requester_agent_id : BILLY_AGENT_ID.into(),
        entry_address      : ENTRY_ADDRESS_1.clone(),
    };
    billy.send(JsonProtocol::FetchEntry(fetch_data.clone()).into())?;

    // Alex sends a failureResult back to the network
    alex.reply_fetch_data(&fetch_data)?;

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
pub fn publish_list_test(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: publish_list_test()");
    setup_normal(alex, billy, can_connect)?;
    println!("\n");

    // author an entry without publishing it
    alex.author_entry(&DNA_ADDRESS, &ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, false)?;

    // Look for the publish_list request received from network module and reply
    let request = alex
        .find_recv_msg(
            0,
            Box::new(one_is!(JsonProtocol::HandleGetPublishingEntryList(_))),
        )
        .expect("Did not receive a HandleGetPublishingDataList request");
    let get_list_data = unwrap_to!(request => JsonProtocol::HandleGetPublishingEntryList);
    alex.reply_get_publish_data_list(&get_list_data)?;

    println!("\n");
    println!("alex.reply_get_publish_data_list() DONE \n");

    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);

    println!("\n");
    println!("alex.wait_HandleFetchEntry_and_reply() DONE \n");

    // billy might receive HandleDhtStore
    let _ = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchEntry(_))), 2000);

    println!("\n");
    println!("billy.send(FetchEntry) ... \n");

    // billy asks for reported published data.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let fetch_entry = FetchEntryData {
        request_id         : FETCH_ENTRY_1_ID.into(),
        dna_address        : DNA_ADDRESS.clone(),
        requester_agent_id : BILLY_AGENT_ID.into(),
        entry_address      : ENTRY_ADDRESS_1.clone(),
    };
    billy.send(JsonProtocol::FetchEntry(fetch_entry).into())?;

    println!("\n");
    println!("billy.send DONE \n");

    let has_received = alex.wait_HandleFetchEntry_and_reply();
    println!("alex.wait_HandleFetchEntry_and_reply(): {}", has_received);
    if !has_received {
        let has_received = billy.wait_HandleFetchEntry_and_reply();
        println!("billy.wait_HandleFetchEntry_and_reply(): {}", has_received);
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
    alex.author_entry(&DNA_ADDRESS, &ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, true)?;
    // author a meta without publishing it
    alex.author_meta(
        &DNA_ADDRESS,
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
    let get_list_data = if let JsonProtocol::HandleGetPublishingMetaList(msg) = request {
        msg
    } else {
        unreachable!()
    };
    // reply with publish_meta_list
    alex.reply_get_publish_meta_list(&get_list_data)?;

    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    assert!(has_received);
    println!("\n");
    println!("alex.wait_HandleFetchMeta_and_reply() DONE \n");
    // billy might receive HandleDhtStore
    let _ = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))), 2000);

    //    // Alex should receive a HandleFetchDhtMeta request
    //    let maybe_request = alex.wait(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))));
    //    if maybe_request.is_some() {
    //        let request = maybe_request.unwrap();
    //        println!("    got request 1: {:?}", request);
    //        // extract data
    //        let fetch_metadata = unwrap_to!(request => JsonProtocol::HandleFetchMeta);
    //        // Respond with data
    //        alex.reply_fetch_meta(&fetch_metadata)?;
    //    }

    // billy asks for reported published data.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let fetch_metadata = FetchMetaData {
        request_id         : FETCH_ENTRY_1_ID.into(),
        dna_address        : DNA_ADDRESS.clone(),
        requester_agent_id : BILLY_AGENT_ID.into(),
        entry_address      : ENTRY_ADDRESS_1.clone(),
        attribute          : META_ATTRIBUTE.into(),
    };
    billy.send(JsonProtocol::FetchMeta(fetch_metadata).into())?;

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
pub fn hold_list_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: hold_list_test()");
    setup_normal(alex, billy, can_connect)?;

    // Have alex hold some data
    alex.hold_data(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1);

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
    alex.reply_get_holding_data_list(&get_list_data)?;

    // Might receive a HandleFetchEntry request from network module:
    // hackmode would want the data right away
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    if has_received {
        // billy might receive HandleDhtStore
        let _ = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchEntry(_))), 2000);
    }
    println!("\n");
    println!("alex.wait_HandleFetchEntry_and_reply() DONE \n");

    // Have billy request that data
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let fetch_data = FetchEntryData {
        request_id         : FETCH_ENTRY_1_ID.into(),
        dna_address        : DNA_ADDRESS.clone(),
        requester_agent_id : BILLY_AGENT_ID.into(),
        entry_address      : ENTRY_ADDRESS_1.clone(),
    };
    billy.send(JsonProtocol::FetchEntry(fetch_data).into())?;

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
    alex.reply_get_holding_meta_list(&get_list_data)?;

    // Might receive a HandleFetchMeta request from network module:
    // hackmode would want the data right away
    let has_received = alex.wait_HandleFetchMeta_and_reply();
    if has_received {
        // billy might receive HandleStoreMeta
        let _ = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))), 2000);
    }
    println!("\n");
    println!("alex.wait_HandleFetchMeta_and_reply() DONE \n");

    // Have billy request that metadata
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let fetch_meta = FetchMetaData {
        attribute          : META_ATTRIBUTE.into(),
        requester_agent_id : BILLY_AGENT_ID.into(),
        request_id         : FETCH_META_1_ID.into(),
        dna_address        : DNA_ADDRESS.clone(),
        entry_address      : ENTRY_ADDRESS_1.clone(),
    };
    billy.send(JsonProtocol::FetchMeta(fetch_meta).into())?;

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
