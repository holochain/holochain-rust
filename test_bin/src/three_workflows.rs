use constants::*;
use holochain_core_types::cas::content::Address;
use holochain_net::{
    connection::{
        json_protocol::{ConnectData, EntryData, JsonProtocol},
        net_connection::NetSend,
        NetResult,
    },
    tweetlog::*,
};
use p2p_node::test_node::TestNode;
use std::time::SystemTime;

/// Do normal setup: 'TrackDna' & 'Connect',
/// and check that we received 'PeerConnected'
#[cfg_attr(tarpaulin, skip)]
pub fn setup_three_nodes(
    alex: &mut TestNode,
    billy: &mut TestNode,
    camille: &mut TestNode,
    dna_address: &Address,
    can_connect: bool,
) -> NetResult<()> {
    // Send TrackDna message on all nodes
    // alex
    alex.track_dna(dna_address, true)
        .expect("Failed sending TrackDna on alex");
    let connect_result_1 = alex
        .wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))
        .unwrap();
    println!("self connected result 1: {:?}", connect_result_1);
    // billy
    billy
        .track_dna(dna_address, true)
        .expect("Failed sending TrackDna on billy");
    let connect_result_2 = billy
        .wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))
        .unwrap();
    println!("self connected result 2: {:?}", connect_result_2);
    // camille
    camille
        .track_dna(dna_address, true)
        .expect("Failed sending TrackDna on camille");
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
            assert_eq!(d.agent_id, *BILLY_AGENT_ID);
        });
        let result_b = billy
            .wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))
            .unwrap();
        println!("got connect result B: {:?}", result_b);
        one_let!(JsonProtocol::PeerConnected(d) = result_b {
            assert_eq!(d.agent_id, *ALEX_AGENT_ID);
        });

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
                { data.agent_id == *CAMILLE_AGENT_ID }
            )))
            .unwrap();
        println!("got connect result on Billy: {:?}", result_b);

        let result_c = camille
            .wait(Box::new(one_is_where!(
                JsonProtocol::PeerConnected(data),
                { data.agent_id == *BILLY_AGENT_ID }
            )))
            .unwrap();
        println!("got connect result on Camille: {:?}", result_c);

        let result_a = alex
            .wait(Box::new(one_is_where!(
                JsonProtocol::PeerConnected(data),
                { data.agent_id == *CAMILLE_AGENT_ID }
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
    alex: &mut TestNode,
    billy: &mut TestNode,
    camille: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    setup_three_nodes(alex, billy, camille, &DNA_ADDRESS_A, can_connect)?;

    // Have alex hold some data
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ENTRY_CONTENT_1.clone()], false)?;
    // Alex: Look for the hold_list request received from network module and reply
    alex.reply_to_first_HandleGetAuthoringEntryList();

    // Might receive a HandleFetchEntry request from network module:
    // hackmode would want the data right away
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);

    // Have billy author the same data
    billy.author_entry(&ENTRY_ADDRESS_2, vec![ENTRY_CONTENT_2.clone()], true)?;

    let _msg_count = camille.listen(3000);

    // Camille requests that entry
    let query_entry = camille.request_entry(ENTRY_ADDRESS_1.clone());
    // Alex or billy or Camille might receive HandleFetchEntry request as this moment
    let has_received = alex.wait_HandleQueryEntry_and_reply();
    if !has_received {
        let has_received = billy.wait_HandleQueryEntry_and_reply();
        if !has_received {
            let _has_received = camille.wait_HandleQueryEntry_and_reply();
        }
    }

    // Camille should receive the data
    let req_id = query_entry.request_id.clone();
    let mut result = camille.find_recv_msg(
        0,
        Box::new(one_is_where!(JsonProtocol::QueryEntryResult(entry_data), {
            entry_data.request_id == req_id
        })),
    );
    if result.is_none() {
        result = camille.wait(Box::new(one_is_where!(
            JsonProtocol::QueryEntryResult(entry_data),
            { entry_data.request_id == query_entry.request_id }
        )))
    }
    let json = result.unwrap();
    log_i!("got result 1: {:?}", json);
    let query_data = unwrap_to!(json => JsonProtocol::QueryEntryResult);
    let query_result: EntryData = bincode::deserialize(&query_data.query_result).unwrap();
    assert_eq!(query_data.entry_address, ENTRY_ADDRESS_1.clone());
    assert_eq!(query_result.entry_address.clone(), query_data.entry_address);
    assert_eq!(query_result.aspect_list.len(), 1);
    assert_eq!(query_result.aspect_list[0].aspect, ENTRY_CONTENT_1.clone());

    // Camille requests that entry
    let query_data = camille.request_entry(ENTRY_ADDRESS_2.clone());
    // Alex or billy or Camille might receive HandleFetchEntry request as this moment
    let has_received = alex.wait_HandleQueryEntry_and_reply();
    if !has_received {
        let has_received = billy.wait_HandleQueryEntry_and_reply();
        if !has_received {
            let _has_received = camille.wait_HandleQueryEntry_and_reply();
        }
    }

    // Camille should receive the data
    let req_id = query_data.request_id.clone();
    let mut result = camille.find_recv_msg(
        0,
        Box::new(one_is_where!(JsonProtocol::QueryEntryResult(entry_data), {
            entry_data.request_id == req_id
        })),
    );
    if result.is_none() {
        result = camille.wait(Box::new(one_is_where!(
            JsonProtocol::QueryEntryResult(entry_data),
            { entry_data.request_id == query_data.request_id }
        )))
    }
    let json = result.unwrap();
    log_i!("got result 2: {:?}", json);
    let query_data = unwrap_to!(json => JsonProtocol::QueryEntryResult);
    let query_result: EntryData = bincode::deserialize(&query_data.query_result).unwrap();
    assert_eq!(query_data.entry_address, ENTRY_ADDRESS_2.clone());
    assert_eq!(query_result.entry_address.clone(), query_data.entry_address);
    assert_eq!(query_result.aspect_list.len(), 1);
    assert_eq!(query_result.aspect_list[0].aspect, ENTRY_CONTENT_2.clone());

    // Done
    Ok(())
}

///
#[cfg_attr(tarpaulin, skip)]
pub fn publish_entry_stress_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    camille: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    let time_start = SystemTime::now();

    // Setup
    setup_three_nodes(alex, billy, camille, &DNA_ADDRESS_A, can_connect)?;

    let time_after_startup = SystemTime::now();

    // Have each node publish lots of entries
    for i in 0..100 {
        // Construct entry
        let (address, entry) = generate_entry(i);
        // select node & publish entry
        match i % 3 {
            0 => {
                alex.author_entry(&address, vec![entry], true)?;
            }
            1 => {
                billy.author_entry(&address, vec![entry], true)?;
            }
            2 => {
                camille.author_entry(&address, vec![entry], true)?;
            }
            _ => unreachable!(),
        };
    }
    let time_after_authoring = SystemTime::now();

    //
    let (address_42, entry_42) = generate_entry(91);
    let address_42_clone = address_42.clone();
    // #fulldht
    // wait for store entry request
    let result = camille.wait_with_timeout(
        Box::new(one_is_where!(
            JsonProtocol::HandleStoreEntryAspect(entry_data),
            { entry_data.entry_address == address_42_clone }
        )),
        10000,
    );
    assert!(result.is_some());

    log_i!("Requesting entry \n\n");
    // Camille requests that entry
    let query_entry = camille.request_entry(address_42.clone());
    let req_id = query_entry.request_id.clone();
    // Alex or Billy or Camille might receive HandleFetchEntry request as this moment
    #[allow(unused_assignments)]
    let mut has_received = false;
    has_received = alex.wait_HandleFetchEntry_and_reply();
    if !has_received {
        has_received = billy.wait_HandleFetchEntry_and_reply();
        if !has_received {
            has_received = camille.wait_HandleFetchEntry_and_reply();
        }
    }
    log_i!("has_received 'HandleFetchEntry': {}", has_received);
    let time_after_handle_fetch = SystemTime::now();

    // Camille should receive the data
    log_i!("Waiting for fetch result...\n\n");

    let mut result = camille.find_recv_msg(
        0,
        Box::new(one_is_where!(JsonProtocol::QueryEntryResult(entry_data), {
            entry_data.request_id == req_id
        })),
    );
    if result.is_none() {
        result = camille.wait_with_timeout(
            Box::new(one_is_where!(JsonProtocol::QueryEntryResult(entry_data), {
                entry_data.request_id == query_entry.request_id
            })),
            10000,
        )
    }
    let json = result.unwrap();
    log_i!("got result 1: {:?}", json);
    let query_data = unwrap_to!(json => JsonProtocol::QueryEntryResult);
    assert_eq!(query_data.entry_address, address_42.clone());
    assert_eq!(query_data.query_result, entry_42.clone());

    let time_end = SystemTime::now();

    // report
    println!(
        "Total : {}s",
        time_end.duration_since(time_start).unwrap().as_millis() as f32 / 1000.0
    );
    println!(
        "  - startup    : {:?}s",
        time_after_startup
            .duration_since(time_start)
            .unwrap()
            .as_millis() as f32
            / 1000.0
    );
    println!(
        "  - Authoring  : {:?}s",
        time_after_authoring
            .duration_since(time_after_startup)
            .unwrap()
            .as_millis() as f32
            / 1000.0
    );
    println!(
        "  - Handling   : {:?}s",
        time_after_handle_fetch
            .duration_since(time_after_authoring)
            .unwrap()
            .as_millis() as f32
            / 1000.0
    );
    println!(
        "  - Fetching   : {:?}s",
        time_end
            .duration_since(time_after_handle_fetch)
            .unwrap()
            .as_millis() as f32
            / 1000.0
    );
    // Done
    Ok(())
}
