
use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    json_protocol::{
        ConnectData, DhtData, DhtMetaData, FetchDhtData, FetchDhtMetaData, JsonProtocol, MessageData,
        TrackDnaData, HandleListResultData, GetListData, FailureResultData, HandleDhtResultData,
    },
    net_connection::NetSend,
    NetResult,
};
use p2p_node::P2pNode;
use constants::*;
use basic_workflows::setup_normal;


use std::{thread, time};


/// Macro for transforming a type check into a predicate
macro_rules! one_is {
    ($p:pat) => {
        |d| {
            if let $p = d {
                return true;
            }
            return false;
        }
    };
}


/// Test the following workflow after normal setup:
/// sequenceDiagram
/// participant a as Peer A
/// participant net as P2P Network
/// net->>a: HandleFetchPublishedDataList
/// a->>net: HandleFetchPublishedDataListResult(list:['xyz_addr'])
/// net->>a: HandleFetchData(xyz_addr)
/// a->>net: HandleFetchDataResult(xyz)
#[cfg_attr(tarpaulin, skip)]
pub fn empty_publish_data_list_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: empty_publish_data_list_test()");
    setup_normal(alex, billy, can_connect)?;

    // Look for the publish_list request received from network module and reply
    let request = alex.find_recv_msg(0, Box::new(one_is!(JsonProtocol::HandleGetPublishingDataList(_))))
                      .expect("Did not receive a HandleGetPublishingDataList request");
    let get_list_data = if let JsonProtocol::HandleGetPublishingDataList(msg) = request { msg } else { unreachable!() };
    alex.reply_get_publish_data_list(&get_list_data)?;

    // billy asks for unpublished data.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let fetch_data = FetchDhtData {
        request_id         : FETCH_ENTRY_1_ID.into(),
        dna_address        : DNA_ADDRESS.clone(),
        requester_agent_id : BILLY_AGENT_ID.into(),
        data_address       : ENTRY_ADDRESS_1.clone(),
    };
    billy.send(JsonProtocol::FetchDhtData(fetch_data.clone()).into())?;

    // Alex sends a failureResult back to the network
    alex.reply_fetch_data(&fetch_data)?;

    // Billy should receive the failureResult back
    let result = billy.wait(Box::new(one_is!(JsonProtocol::FailureResult(_))));
    println!("got result: {:?}", result);

    // Done
    Ok(())
}

/// Reply some data in publish_list
#[cfg_attr(tarpaulin, skip)]
pub fn publish_data_list_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: publish_data_list_test()");
    setup_normal(alex, billy, can_connect)?;

    // author an entry without publishing it
    alex.author_data(
        &DNA_ADDRESS,
        &ENTRY_ADDRESS_1,
        &ENTRY_CONTENT_1,
        false,
    )?;

    // Look for the publish_list request received from network module and reply
    let request = alex.find_recv_msg(0, Box::new(one_is!(JsonProtocol::HandleGetPublishingDataList(_))))
                      .expect("Did not receive a HandleGetPublishingDataList request");
    let get_list_data = if let JsonProtocol::HandleGetPublishingDataList(msg) = request { msg } else { unreachable!() };
    alex.reply_get_publish_data_list(&get_list_data)?;

    // Should receive a HandleFetchDhtData request from network module
    let request = alex.wait(Box::new(one_is!(JsonProtocol::HandleFetchDhtData(_))));
    println!("    got request: {:?}", request);
    // extract msg data
    let fetch_data = if let JsonProtocol::HandleFetchDhtData(msg) = request { msg } else { unreachable!() };

    // Respond with entry data
    alex.reply_fetch_data(&fetch_data)?;

    // billy asks for reported published data.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let fetch_data = FetchDhtData {
        request_id         : FETCH_ENTRY_1_ID.into(),
        dna_address        : DNA_ADDRESS.clone(),
        requester_agent_id : BILLY_AGENT_ID.into(),
        data_address       : ENTRY_ADDRESS_1.clone(),
    };
    billy.send(JsonProtocol::FetchDhtData(fetch_data).into())?;

    // Alex should receive HandleFetchDhtData request
    let request = alex.wait(Box::new(one_is!(JsonProtocol::HandleFetchDhtData(_))));
    println!("    got request 2: {:?}", request);
    // extract msg data
    let fetch_data = if let JsonProtocol::HandleFetchDhtData(msg) = request { msg } else { unreachable!() };

    // Alex responds: should send entry data back
    alex.reply_fetch_data(&fetch_data)?;

    // Billy should receive the entry data
    let result = billy.wait(Box::new(one_is!(JsonProtocol::FetchDhtDataResult(_))));
    println!("got result: {:?}", result);

    // Done
    Ok(())
}

/// Reply some data in publish_meta_list
#[cfg_attr(tarpaulin, skip)]
pub fn publish_meta_list_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: publish_meta_list_test()");
    setup_normal(alex, billy, can_connect)?;

    // author an entry
    alex.author_data(
        &DNA_ADDRESS,
        &ENTRY_ADDRESS_1,
        &ENTRY_CONTENT_1,
        true,
    )?;
    // author a meta without publishing it
    alex.author_meta(
        &DNA_ADDRESS,
        &ENTRY_ADDRESS_1,
        META_ATTRIBUTE.into(),
        &META_CONTENT_1,
        false,
    )?;

    // Look for the publish_meta_list request received from network module
    let request = alex.find_recv_msg(0, Box::new(one_is!(JsonProtocol::HandleGetPublishingMetaList(_))))
        .expect("Did not receive a HandleGetPublishingMetaList request");
    let get_list_data = if let JsonProtocol::HandleGetPublishingMetaList(msg) = request { msg } else { unreachable!() };
    // reply with publish_meta_list
    alex.reply_get_publish_meta_list(&get_list_data)?;

    // Alex should receive a HandleFetchDhtMeta request
    let request = alex.wait(Box::new(one_is!(JsonProtocol::HandleFetchDhtMeta(_))));
    println!("    got request 1: {:?}", request);
    // extract data
    let fetch_metadata = if let JsonProtocol::HandleFetchDhtMeta(msg) = request { msg } else { unreachable!() };

    // Respond with data
    alex.reply_fetch_meta(&fetch_metadata)?;

    // billy asks for reported published data.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let fetch_metadata = FetchDhtMetaData {
        request_id         : FETCH_ENTRY_1_ID.into(),
        dna_address        : DNA_ADDRESS.clone(),
        requester_agent_id : BILLY_AGENT_ID.into(),
        data_address       : ENTRY_ADDRESS_1.clone(),
        attribute          : META_ATTRIBUTE.into(),
    };
    billy.send(JsonProtocol::FetchDhtMeta(fetch_metadata).into())?;

    // Alex should receive HandleFetchDhtData request
    let request = alex.wait(Box::new(one_is!(JsonProtocol::HandleFetchDhtMeta(_))));
    println!("    got request 2: {:?}", request);
    // extract msg data
    let fetch_metadata = if let JsonProtocol::HandleFetchDhtMeta(msg) = request { msg } else { unreachable!() };

    // Alex responds: should send data back
    alex.reply_fetch_meta(&fetch_metadata)?;

    // Billy should receive the data
    let result = billy.wait(Box::new(one_is!(JsonProtocol::FetchDhtMetaResult(_))));
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
    let request = alex.find_recv_msg(0, Box::new(one_is!(JsonProtocol::HandleGetHoldingDataList(_))))
        .expect("Did not receive a HandleGetHoldingDataList request");
    // extract request data
    let get_list_data = if let JsonProtocol::HandleGetHoldingDataList(msg) = request { msg } else { unreachable!() };
    // reply
    alex.reply_get_holding_data_list(&get_list_data)?;

    // Have billy request that data
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let fetch_data = FetchDhtData {
        request_id         : FETCH_ENTRY_1_ID.into(),
        dna_address        : DNA_ADDRESS.clone(),
        requester_agent_id : BILLY_AGENT_ID.into(),
        data_address       : ENTRY_ADDRESS_1.clone(),
    };
    billy.send(JsonProtocol::FetchDhtData(fetch_data).into())?;

    // Alex should receive HandleFetchDhtData request
    let request = alex.wait(Box::new(one_is!(JsonProtocol::HandleFetchDhtData(_))));
    println!("    got request: {:?}", request);
    // extract msg data
    let fetch_data = if let JsonProtocol::HandleFetchDhtData(msg) = request { msg } else { unreachable!() };

    // Alex responds: should send data back
    alex.reply_fetch_data(&fetch_data)?;

    // Billy should receive the data
    let result = billy.wait(Box::new(one_is!(JsonProtocol::FetchDhtDataResult(_))));
    println!("got result: {:?}", result);

    // Done
    Ok(())
}

/// Reply with some meta in hold_meta_list
#[cfg_attr(tarpaulin, skip)]
pub fn hold_meta_list_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: hold_meta_list_test()");
    setup_normal(alex, billy, can_connect)?;

    // Have alex hold some data
    alex.hold_meta(&ENTRY_ADDRESS_1, META_ATTRIBUTE,&META_CONTENT_1);

    // Alex: Look for the hold_list request received from network module and reply
    let request = alex.find_recv_msg(0, Box::new(one_is!(JsonProtocol::HandleGetHoldingMetaList(_))))
                      .expect("Did not receive a HandleGetHoldingMetaList request");
    // extract request data
    let get_list_data = if let JsonProtocol::HandleGetHoldingMetaList(msg) = request { msg } else { unreachable!() };
    // reply
    alex.reply_get_holding_meta_list(&get_list_data)?;

    // Have billy request that metadata
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let fetch_meta = FetchDhtMetaData {
        attribute          : META_ATTRIBUTE.into(),
        requester_agent_id : BILLY_AGENT_ID.into(),
        request_id         : FETCH_META_1_ID.into(),
        dna_address        : DNA_ADDRESS.clone(),
        data_address       : ENTRY_ADDRESS_1.clone(),
    };
    billy.send(JsonProtocol::FetchDhtMeta(fetch_meta).into())?;

    // Alex should receive HandleFetchDhtData request
    let request = alex.wait(Box::new(one_is!(JsonProtocol::HandleFetchDhtMeta(_))));
    println!("    got request 1: {:?}", request);
    // extract msg data
    let fetch_meta = if let JsonProtocol::HandleFetchDhtMeta(msg) = request { msg } else { unreachable!() };

    // Alex responds: should send data back
    alex.reply_fetch_meta(&fetch_meta)?;

    // Billy shoudl receive the data
    let result = billy.wait(Box::new(one_is!(JsonProtocol::FetchDhtMetaResult(_))));
    println!("got result: {:?}", result);

    // Done
    Ok(())
}