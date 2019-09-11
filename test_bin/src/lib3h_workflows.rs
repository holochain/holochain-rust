use constants::*;
use holochain_net::{
    connection::{net_connection::NetSend, NetResult},
    tweetlog::TWEETLOG,
};
use holochain_persistence_api::cas::content::Address;
use lib3h_protocol::{protocol_client::Lib3hClientProtocol, protocol_server::Lib3hServerProtocol};
use p2p_node::test_node::TestNode;
use std::str;
use url::Url;

/// Do normal setup: 'TrackDna' & 'Connect',
/// and check that we received 'PeerConnected'
#[cfg_attr(tarpaulin, skip)]
pub fn setup_two_lib3h_nodes(
    alex: &mut TestNode,
    billy: &mut TestNode,
    dna_address: &Address,
    can_connect: bool,
) -> NetResult<()> {
    // Make sure network module is ready
    let mut time_ms: usize = 0;
    while !(alex.is_network_ready() && billy.is_network_ready()) && time_ms < 1000 {
        let _msg_count = alex.listen(100);
        let _msg_count = billy.listen(100);
        time_ms += 100;
    }
    assert!(alex.is_network_ready());
    assert!(billy.is_network_ready());

    // Send TrackDna message on both nodes
    alex.track_dna(dna_address, true)
        .expect("Failed sending TrackDna on alex");
    // Check if PeerConnected is received
    let connect_result_1 = alex
        .wait_lib3h(Box::new(one_is_lib3h!(
            Lib3hServerProtocol::SuccessResult(_)
        )))
        .unwrap();
    log_i!("self connected result 1: {:?}", connect_result_1);
    billy
        .track_dna(dna_address, true)
        .expect("Failed sending TrackDna on billy");
    let connect_result_2 = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::SuccessResult(_))))
        .unwrap();
    log_i!("self connected result 2: {:?}", connect_result_2);

    if can_connect {
        let mut _node1_id = String::new();
        let node2_binding = String::new();

        // Connect nodes between them
        log_i!("connect: node2_binding = {}", node2_binding);
        alex.send(Lib3hClientProtocol::Connect(
            lib3h_protocol::data_types::ConnectData {
                request_id: "connect_req_1".into(),
                peer_uri: Url::parse(billy.p2p_binding.clone().as_str())
                    .expect("well formed peer uri"),
                network_id: "FIXME".into(),
            },
        ))?;

        // Make sure Peers are connected
        let result_a = alex
            .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::Connected(_))))
            .unwrap();
        log_i!("got connect result A: {:?}", result_a);
        let result_b = billy
            .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::Connected(_))))
            .unwrap();
        log_i!("got connect result B: {:?}", result_b);
    }

    // Make sure we received everything we needed from network module
    // TODO: Make a more robust function that waits for certain messages in msg log (with timeout that panics)
    let _msg_count = alex.listen(100);
    let _msg_count = billy.listen(100);

    log_i!("setup_two_lib3h_nodes() COMPLETE \n\n\n");

    // Done
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
pub fn send_test(alex: &mut TestNode, billy: &mut TestNode, can_connect: bool) -> NetResult<()> {
    // Setup
    setup_two_lib3h_nodes(alex, billy, &DNA_ADDRESS_A, can_connect)?;

    // Send a message from alex to billy
    alex.send_direct_message(&*BILLY_AGENT_ID, ASPECT_CONTENT_1.clone());

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
    assert_eq!(ASPECT_CONTENT_1.clone(), msg.content.as_slice(),);

    // Send a message back from billy to alex
    billy.send_response_lib3h(
        msg.clone(),
        format!("echo: {}", str::from_utf8(msg.content.as_slice()).unwrap()).into_bytes(),
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
        str::from_utf8(msg.content.as_slice()).unwrap()
    );

    // Done
    Ok(())
}
