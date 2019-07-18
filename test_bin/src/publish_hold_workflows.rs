use basic_workflows::setup_two_nodes;
use constants::*;
use holochain_net::{connection::NetResult, tweetlog::*};
use p2p_node::test_node::TestNode;

use lib3h_protocol::{data_types::EntryData, protocol_server::Lib3hServerProtocol};

///
#[cfg_attr(tarpaulin, skip)]
pub fn empty_publish_entry_list_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    // Alex replies an empty list to the initial HandleGetAuthoringEntryList
    alex.reply_to_first_HandleGetAuthoringEntryList();
    // Billy asks for unpublished data.
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // #fullsync
    // Alex sends back a failureResult response to the network
    let res = billy.reply_to_HandleQueryEntry(&query_data);
    assert!(res.is_err());
    // Billy should receive the failureResult back
    let result = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::FailureResult(_))))
        .unwrap();
    log_i!("got result: {:?}", result);
    let gen_res = unwrap_to!(result => Lib3hServerProtocol::FailureResult);
    assert_eq!(res.err().unwrap(), *gen_res);
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
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    // author an entry without publishing it
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_1.clone()], false)?;
    // Reply to the publish_list request received from network module
    alex.reply_to_first_HandleGetAuthoringEntryList();
    // Should receive a HandleFetchEntry request from network module after receiving list
    let _ = alex.wait_HandleFetchEntry_and_reply();
    //    // Should receive a HandleFetchEntry request from network module for gossip
    //    let _ = alex.wait_HandleFetchEntry_and_reply();

    // billy might receive HandleStoreEntryAspect
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleStoreEntryAspect(_))),
        2000,
    );

    println!("[1] Billy got res: {:?}", res);
    // billy asks for reported authored data.
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());
    let res = billy.reply_to_HandleQueryEntry(&query_data);
    println!("[2] Billy got res: {:?}", res);
    // #fullsync
    // Billy answers its own request
    // let has_received = billy.wait_HandleQueryEntry_and_reply();
    assert!(res.is_ok());
    // Billy should receive the entry data
    let mut result = billy.find_recv_lib3h_msg(
        0,
        Box::new(one_is!(Lib3hServerProtocol::QueryEntryResult(_))),
    );
    if result.is_none() {
        result = billy.wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::QueryEntryResult(_))))
    }
    let response = result.unwrap();
    log_i!("got result: {:?}", response);
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
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_1.clone()], true)?;
    alex.reply_to_first_HandleGetAuthoringEntryList();
    //    // Should receive only one HandleFetchEntry request from network module for Gossip
    //    let _ = alex.wait_HandleFetchEntry_and_reply();
    // billy might receive HandleStoreEntryAspect
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    log_i!("Billy got res: {:?}", res);
    // billy asks for reported published data.
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());
    billy.reply_to_HandleQueryEntry(&query_data).unwrap();
    // #fullsync
    // Billy receives and replies to its own query
    //let _ = billy.wait_HandleQueryEntry_and_reply();
    // Billy should receive the entry data back
    let mut result = billy.find_recv_lib3h_msg(
        0,
        Box::new(one_is!(Lib3hServerProtocol::QueryEntryResult(_))),
    );
    if result.is_none() {
        result = billy.wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::QueryEntryResult(_))))
    }
    let json = result.unwrap();
    log_i!("got result: {:?}", json);
    // Done
    Ok(())
}

/// Reply with some meta in hold_meta_list
#[cfg_attr(tarpaulin, skip)]
pub fn hold_list_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    // Have alex hold some data
    alex.hold_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_1.clone()])?;
    // Alex: Look for the hold_list request received from network module and reply
    alex.reply_to_first_HandleGetHoldingEntryList();
    // wait for gossip to ask for the held data
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);
    // wait for billy to receive HandleStoreEntryAspect via gossip
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    log_i!("Billy got res: {:?}", res);
    // billy asks for reported authored data.
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());
    billy.reply_to_HandleQueryEntry(&query_data).unwrap();
    // #fullsync
    // billy replies to own query
    // let _ = billy.wait_HandleQueryEntry_and_reply();
    // Billy should receive the entry data
    let mut result = billy.find_recv_lib3h_msg(
        0,
        Box::new(one_is!(Lib3hServerProtocol::QueryEntryResult(_))),
    );
    if result.is_none() {
        result = billy.wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::QueryEntryResult(_))))
    }
    let json = result.unwrap();
    log_i!("got result: {:?}", json);
    // Done
    Ok(())
}

/// Reply some data in publish_meta_list
#[cfg_attr(tarpaulin, skip)]
pub fn many_aspects_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    // =====
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    // Author & hold several aspects on same address
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_1.clone()], true)?;
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_2.clone()], false)?;
    alex.hold_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_3.clone()])?;
    log_d!("Alex authored and stored Aspects");

    // billy might receive HandleStoreEntryAspect
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    log_i!("Billy got res 0: {:?}", res);

    // Send AuthoringEntryList
    // =======================
    alex.reply_to_first_HandleGetAuthoringEntryList();
    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);
    //    // Maybe 2nd get for gossiping
    //    let has_received = alex.wait_HandleFetchEntry_and_reply();
    //    log_d!("Alex has_received: {}", has_received);

    // billy might receive HandleStoreEntryAspect
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    log_i!("Billy got res 1: {:?}", res);
    // billy might receive HandleStoreEntryAspect
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    log_i!("Billy got res 2: {:?}", res);
    assert!(res.is_some());

    // Send HoldingEntryList
    // =====================
    // Send HoldingEntryList and should receive a HandleFetchEntry request from network module
    alex.reply_to_first_HandleGetHoldingEntryList();
    // #fullsync
    // wait for Network module to ask for the held data
    let _ = alex.wait_HandleFetchEntry_and_reply();
    // assert!(has_received); // n3h doesnt send fetch because gossip already took care of it

    // billy might receive HandleStoreEntryAspect
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    // assert!(res.is_some()); // n3h doesnt send fetch because gossip already took care of it
    log_i!("Billy got res 3: {:?}", res);
    // Billy asks for that data
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // #fullsync
    // Billy sends that data back to the network
    let query_res_data = billy.reply_to_HandleQueryEntry(&query_data).unwrap();
    let query_result: EntryData = bincode::deserialize(&query_res_data.query_result).unwrap();
    log_i!("sending query_result: {:?}", query_result);

    // Billy should receive requested data
    let result = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::QueryEntryResult(_))))
        .unwrap();
    log_i!("got QueryEntryResult: {:?}", result);
    let query_res_data = unwrap_to!(result => Lib3hServerProtocol::QueryEntryResult);
    let query_result: EntryData = bincode::deserialize(&query_res_data.query_result).unwrap();
    log_i!("got query_result: {:?}", query_result);
    assert_eq!(query_res_data.entry_address, ENTRY_ADDRESS_1.clone());
    assert_eq!(
        query_result.entry_address.clone(),
        query_res_data.entry_address
    );
    assert_eq!(query_result.aspect_list.len(), 3);
    assert!(
        query_result.aspect_list[0].aspect_address.clone() == *ASPECT_ADDRESS_1
            || query_result.aspect_list[0].aspect_address.clone() == *ASPECT_ADDRESS_2
            || query_result.aspect_list[0].aspect_address.clone() == *ASPECT_ADDRESS_3
    );
    assert!(
        query_result.aspect_list[1].aspect_address.clone() == *ASPECT_ADDRESS_1
            || query_result.aspect_list[1].aspect_address.clone() == *ASPECT_ADDRESS_2
            || query_result.aspect_list[1].aspect_address.clone() == *ASPECT_ADDRESS_3
    );
    assert!(
        query_result.aspect_list[2].aspect_address.clone() == *ASPECT_ADDRESS_1
            || query_result.aspect_list[2].aspect_address.clone() == *ASPECT_ADDRESS_2
            || query_result.aspect_list[2].aspect_address.clone() == *ASPECT_ADDRESS_3
    );
    // Done
    Ok(())
}
