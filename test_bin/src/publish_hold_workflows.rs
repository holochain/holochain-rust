
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

// CONSTS
static PUBLISH_LIST_ID: &'static str = "publish_list_id_1";


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

    // No data edge case
    alex.send(
        JsonProtocol::HandleGetPublishingDataListResult(HandleListResultData {
            request_id: PUBLISH_LIST_ID.to_string(),
            dna_address: example_dna_address(),
            data_address_list: Vec::new(),
        })
            .into(),
    )?;

//    // Mock a HandleFetchDhtData
//    {
//        let server_msg = JsonProtocol::HandleFetchDhtData(FetchDhtData {
//            request_id: "get_dht_data_1".to_string(),
//            dna_address: example_dna_address(),
//            pub requester_agent_id: String,
//        })
//            .into();
//
//        let server = get_server(alex);
//        server.mock_send_one(
//            example_dna_address(),
//            ALEX_AGENT_ID,
//        data: Protocol,
//        ).expect("mock_send_one");
//
//    }

    // billy asks for unpublished data.
    // Shoud get a failure response
    billy.send(
        JsonProtocol::FetchDhtData(FetchDhtData {
            request_id: FETCH_ENTRY_1_ID.into(),
            dna_address: example_dna_address(),
            requester_agent_id: BILLY_AGENT_ID.into(),
            data_address: ENTRY_ADDRESS_1.into(),
        })
            .into(),
    )?;

    // Alex sends that data back to the network
    alex.send(
        JsonProtocol::FailureResult(FailureResultData {
            request_id: FETCH_ENTRY_1_ID.into(),
            dna_address: example_dna_address(),
            to_agent_id: BILLY_AGENT_ID.into(),
            error_info: json!("does not have data"),
        })
            .into(),
    )?;

    let result = billy.wait(Box::new(one_is!(JsonProtocol::FailureResult(_))))?;
    println!("got result: {:?}", result);

    // Done
    Ok(())
}


use std::{thread, time};

// Some data case
#[cfg_attr(tarpaulin, skip)]
pub fn publish_data_list_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: publish_data_list_test()");
    setup_normal(alex, billy, can_connect)?;

    // Respond to publish_list request
    alex.send(
        JsonProtocol::HandleGetPublishingDataListResult(HandleListResultData {
            request_id: "req_1".to_string(), // FIXME magic string should correspond to received HandleGetPublishingDataList request
            dna_address: example_dna_address(),
            data_address_list: vec![ENTRY_ADDRESS_1.to_string().into()],
        })
            .into(),
    )?;

    // Should receive HandleFetchDhtData request
    let request = alex.wait(Box::new(one_is!(JsonProtocol::HandleFetchDhtData(_))))?;
    println!("    got request: {:?}", request);

    let fetch_data = if let JsonProtocol::HandleFetchDhtData(msg) = request { msg } else { unreachable!() };

    // Respond with data
    alex.send(
        JsonProtocol::HandleFetchDhtDataResult(HandleDhtResultData {
            request_id: fetch_data.request_id.clone(),
            requester_agent_id: fetch_data.requester_agent_id.clone(),
            dna_address: fetch_data.dna_address.clone(),
            provider_agent_id: ALEX_AGENT_ID.to_string(),
            data_address: fetch_data.data_address.clone(),
            data_content: json!("hello"),
        })
            .into(),
    )?;

    // thread::sleep(time::Duration::from_secs(3));

    // billy asks for reported published data.
    billy.send(
        JsonProtocol::FetchDhtData(FetchDhtData {
            request_id: FETCH_ENTRY_1_ID.into(),
            dna_address: example_dna_address(),
            requester_agent_id: BILLY_AGENT_ID.into(),
            data_address: ENTRY_ADDRESS_1.into(),
        })
            .into(),
    )?;

    // Should receive HandleFetchDhtData request
    let request = alex.wait(Box::new(one_is!(JsonProtocol::HandleFetchDhtData(_))))?;
    println!("    got request 2: {:?}", request);

    let fetch_data = if let JsonProtocol::HandleFetchDhtData(msg) = request { msg } else { unreachable!() };

    // Respond with data
    alex.send(
        JsonProtocol::HandleFetchDhtDataResult(HandleDhtResultData {
            request_id: fetch_data.request_id.clone(),
            requester_agent_id: fetch_data.requester_agent_id.clone(),
            dna_address: fetch_data.dna_address.clone(),
            provider_agent_id: ALEX_AGENT_ID.to_string(),
            data_address: fetch_data.data_address.clone(),
            data_content: json!("hello"),
        })
            .into(),
    )?;

    let result = billy.wait(Box::new(one_is!(JsonProtocol::FetchDhtDataResult(_))))?;
    println!("got result: {:?}", result);

    // Done
    Ok(())
}

// TODO: double holding list