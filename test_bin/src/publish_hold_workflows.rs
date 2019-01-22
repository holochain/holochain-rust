
use crate::*;

use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    json_protocol::{
        ConnectData, DhtData, DhtMetaData, FetchDhtData, FetchDhtMetaData, JsonProtocol, MessageData,
        TrackDnaData, HandleListResultData, GetListData,
    },
    net_connection::NetSend,
    NetResult,
};

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
static FETCH_ENTRY_1_ID: &'static str = "fetch_entry_1_id";

/// Test the following workflow after normal setup:
/// sequenceDiagram
/// participant a as Peer A
/// participant net as P2P Network
/// net->>a: HandleFetchPublishedDataList
/// a->>net: HandleFetchPublishedDataListResult(list:['xyz_addr'])
/// net->>a: HandleFetchData(xyz_addr)
/// a->>net: HandleFetchDataResult(xyz)
#[cfg_attr(tarpaulin, skip)]
fn empty_publish_data_list_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
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
            request_id: FETCH_ENTRY_1_ID.to_string(),
            dna_address: example_dna_address(),
            requester_agent_id: BILLY_AGENT_ID.to_string(),
            data_address: ENTRY_ADDRESS_1.to_string(),
        })
            .into(),
    )?;

    let result = billy.wait(Box::new(one_is!(JsonProtocol::FailureResult(_))))?;
    println!("got result: {:?}", result);

    // Done
    Ok(())
}



#[cfg_attr(tarpaulin, skip)]
fn publish_data_list_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: publish_data_list_test()");
    setup_normal(alex, billy, can_connect)?;

    // Some data case
    alex.send(
        JsonProtocol::HandleGetPublishingDataListResult(HandleListResultData {
            request_id: PUBLISH_LIST_ID.to_string(),
            dna_address: example_dna_address(),
            data_address_list: vec![ENTRY_ADDRESS_1.to_string().into(), ENTRY_ADDRESS_2.to_string().into()],
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
            request_id: FETCH_ENTRY_1_ID.to_string(),
            dna_address: example_dna_address(),
            requester_agent_id: BILLY_AGENT_ID.to_string(),
            data_address: ENTRY_ADDRESS_1.to_string(),
        })
            .into(),
    )?;

    let result = billy.wait(Box::new(one_is!(JsonProtocol::HandleFetchDhtDataResult(_))))?;
    println!("got result: {:?}", result);

    // Done
    Ok(())
}