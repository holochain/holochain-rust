use constants::*;

use holochain_net::{
    connection::{net_connection::NetSend, NetResult},
    tweetlog::TWEETLOG,
};
use holochain_persistence_api::{cas::content::Address, hash::HashString};
use lib3h_protocol::{
    data_types::{ConnectData, EntryData},
    protocol_client::Lib3hClientProtocol,
    protocol_server::Lib3hServerProtocol,
    uri::Lib3hUri
};
use p2p_node::test_node::TestNode;

/// Do normal setup: 'TrackDna' & 'Connect',
/// and check that we received 'PeerConnected'
#[cfg_attr(tarpaulin, skip)]
pub fn setup_one_node(
    alex: &mut TestNode,
    _billy: &mut TestNode,
    dna_address: &Address,
    _can_connect: bool,
) -> NetResult<()> {
    // Send TrackDna message on both nodes
    alex.track_dna(dna_address, true)
        .expect("Failed sending TrackDna on alex");
    // Check if PeerConnected is received
    let connect_result_1 = alex
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::Connected(_))))
        .unwrap();
    log_i!("self connected result 1: {:?}", connect_result_1);

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
    alex: &mut TestNode,
    billy: &mut TestNode,
    dna_address: &Address,
    can_connect: bool,
) -> NetResult<()> {
    // Send TrackDna message on both nodes
    alex.track_dna(dna_address, true)
        .expect("Failed sending TrackDna on alex");
    // Check if PeerConnected is received
    let connect_result_1 = alex
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::P2pReady)))
        .unwrap();
    println!("self connected result 1: {:?}", connect_result_1);
    billy
        .track_dna(dna_address, true)
        .expect("Failed sending TrackDna on billy");
    let connect_result_2 = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::P2pReady)))
        .unwrap();
    println!("self connected result 2: {:?}", connect_result_2);

    // get ipcServer IDs for each node from the IpcServer's state
    if can_connect {
        let mut _node1_id = String::new();
        let node2_binding = billy.p2p_binding.clone();
        // Connect nodes between them
        println!("connect: node2_binding = {}", node2_binding);
        alex.send(Lib3hClientProtocol::Connect(ConnectData {
            request_id: "alex_send_connect".into(),
            peer_location: Lib3hUri(url::Url::parse(node2_binding.as_str())
                .unwrap_or_else(|_| panic!("malformed node2 url: {:?}", node2_binding))),
            network_id: "alex_to_node2".into(),
        }))?;

        // Make sure Peers are connected
        let result_a = alex
            .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::Connected(_))))
            .unwrap();
        println!("got connect result A: {:?}", result_a);
        one_let!(Lib3hServerProtocol::Connected(d) = result_a {
           assert_eq!(d.request_id, "alex_send_connect");
           assert_eq!(d.uri, Lib3hUri(node2_binding.clone()));
        });
        let result_b = billy
            .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::Connected(_))))
            .unwrap();
        println!("got connect result B: {:?}", result_b);
        one_let!(Lib3hServerProtocol::Connected(d) = result_b {
           assert_eq!(d.request_id, "alex_send_connect");
           assert_eq!(d.uri, Lib3hUri(node2_binding));
        });
    } else {
        println!("can connect is false")
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
pub fn send_test(alex: &mut TestNode, billy: &mut TestNode, can_connect: bool) -> NetResult<()> {
    // Setup
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;

    // Send a message from alex to billy
    alex.send_direct_message(&BILLY_AGENT_ID, ASPECT_CONTENT_1.clone());

    // Check if billy received it
    let res = billy
        .wait_lib3h(Box::new(one_is!(
            Lib3hServerProtocol::HandleSendDirectMessage(_)
        )))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        Lib3hServerProtocol::HandleSendDirectMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(ASPECT_CONTENT_1.to_owned(), *msg.content);

    // Send a message back from billy to alex
    billy.send_response_lib3h(
        msg.clone(),
        format!("echo: {}", std::str::from_utf8(&msg.content).unwrap())
            .as_bytes()
            .to_vec(),
    );
    // Check if alex received it
    let res = alex
        .wait_lib3h(Box::new(one_is!(
            Lib3hServerProtocol::SendDirectMessageResult(_)
        )))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        Lib3hServerProtocol::SendDirectMessageResult(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(
        "echo: hello-1".to_string(),
        std::str::from_utf8(&msg.content).unwrap(),
    );

    // Done
    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
pub fn dht_test(alex: &mut TestNode, billy: &mut TestNode, can_connect: bool) -> NetResult<()> {
    // Setup
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;

    // Alex publish data on the network
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_1.clone()], true)?;

    // #fullsync
    // Alex should receive the data
    let result_a = alex.wait_lib3h(Box::new(one_is!(
        Lib3hServerProtocol::HandleStoreEntryAspect(_)
    )));
    assert!(result_a.is_some());
    log_i!("got HandleStoreEntryAspect on node A: {:?}", result_a);
    // Gossip should ask Alex for the data
    let maybe_fetch_a =
        alex.wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::HandleFetchEntry(_))));
    if let Some(fetch_a) = maybe_fetch_a {
        let fetch = unwrap_to!(fetch_a => Lib3hServerProtocol::HandleFetchEntry);
        let _ = alex.reply_to_HandleFetchEntry(&fetch).unwrap();
    }
    // #fullsync
    // Billy should receive the data
    let result_b = billy.wait_lib3h(Box::new(one_is!(
        Lib3hServerProtocol::HandleStoreEntryAspect(_)
    )));
    assert!(result_b.is_some());
    log_i!("got HandleStoreEntryAspect on node B: {:?}", result_b);

    // Billy asks for that data
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // #fullsync
    // Billy sends that data back to the network
    let _ = billy.reply_to_HandleQueryEntry(&query_data).unwrap();

    // Billy should receive requested data
    let result = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::QueryEntryResult(_))))
        .unwrap();
    log_i!("got QueryEntryResult: {:?}\n\n\n\n", result);

    // Billy asks for unknown data
    let query_data = billy.request_entry(ENTRY_ADDRESS_2.clone());

    // #fullsync
    // Billy sends that data back to the network
    let res = billy.reply_to_HandleQueryEntry(&query_data);
    assert!(res.is_err());
    // Billy should receive FailureResult
    let result = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::FailureResult(_))))
        .unwrap();
    log_i!("got FailureResult: {:?}", result);
    let gen_res = unwrap_to!(result => Lib3hServerProtocol::FailureResult);
    log_i!(
        "Failure result_info: {}",
        std::str::from_utf8(&gen_res.result_info).unwrap()
    );
    assert_eq!(res.err().unwrap(), *gen_res);

    // Done
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
pub fn dht_two_aspects_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;

    // Alex publish data on the network
    alex.author_entry(
        &ENTRY_ADDRESS_1,
        vec![ASPECT_CONTENT_1.clone(), ASPECT_CONTENT_2.clone()],
        true,
    )?;

    // Check if both nodes are asked to store it
    let result_a = alex.wait_lib3h(Box::new(one_is!(
        Lib3hServerProtocol::HandleStoreEntryAspect(_)
    )));
    // #fullsync
    assert!(result_a.is_some());
    let json = result_a.unwrap();
    log_i!("got HandleStoreEntryAspect on node A: {:?}", json);
    let store_data_1 = unwrap_to!(json => Lib3hServerProtocol::HandleStoreEntryAspect);
    let entry_address_1 = ENTRY_ADDRESS_1.clone();
    let aspect_address: HashString = store_data_1.entry_aspect.aspect_address.clone();

    assert_eq!(store_data_1.entry_address, entry_address_1);
    assert!(aspect_address == *ASPECT_ADDRESS_1 || aspect_address == *ASPECT_ADDRESS_2);
    assert!(
        *store_data_1.entry_aspect.aspect.clone() == *ASPECT_CONTENT_1
            || *store_data_1.entry_aspect.aspect.clone() == *ASPECT_CONTENT_2
    );
    // 2nd store
    let result_a = alex.wait_lib3h(Box::new(one_is!(
        Lib3hServerProtocol::HandleStoreEntryAspect(_)
    )));
    assert!(result_a.is_some());
    let json = result_a.unwrap();
    log_i!("got 2nd HandleStoreEntryAspect on node A: {:?}", json);
    let store_data_2 = unwrap_to!(json => Lib3hServerProtocol::HandleStoreEntryAspect);
    assert_ne!(store_data_1, store_data_2);
    assert_eq!(store_data_2.entry_address, entry_address_1);

    let aspect_address_2: HashString = store_data_2.entry_aspect.aspect_address.clone();

    assert!(aspect_address_2 == *ASPECT_ADDRESS_1 || aspect_address_2 == *ASPECT_ADDRESS_2);
    assert!(
        *store_data_2.entry_aspect.aspect.clone() == *ASPECT_CONTENT_1
            || *store_data_2.entry_aspect.aspect.clone() == *ASPECT_CONTENT_2
    );
    // Gossip might ask us for the data
    let maybe_fetch_a =
        alex.wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::HandleFetchEntry(_))));
    if let Some(fetch_a) = maybe_fetch_a {
        let fetch = unwrap_to!(fetch_a => Lib3hServerProtocol::HandleFetchEntry);
        let _ = alex.reply_to_HandleFetchEntry(&fetch).unwrap();
    }
    // #fullsync
    // also check aspects on billy?
    let result_b = billy.wait_lib3h(Box::new(one_is!(
        Lib3hServerProtocol::HandleStoreEntryAspect(_)
    )));
    assert!(result_b.is_some());
    log_i!("got HandleStoreEntryAspect on node B: {:?}", result_b);
    let result_b = billy.wait_lib3h(Box::new(one_is!(
        Lib3hServerProtocol::HandleStoreEntryAspect(_)
    )));
    assert!(result_b.is_some());
    log_i!("got 2nd HandleStoreEntryAspect on node B: {:?}", result_b);

    // Billy asks for that data
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // #fullsync
    // Billy sends that data back to the network
    let _ = billy.reply_to_HandleQueryEntry(&query_data).unwrap();

    // Billy should receive requested data
    let result = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::QueryEntryResult(_))))
        .unwrap();
    log_i!("got QueryEntryResult: {:?}", result);
    let query_data = unwrap_to!(result => Lib3hServerProtocol::QueryEntryResult);
    let query_result: EntryData = bincode::deserialize(&query_data.query_result).unwrap();
    log_i!("got query_result: {:?}", query_result);
    assert_eq!(query_data.entry_address, entry_address_1);
    assert_eq!(query_result.entry_address.clone(), query_data.entry_address);
    assert_eq!(query_result.aspect_list.len(), 2);
    let query_result_aspect_address: HashString =
        query_result.aspect_list[0].aspect_address.clone();

    assert!(
        query_result_aspect_address == *ASPECT_ADDRESS_1
            || query_result_aspect_address == *ASPECT_ADDRESS_2
    );
    // Done
    Ok(())
}

/// Sending a Message before doing a 'TrackDna' should fail
pub fn no_setup_test(alex: &mut TestNode, billy: &mut TestNode, _connect: bool) -> NetResult<()> {
    // Little dance for making alex have its current_dna set to DNA_ADDRESS_A
    alex.track_dna(&DNA_ADDRESS_A, true)
        .expect("Failed sending TrackDna message on alex");
    alex.untrack_current_dna()
        .expect("Failed sending UntrackDna message on alex");
    alex.set_current_dna(&DNA_ADDRESS_A);

    // Send a message from alex to billy
    alex.send_direct_message(&BILLY_AGENT_ID, ASPECT_CONTENT_1.clone());

    // Alex should receive a FailureResult
    let _res = alex.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::FailureResult(_))),
        500,
    );
    // in-memory can't send a failure result back
    // assert!(_res.is_some());

    // Billy should not receive anything
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleSendDirectMessage(_))),
        2000,
    );
    assert!(res.is_none());
    Ok(())
}

/// Sending a Message before doing a 'TrackDna' should fail
pub fn untrack_alex_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;

    // Send Untrack
    alex.untrack_current_dna()
        .expect("Failed sending UntrackDna message on alex");
    alex.set_current_dna(&DNA_ADDRESS_A);

    // Send a message from alex to billy
    let before_count = alex.count_recv_json_messages();
    alex.send_direct_message(&BILLY_AGENT_ID, ASPECT_CONTENT_1.clone());

    // Billy should not receive it.
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleSendDirectMessage(_))),
        2000,
    );
    assert!(res.is_none());
    // Alex should also not receive anything back
    assert_eq!(before_count, alex.count_recv_json_messages());

    // Done
    Ok(())
}

/// Sending a Message before doing a 'TrackDna' should fail
pub fn untrack_billy_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;

    // Send Untrack
    billy
        .untrack_current_dna()
        .expect("Failed sending UntrackDna message on alex");

    // Making sure Untrack has been received
    // TODO: Have server reply with successResult
    alex.listen(1000);
    billy.listen(1000);

    // Send a message from alex to billy
    alex.send_direct_message(&BILLY_AGENT_ID, ASPECT_CONTENT_1.clone());

    log_i!("waiting for FailureResult...");

    // Alex should receive FailureResult
    let result = alex
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::FailureResult(_))))
        .unwrap();
    log_i!("got FailureResult: {:?}", result);

    // Billy should not receive it.
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleSendDirectMessage(_))),
        2000,
    );
    assert!(res.is_none());

    // Done
    Ok(())
}

/// Sending a Message before doing a 'TrackDna' should fail
pub fn retrack_test(alex: &mut TestNode, billy: &mut TestNode, can_connect: bool) -> NetResult<()> {
    // Setup
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;

    // Billy untracks DNA
    billy
        .untrack_current_dna()
        .expect("Failed sending UntrackDna message on billy");

    // Alex untracks DNA
    alex.untrack_current_dna()
        .expect("Failed sending UntrackDna message on alex");

    // Making sure Untrack has been received
    // TODO: Have server reply with successResult
    alex.listen(100);
    billy.listen(100);

    // Billy re-tracks DNA
    billy
        .track_dna(&DNA_ADDRESS_A, true)
        .expect("Failed sending TrackDna on billy");
    // alex re-tracks DNA
    alex.track_dna(&DNA_ADDRESS_A, true)
        .expect("Failed sending TrackDna on alex");

    // Making sure Track has been received
    // TODO: Have server reply with successResult
    alex.listen(100);
    billy.listen(100);

    log_i!("Alternate setup COMPLETE");

    // Send a message from alex to billy
    alex.send_direct_message(&BILLY_AGENT_ID, ASPECT_CONTENT_1.clone());

    // Check if billy received it
    let res = billy
        .wait_lib3h(Box::new(one_is!(
            Lib3hServerProtocol::HandleSendDirectMessage(_)
        )))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        Lib3hServerProtocol::HandleSendDirectMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(
        "hello-1".to_string(),
        std::str::from_utf8(&msg.content).unwrap()
    );

    // Send a message back from billy to alex
    billy.send_response_lib3h(
        msg.clone(),
        format!("echo: {}", std::str::from_utf8(&msg.content).unwrap())
            .as_bytes()
            .to_vec(),
    );
    // Check if alex received it
    let res = alex
        .wait_lib3h(Box::new(one_is!(
            Lib3hServerProtocol::SendDirectMessageResult(_)
        )))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        Lib3hServerProtocol::SendDirectMessageResult(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(
        "echo: hello-1".to_string(),
        std::str::from_utf8(&msg.content).unwrap(),
    );

    // Done
    Ok(())
}

/// Send Lib3hClientProtocol::Shutdown
// TODO @neonphog make sure passes in context of IPC net worker
pub fn shutdown_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;
    assert_eq!(alex.is_network_ready(), true);

    // Do something
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_1.clone()], true)?;
    let _ = billy.listen(200);
    let _ = alex.listen(200);

    // kill alex manually
    alex.send(Lib3hClientProtocol::Shutdown)?;

    // alex should receive 'Terminated' which should set `is_network_ready` to false
    let _ = alex.wait_lib3h_with_timeout(Box::new(|_| true), 200);
    assert_eq!(alex.is_network_ready(), false);

    // Done
    Ok(())
}

/// Entry with no Aspect case: Should no-op
// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
pub fn no_aspect_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;

    // Alex publish Entry but no Aspect on the network
    alex.author_entry(&ENTRY_ADDRESS_1, vec![], true)?;

    // Billy asks for missing metadata on the network.
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // Alex sends that data back to the network
    let res = alex.reply_to_HandleQueryEntry(&query_data);
    log_i!("alex responds with: {:?}", res);
    let info = std::string::String::from_utf8_lossy(&res.err().unwrap().result_info).into_owned();
    log_i!("got result_info: {}", info);
    assert_eq!(info, "No entry found");
    // Done
    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
pub fn two_authors_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    setup_two_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;

    // Again but 'wait' at the end
    // Alex publishs data & meta on the network
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_1.clone()], true)?;
    // #fullsync
    let _ = alex.wait_lib3h(Box::new(one_is!(
        Lib3hServerProtocol::HandleStoreEntryAspect(_)
    )));
    // wait for broadcast
    // Gossip might ask us for the data
    let maybe_fetch_a =
        alex.wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::HandleFetchEntry(_))));
    if let Some(fetch_a) = maybe_fetch_a {
        let fetch = unwrap_to!(fetch_a => Lib3hServerProtocol::HandleFetchEntry);
        let _ = alex.reply_to_HandleFetchEntry(&fetch).unwrap();
    }
    // Check if billy is asked to store it
    let result = billy.wait_lib3h(Box::new(one_is!(
        Lib3hServerProtocol::HandleStoreEntryAspect(_)
    )));
    assert!(result.is_some());
    log_i!("Billy got HandleStoreEntryAspect: {:?}", result.unwrap());

    // Billy authors second aspect
    billy.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_2.clone()], true)?;
    // #fullsync
    let _ = billy.wait_lib3h(Box::new(one_is!(
        Lib3hServerProtocol::HandleStoreEntryAspect(_)
    )));
    // Gossip might ask us for the data
    let maybe_fetch_b =
        billy.wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::HandleFetchEntry(_))));
    if let Some(fetch_b) = maybe_fetch_b {
        let fetch = unwrap_to!(fetch_b => Lib3hServerProtocol::HandleFetchEntry);
        let _ = billy.reply_to_HandleFetchEntry(&fetch).unwrap();
    }
    // wait for gossip / broadcast
    // Check if billy is asked to store it
    let result = alex.wait_lib3h(Box::new(one_is!(
        Lib3hServerProtocol::HandleStoreEntryAspect(_)
    )));
    // #fullsync
    assert!(result.is_some());
    log_i!("Alex got HandleStoreEntryAspect: {:?}", result.unwrap());

    // Billy asks for that data
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // #fullsync
    // Billy sends that data back to the network
    let _ = billy.reply_to_HandleQueryEntry(&query_data).unwrap();

    // Billy should receive requested data
    let result = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::QueryEntryResult(_))))
        .unwrap();
    log_i!("got QueryEntryResult: {:?}", result);
    let query_data = unwrap_to!(result => Lib3hServerProtocol::QueryEntryResult);
    let query_result: EntryData = bincode::deserialize(&query_data.query_result).unwrap();
    log_i!("got query_result: {:?}", query_result);
    let entry_address_1 = ENTRY_ADDRESS_1.clone();

    assert_eq!(query_data.entry_address, entry_address_1);
    assert_eq!(query_result.entry_address.clone(), query_data.entry_address);
    assert_eq!(query_result.aspect_list.len(), 2);
    let aspect_address_1: HashString = query_result.aspect_list[0].aspect_address.clone();

    assert!(aspect_address_1 == *ASPECT_ADDRESS_1 || aspect_address_1 == *ASPECT_ADDRESS_2);
    // Done
    Ok(())
}
