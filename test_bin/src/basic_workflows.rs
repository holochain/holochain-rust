use constants::*;
use holochain_core_types::cas::content::Address;
use holochain_net::{
    connection::{
        json_protocol::{ConnectData, JsonProtocol, TrackDnaData},
        net_connection::NetSend,
        NetResult,
    },
    tweetlog::TWEETLOG,
};
use p2p_node::P2pNode;

/// Tests if we can get back data published on the network
#[cfg_attr(tarpaulin, skip)]
fn confirm_published_data(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    address: &Address,
    content: &serde_json::Value,
) -> NetResult<()> {
    // Alex publishs data on the network
    alex.author_entry(address.into(), content, true)?;

    // Check if both nodes are asked to store it
    let result_a = alex.wait(Box::new(one_is!(JsonProtocol::HandleStoreEntry(_))));
    // #fulldht
    assert!(result_a.is_some());
    log_i!("got HandleStoreEntry on node A: {:?}", result_a);

    let result_b = billy.wait(Box::new(one_is!(JsonProtocol::HandleStoreEntry(_))));
    assert!(result_b.is_some());
    log_i!("got HandleStoreEntry on node B: {:?}", result_b);

    let fetch_data = billy.request_entry(address.clone());

    // Alex having that data, sends it to the network.
    alex.reply_to_HandleFetchEntry(&fetch_data)?;

    // billy should receive the data it requested from the netowrk
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchEntryResult(_))))
        .unwrap();
    log_i!("got dht Entry result: {:?}", result);

    Ok(())
}

/// Tests if we can get back metadata published on the network
#[cfg_attr(tarpaulin, skip)]
fn confirm_published_metadata(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    address: &Address,
    attribute: &str,
    link_entry_address: &serde_json::Value,
) -> NetResult<()> {
    // Alex publishs metadata on the network
    let _meta_key = alex.author_meta(address, attribute, link_entry_address, true)?;

    // Check if both nodes are asked to store it
    let result_a = alex.wait(Box::new(one_is!(JsonProtocol::HandleStoreMeta(_))));
    // #fulldht
    assert!(result_a.is_some());
    log_i!("got HandleStoreMeta on node A: {:?}", result_a);
    let result_b = billy.wait(Box::new(one_is!(JsonProtocol::HandleStoreMeta(_))));
    assert!(result_b.is_some());
    log_i!("got HandleStoreMeta on node B: {:?}", result_b);

    // Billy asks for that metadata on the network.
    let fetch_meta = billy.request_meta(address.clone(), META_LINK_ATTRIBUTE.to_string());

    // Alex having that metadata, sends it to the network.
    alex.reply_to_HandleFetchMeta(&fetch_meta)?;

    // billy should receive the metadata it requested from the netowrk
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))))
        .unwrap();
    log_i!("got dht meta result: {:?}", result);
    // Done
    Ok(())
}

/// Do normal setup: 'TrackDna' & 'Connect',
/// and check that we received 'PeerConnected'
#[cfg_attr(tarpaulin, skip)]
pub fn setup_one_node(
    alex: &mut P2pNode,
    _billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Send TrackDna message on both nodes
    alex.send(
        JsonProtocol::TrackDna(TrackDnaData {
            dna_address: DNA_ADDRESS.clone(),
            agent_id: ALEX_AGENT_ID.to_string(),
        })
        .into(),
    )
    .expect("Failed sending TrackDnaData on alex");
    // Check if PeerConnected is received
    let connect_result_1 = alex
        .wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))
        .unwrap();
    log_i!("self connected result 1: {:?}", connect_result_1);

    // get ipcServer IDs for each node from the IpcServer's state
    if can_connect {
        let mut _node1_binding = String::new();

        alex.send(JsonProtocol::GetState.into())
            .expect("Failed sending RequestState on alex");
        let alex_state = alex
            .wait(Box::new(one_is!(JsonProtocol::GetStateResult(_))))
            .unwrap();

        one_let!(JsonProtocol::GetStateResult(state) = alex_state {
            _node1_binding = state.id
        });

        // Connect nodes between them
        log_i!("node1_binding = {}", _node1_binding);
    }

    // Make sure we received everything we needed from network module
    // TODO: Make a more robust function that waits for certain messages in msg log (with timeout that panics)
    let _msg_count = alex.listen(100);

    let mut time_ms: usize = 0;
    while !alex.is_network_ready() && time_ms < 1000 {
        let _msg_count = alex.listen(100);
        time_ms += 100;
    }

    log_i!("setup_one_node() COMPLETE \n\n\n");

    // Done
    Ok(())
}

/// Do normal setup: 'TrackDna' & 'Connect',
/// and check that we received 'PeerConnected'
#[cfg_attr(tarpaulin, skip)]
pub fn setup_two_nodes(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Send TrackDna message on both nodes
    alex.send(
        JsonProtocol::TrackDna(TrackDnaData {
            dna_address: DNA_ADDRESS.clone(),
            agent_id: ALEX_AGENT_ID.to_string(),
        })
        .into(),
    )
    .expect("Failed sending TrackDnaData on alex");
    // Check if PeerConnected is received
    let connect_result_1 = alex
        .wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))
        .unwrap();
    log_i!("self connected result 1: {:?}", connect_result_1);
    billy
        .send(
            JsonProtocol::TrackDna(TrackDnaData {
                dna_address: DNA_ADDRESS.clone(),
                agent_id: BILLY_AGENT_ID.to_string(),
            })
            .into(),
        )
        .expect("Failed sending TrackDnaData on billy");
    let connect_result_2 = billy
        .wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))
        .unwrap();
    log_i!("self connected result 2: {:?}", connect_result_2);

    // get ipcServer IDs for each node from the IpcServer's state
    if can_connect {
        let mut _node1_id = String::new();
        let mut node2_binding = String::new();

        alex.send(JsonProtocol::GetState.into())
            .expect("Failed sending RequestState on alex");
        let alex_state = alex
            .wait(Box::new(one_is!(JsonProtocol::GetStateResult(_))))
            .unwrap();
        billy
            .send(JsonProtocol::GetState.into())
            .expect("Failed sending RequestState on billy");
        let billy_state = billy
            .wait(Box::new(one_is!(JsonProtocol::GetStateResult(_))))
            .unwrap();

        one_let!(JsonProtocol::GetStateResult(state) = alex_state {
            _node1_id = state.id
        });
        one_let!(JsonProtocol::GetStateResult(state) = billy_state {
            if !state.bindings.is_empty() {
                node2_binding = state.bindings[0].clone();
            }
        });

        // Connect nodes between them
        log_i!("connect: node2_binding = {}", node2_binding);
        alex.send(
            JsonProtocol::Connect(ConnectData {
                peer_address: node2_binding.into(),
            })
            .into(),
        )?;

        // Make sure Peers are connected
        let result_a = alex
            .wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))
            .unwrap();
        log_i!("got connect result A: {:?}", result_a);
        one_let!(JsonProtocol::PeerConnected(d) = result_a {
            assert_eq!(d.agent_id, BILLY_AGENT_ID);
        });
        let result_b = billy
            .wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))
            .unwrap();
        log_i!("got connect result B: {:?}", result_b);
        one_let!(JsonProtocol::PeerConnected(d) = result_b {
            assert_eq!(d.agent_id, ALEX_AGENT_ID);
        });
    }

    // Make sure we received everything we needed from network module
    // TODO: Make a more robust function that waits for certain messages in msg log (with timeout that panics)
    let _msg_count = alex.listen(100);
    let _msg_count = billy.listen(100);

    let mut time_ms: usize = 0;
    while !(alex.is_network_ready() && billy.is_network_ready()) && time_ms < 1000 {
        let _msg_count = alex.listen(100);
        let _msg_count = billy.listen(100);
        time_ms += 100;
    }

    log_i!("setup_two_nodes() COMPLETE \n\n\n");

    // Done
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
pub fn send_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: send_test()");
    setup_two_nodes(alex, billy, can_connect)?;

    // Send a message from alex to billy
    alex.send_message(BILLY_AGENT_ID.to_string(), ENTRY_CONTENT_1.clone());

    // Check if billy received it
    let res = billy
        .wait(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::HandleSendMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(
        "{\"entry\":{\"content\":\"hello\"}}".to_string(),
        msg.content.to_string()
    );

    // Send a message back from billy to alex
    billy.send_reponse(
        msg.clone(),
        json!(format!("echo: {}", msg.content.to_string())),
    );
    // Check if alex received it
    let res = alex
        .wait(Box::new(one_is!(JsonProtocol::SendMessageResult(_))))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::SendMessageResult(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(
        "\"echo: {\\\"entry\\\":{\\\"content\\\":\\\"hello\\\"}}\"".to_string(),
        msg.content.to_string()
    );

    // Done
    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
pub fn meta_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: meta_test()");
    setup_two_nodes(alex, billy, can_connect)?;

    // Send data & metadata on same address
    confirm_published_data(alex, billy, &ENTRY_ADDRESS_1, &ENTRY_CONTENT_1)?;
    confirm_published_metadata(
        alex,
        billy,
        &ENTRY_ADDRESS_1,
        META_LINK_ATTRIBUTE,
        &META_LINK_CONTENT_1,
    )?;
    log_i!("confirm_published_metadata(ENTRY_ADDRESS_1) COMPLETE");

    // Again but now send metadata first
    confirm_published_metadata(
        alex,
        billy,
        &ENTRY_ADDRESS_2,
        META_LINK_ATTRIBUTE,
        &META_LINK_CONTENT_2,
    )?;
    confirm_published_data(alex, billy, &ENTRY_ADDRESS_2, &ENTRY_CONTENT_2)?;
    log_i!("confirm_published_metadata(ENTRY_ADDRESS_2) COMPLETE");

    // Again but 'wait' at the end
    // Alex publishs data & meta on the network
    alex.author_entry(&ENTRY_ADDRESS_3, &ENTRY_CONTENT_3, true)?;
    alex.author_meta(
        &ENTRY_ADDRESS_3,
        &META_LINK_ATTRIBUTE.to_string(),
        &META_LINK_CONTENT_3,
        true,
    )?;

    // wait for gossip
    // Check if billy is asked to store it
    let result = billy.wait(Box::new(one_is!(JsonProtocol::HandleStoreEntry(_))));
    // #fulldht
    assert!(result.is_some());
    log_i!("Billy got HandleStoreEntry: {:?}", result);

    let result = billy.wait(Box::new(one_is!(JsonProtocol::HandleStoreMeta(_))));
    assert!(result.is_some());
    log_i!("Billy got HandleStoreEntry: {:?}", result);

    // Billy sends FetchEntry message
    let fetch_data = billy.request_entry(ENTRY_ADDRESS_3.clone());
    // Billy sends HandleFetchEntryResult message
    alex.reply_to_HandleFetchEntry(&fetch_data)?;
    // Billy sends FetchMeta message
    let fetch_meta = billy.request_meta(ENTRY_ADDRESS_3.clone(), META_LINK_ATTRIBUTE.to_string());
    // Alex sends HandleFetchMetaResult message
    alex.reply_to_HandleFetchMeta(&fetch_meta)?;
    // billy should receive requested metadata
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))))
        .unwrap();
    log_i!("got GetMetaResult: {:?}", result);
    let meta_data = unwrap_to!(result => JsonProtocol::FetchMetaResult);
    assert_eq!(meta_data.entry_address, ENTRY_ADDRESS_3.clone());
    assert_eq!(meta_data.attribute, META_LINK_ATTRIBUTE.clone());
    assert_eq!(meta_data.content_list.len(), 1);
    assert_eq!(meta_data.content_list[0], META_LINK_CONTENT_3.clone());
    // Done
    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
pub fn dht_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: dht_test()");
    setup_two_nodes(alex, billy, can_connect)?;

    // Alex publish data on the network
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, true)?;

    // Check if both nodes are asked to store it
    let result_a = alex.wait(Box::new(one_is!(JsonProtocol::HandleStoreEntry(_))));
    // #fulldht
    assert!(result_a.is_some());
    log_i!("got HandleStoreEntry on node A: {:?}", result_a);

    let result_b = billy.wait(Box::new(one_is!(JsonProtocol::HandleStoreEntry(_))));
    assert!(result_b.is_some());
    log_i!("got HandleStoreEntry on node B: {:?}", result_b);

    // Billy asks for that data
    let fetch_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // Alex sends that data back to the network
    alex.reply_to_HandleFetchEntry(&fetch_data)?;

    // Billy should receive requested data
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchEntryResult(_))))
        .unwrap();
    log_i!("got FetchEntryResult: {:?}", result);

    // Billy asks for unknown data
    let fetch_data = billy.request_entry(ENTRY_ADDRESS_2.clone());

    // Alex sends that data back to the network
    alex.reply_to_HandleFetchEntry(&fetch_data)?;

    // Billy should receive FailureResult
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FailureResult(_))))
        .unwrap();
    log_i!("got FailureResult: {:?}", result);

    // Done
    Ok(())
}

/// Sending a Message before doing a 'TrackDna' should fail
pub fn no_setup_test(alex: &mut P2pNode, billy: &mut P2pNode, _connect: bool) -> NetResult<()> {
    // Send a message from alex to billy
    alex.send_message(BILLY_AGENT_ID.to_string(), ENTRY_CONTENT_1.clone());

    // Alex should receive a FailureResult
    let res = alex.wait_with_timeout(Box::new(one_is!(JsonProtocol::FailureResult(_))), 500);
    // in-memory can't send a failure result back
    // assert!(res.is_some());

    // Billy should not receive anything
    let res = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))), 2000);
    assert!(res.is_none());
    Ok(())
}

/// Sending a Message before doing a 'TrackDna' should fail
pub fn untrack_alex_test(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: untrack_alex_test()");
    setup_two_nodes(alex, billy, can_connect)?;

    // Send Untrack
    alex.send(
        JsonProtocol::UntrackDna(TrackDnaData {
            dna_address: DNA_ADDRESS.clone(),
            agent_id: ALEX_AGENT_ID.to_string(),
        })
        .into(),
    )
    .expect("Failed sending UntrackDna message on alex");

    // Send a message from alex to billy
    let before_count = alex.count_recv_json_messages();
    alex.send_message(BILLY_AGENT_ID.to_string(), ENTRY_CONTENT_1.clone());

    // Billy should not receive it.
    let res = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))), 2000);
    assert!(res.is_none());
    // Alex should also not receive anything back
    assert_eq!(before_count, alex.count_recv_json_messages());

    // Done
    Ok(())
}

/// Sending a Message before doing a 'TrackDna' should fail
pub fn untrack_billy_test(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: untrack_billy_test()");
    setup_two_nodes(alex, billy, can_connect)?;

    // Send Untrack
    billy
        .send(
            JsonProtocol::UntrackDna(TrackDnaData {
                dna_address: DNA_ADDRESS.clone(),
                agent_id: BILLY_AGENT_ID.to_string(),
            })
            .into(),
        )
        .expect("Failed sending UntrackDna message on alex");

    // Making sure Untrack has been received
    // TODO: Have server reply with successResult
    alex.listen(1000);
    billy.listen(1000);

    // Send a message from alex to billy
    alex.send_message(BILLY_AGENT_ID.to_string(), ENTRY_CONTENT_1.clone());

    // Alex should receive FailureResult
    let result = alex
        .wait(Box::new(one_is!(JsonProtocol::FailureResult(_))))
        .unwrap();
    log_i!("got FailureResult: {:?}", result);

    // Billy should not receive it.
    let res = billy.wait_with_timeout(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))), 2000);
    assert!(res.is_none());

    // Done
    Ok(())
}

/// Sending a Message before doing a 'TrackDna' should fail
pub fn retrack_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: untrack_billy_test()");
    setup_two_nodes(alex, billy, can_connect)?;

    // Billy untracks DNA
    billy
        .send(
            JsonProtocol::UntrackDna(TrackDnaData {
                dna_address: DNA_ADDRESS.clone(),
                agent_id: BILLY_AGENT_ID.to_string(),
            })
            .into(),
        )
        .expect("Failed sending UntrackDna message on billy");

    // Alex untracks DNA
    alex.send(
        JsonProtocol::UntrackDna(TrackDnaData {
            dna_address: DNA_ADDRESS.clone(),
            agent_id: ALEX_AGENT_ID.to_string(),
        })
        .into(),
    )
    .expect("Failed sending UntrackDna message on alex");

    // Making sure Untrack has been received
    // TODO: Have server reply with successResult
    alex.listen(100);
    billy.listen(100);

    // Billy re-tracks DNA
    billy
        .send(
            JsonProtocol::TrackDna(TrackDnaData {
                dna_address: DNA_ADDRESS.clone(),
                agent_id: BILLY_AGENT_ID.to_string(),
            })
            .into(),
        )
        .expect("Failed sending TrackDnaData on billy");
    // alex re-tracks DNA
    alex.send(
        JsonProtocol::TrackDna(TrackDnaData {
            dna_address: DNA_ADDRESS.clone(),
            agent_id: ALEX_AGENT_ID.to_string(),
        })
        .into(),
    )
    .expect("Failed sending TrackDnaData on alex");

    // Making sure Track has been received
    // TODO: Have server reply with successResult
    alex.listen(100);
    billy.listen(100);

    log_i!("Alternate setup COMPLETE");

    // Send a message from alex to billy
    alex.send_message(BILLY_AGENT_ID.to_string(), ENTRY_CONTENT_1.clone());

    // Check if billy received it
    let res = billy
        .wait(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::HandleSendMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(
        "{\"entry\":{\"content\":\"hello\"}}".to_string(),
        msg.content.to_string()
    );

    // Send a message back from billy to alex
    billy.send_reponse(
        msg.clone(),
        json!(format!("echo: {}", msg.content.to_string())),
    );
    // Check if alex received it
    let res = alex
        .wait(Box::new(one_is!(JsonProtocol::SendMessageResult(_))))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::SendMessageResult(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(
        "\"echo: {\\\"entry\\\":{\\\"content\\\":\\\"hello\\\"}}\"".to_string(),
        msg.content.to_string()
    );

    // Done
    Ok(())
}
