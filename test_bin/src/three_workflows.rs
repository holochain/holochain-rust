use constants::*;
use holochain_net::{
    connection::{
        json_protocol::{ConnectData, JsonProtocol, TrackDnaData},
        net_connection::NetSend,
        NetResult,
    },
    tweetlog::*,
};
use p2p_node::P2pNode;

/// Do normal setup: 'TrackDna' & 'Connect',
/// and check that we received 'PeerConnected'
#[cfg_attr(tarpaulin, skip)]
pub fn setup_three_nodes(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    camille: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Send TrackDna message on all nodes
    // alex
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
    // billy
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
    // camille
    camille
        .send(
            JsonProtocol::TrackDna(TrackDnaData {
                dna_address: DNA_ADDRESS.clone(),
                agent_id: CAMILLE_AGENT_ID.to_string(),
            })
            .into(),
        )
        .expect("Failed sending TrackDnaData on camille");
    let connect_result_3 = camille
        .wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))
        .unwrap();
    println!("self connected result 2: {:?}", connect_result_3);

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
        camille
            .send(JsonProtocol::GetState.into())
            .expect("Failed sending RequestState on camille");
        let camille_state = camille
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
        one_let!(
            JsonProtocol::GetStateResult(_state) = camille_state {
            // n/a
        }
        );

        // Connect nodes between them
        println!("Connect Alex to Billy ({})", node2_binding);
        alex.send(
            JsonProtocol::Connect(ConnectData {
                peer_address: node2_binding.clone().into(),
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

        println!("\n\n\n\n");

        // Connect nodes between them
        println!("Connect  Camille to Billy ({})", node2_binding);
        camille.send(
            JsonProtocol::Connect(ConnectData {
                peer_address: node2_binding.into(),
            })
            .into(),
        )?;

        // Make sure Peers are connected

        let result_b = billy
            .wait(Box::new(one_is_where!(
                JsonProtocol::PeerConnected(data),
                { data.agent_id == CAMILLE_AGENT_ID }
            )))
            .unwrap();
        println!("got connect result on Billy: {:?}", result_b);

        let result_c = camille
            .wait(Box::new(one_is_where!(
                JsonProtocol::PeerConnected(data),
                { data.agent_id == BILLY_AGENT_ID }
            )))
            .unwrap();
        println!("got connect result on Camille: {:?}", result_c);

        let result_a = alex
            .wait(Box::new(one_is_where!(
                JsonProtocol::PeerConnected(data),
                { data.agent_id == CAMILLE_AGENT_ID }
            )))
            .unwrap();
        println!("got connect result on Alex: {:?}", result_a);
    }

    // Make sure we received everything we needed from network module
    // TODO: Make a more robust function that waits for certain messages in msg log (with timeout that panics)
    let _msg_count = alex.listen(100);
    let _msg_count = billy.listen(100);
    let _msg_count = camille.listen(100);

    let mut time_ms: usize = 0;
    while !(alex.is_network_ready() && billy.is_network_ready() && camille.is_network_ready())
        && time_ms < 1000
    {
        let _msg_count = alex.listen(100);
        let _msg_count = billy.listen(100);
        let _msg_count = camille.listen(100);
        time_ms += 100;
    }

    log_i!("setup_three_nodes() COMPLETE \n\n\n");

    // Done
    Ok(())
}

/// Reply with some data in hold_list
#[cfg_attr(tarpaulin, skip)]
pub fn hold_and_publish_test(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    camille: &mut P2pNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    println!("Testing: hold_entry_list_test()");
    setup_three_nodes(alex, billy, camille, can_connect)?;

    // Have alex hold some data
    alex.author_entry(&ENTRY_ADDRESS_1, &ENTRY_CONTENT_1, false)?;
    // Alex: Look for the hold_list request received from network module and reply
    alex.reply_to_first_HandleGetPublishingEntryList();

    // Might receive a HandleFetchEntry request from network module:
    // hackmode would want the data right away
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);

    // Have billy author the same data
    billy.author_entry(&ENTRY_ADDRESS_2, &ENTRY_CONTENT_2, true)?;

    let _msg_count = camille.listen(3000);

    // Camille requests that entry
    let fetch_entry = camille.request_entry(ENTRY_ADDRESS_1.clone());
    // Alex or billy or Camille might receive HandleFetchEntry request as this moment
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    if !has_received {
        let has_received = billy.wait_HandleFetchEntry_and_reply();
        if !has_received {
            let _has_received = camille.wait_HandleFetchEntry_and_reply();
        }
    }

    // Camille should receive the data
    let req_id = fetch_entry.request_id.clone();
    let mut result = camille.find_recv_msg(0, Box::new(one_is_where!(JsonProtocol::FetchEntryResult(entry_data), {
            entry_data.request_id == req_id
        })));
    if result.is_none() {
        result = camille
            .wait(Box::new(one_is_where!(JsonProtocol::FetchEntryResult(entry_data), {
            entry_data.request_id == fetch_entry.request_id
        })))
    }

    let json = result.unwrap();
    log_i!("got result 1: {:?}", json);
    let entry_data = unwrap_to!(json => JsonProtocol::FetchEntryResult);
    assert_eq!(entry_data.entry_address, ENTRY_ADDRESS_1.clone());
    assert_eq!(entry_data.entry_content, ENTRY_CONTENT_1.clone());

    // Camille requests that entry
    let fetch_entry = camille.request_entry(ENTRY_ADDRESS_2.clone());
    // Alex or billy or Camille might receive HandleFetchEntry request as this moment
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    if !has_received {
        let has_received = billy.wait_HandleFetchEntry_and_reply();
        if !has_received {
            let _has_received = camille.wait_HandleFetchEntry_and_reply();
        }
    }

    // Camille should receive the data
    let req_id = fetch_entry.request_id.clone();
    let mut result = camille.find_recv_msg(0, Box::new(one_is_where!(JsonProtocol::FetchEntryResult(entry_data), {
            entry_data.request_id == req_id
        })));
    if result.is_none() {
        result = camille
            .wait(Box::new(one_is_where!(JsonProtocol::FetchEntryResult(entry_data), {
            entry_data.request_id == fetch_entry.request_id
        })))
    }
    let json = result.unwrap();
    log_i!("got result 2: {:?}", json);
    let entry_data = unwrap_to!(json => JsonProtocol::FetchEntryResult);
    assert_eq!(entry_data.entry_address, ENTRY_ADDRESS_2.clone());
    assert_eq!(entry_data.entry_content, ENTRY_CONTENT_2.clone());

    // Done
    Ok(())
}
