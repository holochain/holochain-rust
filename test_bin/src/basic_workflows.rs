use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    json_protocol::{
        ConnectData, DhtData, DhtMetaData, FetchDhtData, FetchDhtMetaData, JsonProtocol, MessageData,
        TrackDnaData, HandleDhtResultData, HandleDhtMetaResultData,
    },
    net_connection::NetSend,
    NetResult,
};
use p2p_node::P2pNode;
use constants::*;

// MACROS
macro_rules! one_let {
    ($p:pat = $enum:ident $code:tt) => {
        if let $p = $enum {
            $code
        } else {
            unimplemented!();
        }
    };
}

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
fn confirm_published_data(alex: &mut P2pNode, billy: &mut P2pNode, address: &Address) -> NetResult<()> {
    // Alex publishs data on the network
    alex.author_data(
        &example_dna_address(),
        address.into(),
        json!("hello"),
        true,
    )?;

    // Check if both nodes received a HandleStore command.
    let result_a = alex.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtData(_))))?;
    println!(" got store result A: {:?}\n", result_a);
    let result_b = billy.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtData(_))))?;
    println!("got store result B: {:?}\n", result_b);
    assert!(billy.data_store.contains(address));

    // Billy asks for that data on the network.
    let fetch_data = FetchDhtData {
        request_id: "testGetEntry".to_string(),
        dna_address: example_dna_address(),
        requester_agent_id: BILLY_AGENT_ID.to_string(),
        data_address: address.clone(),
    };
    billy.send(JsonProtocol::FetchDhtData(fetch_data).into())?;

    // Alex having that data, sends it to the network.
    alex.reply_fetch_data(&fetch_data);

    // billy should receive the data it requested from the netowrk
    let result = billy.wait(Box::new(one_is!(JsonProtocol::FetchDhtDataResult(_))))?;
    println!("got dht data result: {:?}", result);

    Ok(())
}

/// Tests if we can get back metadata published on the network
#[cfg_attr(tarpaulin, skip)]
fn confirm_published_metadata(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    address: &Address,
) -> NetResult<()> {
    // Alex publishs metadata on the network
    alex.author_meta(
        &example_dna_address(),
             address,
            &META_ATTRIBUTE.to_string(),
            json!("hello-meta"),
        true,
    )?;
    // Check if both nodes received a HandleStore command.
    let result_a = alex.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtMeta(_))))?;
    println!("got store meta result 1: {:?}", result_a);
    let result_b = billy.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtMeta(_))))?;
    println!("got store meta result 2: {:?}", result_b);
    assert!(billy.meta_store.contains((address, META_ATTRIBUTE)));

    // Billy asks for that metadata on the network.
    let fetch_meta = FetchDhtMetaData {
        request_id: "testGetMeta".to_string(),
        dna_address: example_dna_address(),
        requester_agent_id: BILLY_AGENT_ID.to_string(),
        data_address: address.clone(),
        attribute: META_ATTRIBUTE.to_string(),
    };
    billy.send(JsonProtocol::FetchDhtMeta(fetch_meta).into())?;

    // Alex having that metadata, sends it to the network.
    alex.reply_fetch_meta(&fetch_meta)?;

    // billy should receive the metadata it requested from the netowrk
    let result = billy.wait(Box::new(one_is!(JsonProtocol::FetchDhtMetaResult(_))))?;
    println!("got dht meta result: {:?}", result);

    Ok(())
}

/// Do normal setup: 'TrackDna' & 'Connect',
/// and check that we received 'PeerConnected'
#[cfg_attr(tarpaulin, skip)]
pub fn setup_normal(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Send TrackDna message on both nodes
    alex.send(
        JsonProtocol::TrackDna(TrackDnaData {
            dna_address: example_dna_address(),
            agent_id: ALEX_AGENT_ID.to_string(),
        })
            .into(),
    )
        .expect("Failed sending TrackDnaData on alex");
    let connect_result_1 = alex.wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))?;
    println!("self connected result 1: {:?}", connect_result_1);
    billy
        .send(
            JsonProtocol::TrackDna(TrackDnaData {
                dna_address: example_dna_address(),
                agent_id: BILLY_AGENT_ID.to_string(),
            })
                .into(),
        )
        .expect("Failed sending TrackDnaData on billy");
    let connect_result_2 = billy.wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))?;
    println!("self connected result 2: {:?}", connect_result_2);

    // get ipcServer IDs for each node from the IpcServer's state
    if can_connect {
        let mut _node1_id = String::new();
        let mut node2_binding = String::new();

        alex.send(JsonProtocol::GetState.into())
            .expect("Failed sending RequestState on alex");
        let node_state_A = alex.wait(Box::new(one_is!(JsonProtocol::GetStateResult(_))))?;
        billy
            .send(JsonProtocol::GetState.into())
            .expect("Failed sending RequestState on billy");
        let node_state_B = billy.wait(Box::new(one_is!(JsonProtocol::GetStateResult(_))))?;

        one_let!(JsonProtocol::GetStateResult(state) = node_state_A {
            _node1_id = state.id
        });
        one_let!(JsonProtocol::GetStateResult(state) = node_state_B {
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
        let result_a = alex.wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))?;
        println!("got connect result A: {:?}", result_a);
        one_let!(JsonProtocol::PeerConnected(d) = result_a {
            assert_eq!(d.agent_id, BILLY_AGENT_ID);
        });
        let result_b = billy.wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))?;
        println!("got connect result B: {:?}", result_b);
        one_let!(JsonProtocol::PeerConnected(d) = result_b {
            assert_eq!(d.agent_id, ALEX_AGENT_ID);
        });
    }

    // Done
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
fn send_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: send_test()");
    setup_normal(alex, billy, can_connect)?;

    println!("setup done");

    // Send a message from alex to billy
    alex.send(
        JsonProtocol::SendMessage(MessageData {
            dna_address: example_dna_address(),
            to_agent_id: BILLY_AGENT_ID.to_string(),
            from_agent_id: ALEX_AGENT_ID.to_string(),
            msg_id: "yada".to_string(),
            data: json!("hello"),
        })
            .into(),
    )
        .expect("Failed sending SendMessage to billy");

    println!("SendMessage done");

    // Check if billy received it
    let res = billy.wait(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))))?;
    println!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::HandleSendMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!("\"hello\"".to_string(), msg.data.to_string());

    // Send a message back from billy to alex
    billy
        .send(
            JsonProtocol::HandleSendMessageResult(MessageData {
                dna_address: example_dna_address(),
                to_agent_id: ALEX_AGENT_ID.to_string(),
                from_agent_id: BILLY_AGENT_ID.to_string(),
                msg_id: "yada".to_string(),
                data: json!(format!("echo: {}", msg.data.to_string())),
            })
                .into(),
        )
        .expect("Failed sending HandleSendResult on billy");
    // Check if alex received it
    let res = alex.wait(Box::new(one_is!(JsonProtocol::SendMessageResult(_))))?;
    println!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::SendMessageResult(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!("\"echo: \\\"hello\\\"\"".to_string(), msg.data.to_string());

    // Done
    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn meta_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: meta_test()");
    setup_normal(alex, billy, can_connect)?;

    // Send data & metadata on same address
    confirm_published_data(alex, billy, &ENTRY_ADDRESS_1.into())?;
    confirm_published_metadata(alex, billy, &ENTRY_ADDRESS_1.into())?;

    // Again but now send metadata first
    confirm_published_metadata(alex, billy, &ENTRY_ADDRESS_2.into())?;
    confirm_published_data(alex, billy, &ENTRY_ADDRESS_2.into())?;

    // Again but 'wait' at the end
    // Alex publishs data & meta on the network
    alex.author_data(
        &example_dna_address(),
        ENTRY_ADDRESS_3.into(),
        json!("hello-3"),
        true,
    )?;
    alex.author_meta(
        &example_dna_address(),
        ENTRY_ADDRESS_3.into(),
        &META_ATTRIBUTE.to_string(),
        json!("hello-3-meta"),
        true,
    )?;

    // Billy sends GetDhtData message
    let fetch_data = FetchDhtData {
        request_id: "testGetEntry".to_string(),
        dna_address: example_dna_address(),
        requester_agent_id: BILLY_AGENT_ID.to_string(),
        data_address: ENTRY_ADDRESS_3.into(),
    };
    billy.send(JsonProtocol::FetchDhtData(fetch_data).into())?;

    // Billy sends HandleGetDhtDataResult message
    billy.reply_fetch_data(&fetch_data)?;

    // Billy sends GetDhtMeta message
    let fetch_meta = FetchDhtMetaData {
        request_id: "testGetMeta".to_string(),
        dna_address: example_dna_address(),
        requester_agent_id: BILLY_AGENT_ID.to_string(),
        data_address: ENTRY_ADDRESS_3.into(),
        attribute: META_ATTRIBUTE.to_string(),
    };
    billy.send(JsonProtocol::FetchDhtMeta(fetch_meta).into())?;
    // Alex sends HandleGetDhtMetaResult message
    alex.reply_fetch_meta(&fetch_meta)?;
    // billy should receive requested metadata
    let result = billy.wait(Box::new(one_is!(JsonProtocol::FetchDhtMetaResult(_))))?;
    println!("got GetDhtMetaResult: {:?}", result);
    // Done
    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn dht_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: dht_test()");
    setup_normal(alex, billy, can_connect)?;

    // Alex publish data on the network
    alex.author_data(
        &example_dna_address(),
             ENTRY_ADDRESS_1.into(),
            json!("hello"),
true,
    )?;

    // Check if both nodes are asked to store it
    let result_a = alex.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtData(_))))?;
    println!("got HandleStoreDhtData on node A: {:?}", result_a);
    let result_b = billy.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtData(_))))?;
    println!("got HandleStoreDhtData on node B: {:?}", result_b);
    assert!(billy.data_store.contains(address));

    // Billy asks for that data
    let fetch_data = FetchDhtData {
        request_id: "testGet".to_string(),
        dna_address: example_dna_address(),
        requester_agent_id: BILLY_AGENT_ID.to_string(),
        data_address: ENTRY_ADDRESS_1.into(),
    };
    billy.send(JsonProtocol::FetchDhtData(fetch_data).into())?;

    // Alex sends that data back to the network
    alex.reply_fetch_data(&fetch_data);

    // Billy should receive requested data
    let result = billy.wait(Box::new(one_is!(JsonProtocol::FetchDhtDataResult(_))))?;
    println!("got GetDhtDataResult: {:?}", result);
    // Done
    Ok(())
}
