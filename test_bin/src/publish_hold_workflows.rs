use basic_workflows::setup_two_nodes;
use constants::*;
use holochain_net::{
    connection::{
        json_protocol::{EntryData, JsonProtocol},
        NetResult,
    },
    tweetlog::*,
};
use p2p_node::test_node::TestNode;

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
    // Alex sends back a failureResult response to the network
    let res = alex.reply_to_HandleQueryEntry(&query_data);
    assert!(res.is_err());
    // Billy should receive the failureResult back
    let result = billy
        .wait_json(Box::new(one_is!(JsonProtocol::FailureResult(_))))
        .unwrap();
    log_i!("got result: {:?}", result);
    let gen_res = unwrap_to!(result => JsonProtocol::FailureResult);
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
    // Should receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);
    // billy might receive HandleStoreEntryAspect
    let _ = billy.wait_json_with_timeout(
        Box::new(one_is!(JsonProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    // billy asks for reported authored data.
    billy.request_entry(ENTRY_ADDRESS_1.clone());
    let has_received = alex.wait_HandleQueryEntry_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleQueryEntry_and_reply();
    }
    // Billy should receive the entry data
    let mut result =
        billy.find_recv_json_msg(0, Box::new(one_is!(JsonProtocol::QueryEntryResult(_))));
    if result.is_none() {
        result = billy.wait_json(Box::new(one_is!(JsonProtocol::QueryEntryResult(_))))
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
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_1.clone()], true)?;
    alex.reply_to_first_HandleGetAuthoringEntryList();
    // Should NOT receive a HandleFetchEntry request from network module
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(!has_received);
    // billy asks for reported published data.
    billy.request_entry(ENTRY_ADDRESS_1.clone());
    // Alex or Billy receives and replies to a HandleFetchEntry
    let has_received = alex.wait_HandleQueryEntry_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleQueryEntry_and_reply();
    }
    // Billy should receive the entry data back
    let mut result =
        billy.find_recv_json_msg(0, Box::new(one_is!(JsonProtocol::QueryEntryResult(_))));
    if result.is_none() {
        result = billy.wait_json(Box::new(one_is!(JsonProtocol::QueryEntryResult(_))))
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
    let _ = billy.wait_json_with_timeout(
        Box::new(one_is!(JsonProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    // billy asks for reported authored data.
    billy.request_entry(ENTRY_ADDRESS_1.clone());
    let has_received = alex.wait_HandleQueryEntry_and_reply();
    if !has_received {
        let _has_received = billy.wait_HandleQueryEntry_and_reply();
    }
    // Billy should receive the entry data
    let mut result =
        billy.find_recv_json_msg(0, Box::new(one_is!(JsonProtocol::QueryEntryResult(_))));
    if result.is_none() {
        result = billy.wait_json(Box::new(one_is!(JsonProtocol::QueryEntryResult(_))))
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
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    // Author meta and reply to HandleGetPublishingMetaList
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_1.clone()], true)?;
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_2.clone()], false)?;
    alex.hold_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_3.clone()])?;
    log_d!("aspects authored and stored");

    // Send AuthoringEntryList and should receive a HandleFetchEntry request from network module
    alex.reply_to_first_HandleGetAuthoringEntryList();
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);

    // Send AuthoringEntryList and should receive a HandleFetchEntry request from network module
    alex.reply_to_first_HandleGetHoldingEntryList();
    // wait for gossip to ask for the held data
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);
    // Billy asks for that data
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // Alex sends that data back to the network
    let _ = alex.reply_to_HandleQueryEntry(&query_data).unwrap();

    // Billy should receive requested data
    let result = billy
        .wait_json(Box::new(one_is!(JsonProtocol::QueryEntryResult(_))))
        .unwrap();
    log_i!("got QueryEntryResult: {:?}", result);
    let query_data = unwrap_to!(result => JsonProtocol::QueryEntryResult);
    let query_result: EntryData = bincode::deserialize(&query_data.query_result).unwrap();
    log_i!("got query_result: {:?}", query_result);
    assert_eq!(query_data.entry_address, ENTRY_ADDRESS_1.clone());
    assert_eq!(query_result.entry_address.clone(), query_data.entry_address);
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
