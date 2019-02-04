use constants::*;
use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    json_protocol::{ConnectData, JsonProtocol, TrackDnaData},
    net_connection::NetSend,
    NetResult,
};
use p2p_node::P2pNode;

// TODO make test: Sending a Message before doing a 'TrackApp' should fail
//fn no_track_test(
//    node1: &mut P2pNode,
//    node2: &mut P2pNode,
//    can_test_connect: bool,
//) -> NetResult<()> {
//    // FIXME: not calling trackApp should make sends or whatever else fail
//    Ok(())
//}

// TODO make test: Sending a Message before doing a 'Connect' should fail.
// fn no_connect_test()

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

    // Check if both nodes received a HandleStore command.
    let result_a = alex
        .wait(Box::new(one_is!(JsonProtocol::HandleStoreEntry(_))))
        .unwrap();
    println!("got store result A: {:?}\n", result_a);
    let result_b = billy
        .wait(Box::new(one_is!(JsonProtocol::HandleStoreEntry(_))))
        .unwrap();
    println!("got store result B: {:?}\n", result_b);
    assert!(billy.entry_store.contains_key(address));

    let fetch_data = billy.request_entry(address.clone());

    // Alex having that data, sends it to the network.
    alex.reply_to_HandleFetchEntry(&fetch_data)?;

    // billy should receive the data it requested from the netowrk
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchEntryResult(_))))
        .unwrap();
    println!("got dht data result: {:?}", result);

    Ok(())
}

/// Tests if we can get back metadata published on the network
#[cfg_attr(tarpaulin, skip)]
fn confirm_published_metadata(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    address: &Address,
    attribute: &str,
    content: &serde_json::Value,
) -> NetResult<()> {
    // Alex publishs metadata on the network
    alex.author_meta(address, attribute, content, true)?;
    // Check if both nodes received a HandleStore command.
    let result_a = alex
        .wait(Box::new(one_is!(JsonProtocol::HandleStoreMeta(_))))
        .unwrap();
    println!("got store meta result 1: {:?}", result_a);
    let result_b = billy
        .wait(Box::new(one_is!(JsonProtocol::HandleStoreMeta(_))))
        .unwrap();
    println!("got store meta result 2: {:?}", result_b);
    assert!(billy
        .meta_store
        .contains_key(&(address.clone(), META_ATTRIBUTE.to_string())));

    // Billy asks for that metadata on the network.
    let fetch_meta = billy.request_meta(address.clone(), META_ATTRIBUTE.to_string());

    // Alex having that metadata, sends it to the network.
    alex.reply_to_HandleFetchMeta(&fetch_meta)?;

    // billy should receive the metadata it requested from the netowrk
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))))
        .unwrap();
    println!("got dht meta result: {:?}", result);
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
    let connect_result_1 = alex
        .wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))
        .unwrap();
    println!("self connected result 1: {:?}", connect_result_1);
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
    println!("self connected result 2: {:?}", connect_result_2);

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
        println!("connect: node2_binding = {}", node2_binding);
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
        println!("got connect result A: {:?}", result_a);
        one_let!(JsonProtocol::PeerConnected(d) = result_a {
            assert_eq!(d.agent_id, BILLY_AGENT_ID);
        });
        let result_b = billy
            .wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))
            .unwrap();
        println!("got connect result B: {:?}", result_b);
        one_let!(JsonProtocol::PeerConnected(d) = result_b {
            assert_eq!(d.agent_id, ALEX_AGENT_ID);
        });
    }

    // Make sure we received everything we needed from network module
    // TODO: Make a more robust function that waits for certain messages in msg log (with timeout that panics)
    let _msg_count = alex.listen(100);
    let _msg_count = billy.listen(100);

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
    println!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::HandleSendMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!("\"hello\"".to_string(), msg.content.to_string());

    // Send a message back from billy to alex
    billy.send_reponse(
        msg.clone(),
        json!(format!("echo: {}", msg.content.to_string())),
    );
    // Check if alex received it
    let res = alex
        .wait(Box::new(one_is!(JsonProtocol::SendMessageResult(_))))
        .unwrap();
    println!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::SendMessageResult(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(
        "\"echo: \\\"hello\\\"\"".to_string(),
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
        META_ATTRIBUTE,
        &META_CONTENT_1,
    )?;

    // Again but now send metadata first
    confirm_published_metadata(
        alex,
        billy,
        &ENTRY_ADDRESS_2,
        META_ATTRIBUTE,
        &META_CONTENT_2,
    )?;
    confirm_published_data(alex, billy, &ENTRY_ADDRESS_2, &ENTRY_CONTENT_2)?;

    // Again but 'wait' at the end
    // Alex publishs data & meta on the network
    alex.author_entry(&ENTRY_ADDRESS_3, &ENTRY_CONTENT_3, true)?;
    alex.author_meta(
        &ENTRY_ADDRESS_3,
        &META_ATTRIBUTE.to_string(),
        &META_CONTENT_3,
        true,
    )?;
    // Billy sends FetchEntry message
    let fetch_data = billy.request_entry(ENTRY_ADDRESS_3.clone());
    // Billy sends HandleFetchEntryResult message
    billy.reply_to_HandleFetchEntry(&fetch_data)?;
    // Billy sends FetchMeta message
    let fetch_meta = billy.request_meta(ENTRY_ADDRESS_3.clone(), META_ATTRIBUTE.to_string());
    // Alex sends HandleFetchMetaResult message
    alex.reply_to_HandleFetchMeta(&fetch_meta)?;
    // billy should receive requested metadata
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))))
        .unwrap();
    println!("got GetMetaResult: {:?}", result);
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
    let result_a = alex
        .wait(Box::new(one_is!(JsonProtocol::HandleStoreEntry(_))))
        .unwrap();
    println!("got HandleStoreEntry on node A: {:?}", result_a);
    let result_b = billy
        .wait(Box::new(one_is!(JsonProtocol::HandleStoreEntry(_))))
        .unwrap();
    println!("got HandleStoreEntry on node B: {:?}", result_b);
    assert!(billy.entry_store.contains_key(&ENTRY_ADDRESS_1));

    // Billy asks for that data
    let fetch_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // Alex sends that data back to the network
    alex.reply_to_HandleFetchEntry(&fetch_data)?;

    // Billy should receive requested data
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FetchEntryResult(_))))
        .unwrap();
    println!("got FetchEntryResult: {:?}", result);

    // Billy asks for unknown data
    let fetch_data = billy.request_entry(ENTRY_ADDRESS_2.clone());

    // Alex sends that data back to the network
    alex.reply_to_HandleFetchEntry(&fetch_data)?;

    // Billy should receive FailureResult
    let result = billy
        .wait(Box::new(one_is!(JsonProtocol::FailureResult(_))))
        .unwrap();
    println!("got FailureResult: {:?}", result);

    // Done
    Ok(())
}
