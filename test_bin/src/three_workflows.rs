use constants::*;
use holochain_net::{
    connection::{net_connection::NetSend, NetResult},
    tweetlog::*,
};

use lib3h_protocol::{
    data_types::{ConnectData, EntryData},
    protocol_client::Lib3hClientProtocol,
    protocol_server::Lib3hServerProtocol,
    uri::Lib3hUri,
};

use holochain_persistence_api::cas::content::Address;
use p2p_node::test_node::TestNode;
use std::time::SystemTime;

/// Do normal setup: 'TrackDna' & 'Connect',
/// and check that we received 'Connected'
#[cfg_attr(tarpaulin, skip)]
// TODO consider that synchronization from peer connected is no longer reliable mechanism
// might want to mark as broken test
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
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::P2pReady)))
        .unwrap();
    println!("self connected result 1: {:?}", connect_result_1);
    // billy
    billy
        .track_dna(dna_address, true)
        .expect("Failed sending TrackDna on billy");
    let connect_result_2 = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::P2pReady)))
        .unwrap();
    println!("self connected result 2: {:?}", connect_result_2);
    // camille
    camille
        .track_dna(dna_address, true)
        .expect("Failed sending TrackDna on camille");
    let connect_result_3 = camille
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::P2pReady)))
        .unwrap();
    println!("self connected result 2: {:?}", connect_result_3);

    // get ipcServer IDs for each node from the IpcServer's state
    if can_connect {
        let mut _node1_id = String::new();
        let node2_binding = String::new();

        // Connect nodes between them
        println!("Connect Alex to Billy ({})", node2_binding);
        alex.send(Lib3hClientProtocol::Connect(ConnectData {
            request_id: "alex_to_billy_request_id".into(),
            peer_location: Lib3hUri(
                url::Url::parse(node2_binding.clone().as_str())
                    .expect("well formed node 2 uri (billy)"),
            ),
            network_id: "".into(),
        }))?;
        // Make sure Peers are connected
        let result_a = alex
            .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::Connected(_))))
            .unwrap();
        println!("got connect result A: {:?}", result_a);
        one_let!(Lib3hServerProtocol::Connected(d) = result_a {
            assert_eq!(d.request_id, "alex_to_billy_request_id");
            assert_eq!(d.uri.to_string(), node2_binding.clone());
        });
        let result_b = billy
            .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::Connected(_))))
            .unwrap();
        println!("got connect result B: {:?}", result_b);
        one_let!(Lib3hServerProtocol::Connected(d) = result_b {
           assert_eq!(d.request_id, "alex_to_billy_request_id");
           assert_eq!(d.uri.to_string(), node2_binding.clone());
        });

        // Connect nodes between them
        println!("Connect  Camille to Billy ({})", node2_binding);
        camille.send(Lib3hClientProtocol::Connect(ConnectData {
            request_id: "camille_to_billy_request_id".into(),
            peer_location: Lib3hUri(
                url::Url::parse(node2_binding.clone().as_str())
                    .expect("well formed billy (node2) ur"),
            ),
            network_id: "".into(),
        }))?;

        // Make sure Peers are connected

        let result_b = billy
            .wait_lib3h(Box::new(one_is_where!(
                Lib3hServerProtocol::Connected(data),
                { data.request_id == "camille_to_billy_request_id" }
            )))
            .unwrap();
        println!("got connect result on Billy: {:?}", result_b);

        // TODO BLOCKER used to be PeerConnected
        let result_c = camille
            .wait_lib3h(Box::new(one_is_where!(
                Lib3hServerProtocol::Connected(data),
                { data.request_id == "fixme" }
            )))
            .unwrap();
        println!("got connect result on Camille: {:?}", result_c);

        // TODO BLOCKER used to be PeerConnected
        let result_a = alex
            .wait_lib3h(Box::new(one_is_where!(
                Lib3hServerProtocol::Connected(data),
                { data.request_id == "fixme" }
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
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_1.clone()], false)?;
    // Alex: Look for the hold_list request received from network module and reply
    alex.reply_to_first_HandleGetAuthoringEntryList();
    // Might receive a HandleFetchEntry request from network module:
    // hackmode would want the data right away
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    assert!(has_received);
    // Maybe 2nd get for gossiping
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    log_d!("Alex has_received 2: {}", has_received);
    // Maybe 3nd get for gossiping
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    log_d!("Alex has_received 3: {}", has_received);
    // billy might receive HandleStoreEntryAspect
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    log_i!("Billy got res 1: {:?}", res);
    // Camille might receive HandleStoreEntryAspect
    let res = camille.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    log_i!("Camille got res 1: {:?}", res);

    // -- Billy authors -- //

    // Have billy author some other data
    billy.author_entry(&ENTRY_ADDRESS_2, vec![ASPECT_CONTENT_2.clone()], true)?;
    // Maybe recv fetch for gossiping
    let has_received = billy.wait_HandleFetchEntry_and_reply();
    log_d!("Billy has_received: {}", has_received);
    // Maybe recv fetch for gossiping
    let has_received = billy.wait_HandleFetchEntry_and_reply();
    log_d!("Billy has_received 2: {}", has_received);
    // billy might receive HandleStoreEntryAspect
    let res = alex.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    log_i!("Alex got res 2: {:?}", res);
    // Camille might receive HandleStoreEntryAspect
    let res = camille.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleStoreEntryAspect(_))),
        2000,
    );
    log_i!("Camille got res 2: {:?}", res);

    // -- Camille requests -- //

    // Camille requests 1st entry
    let query_entry = camille.request_entry(ENTRY_ADDRESS_1.clone());
    // #fullsync
    // Have Camille reply
    camille.reply_to_HandleQueryEntry(&query_entry).unwrap();

    // Camille should receive the data
    let req_id = query_entry.request_id.clone();
    let mut result = camille.find_recv_lib3h_msg(
        0,
        Box::new(one_is_where!(
            Lib3hServerProtocol::QueryEntryResult(entry_data),
            { entry_data.request_id == req_id }
        )),
    );
    if result.is_none() {
        result = camille.wait_lib3h(Box::new(one_is_where!(
            Lib3hServerProtocol::QueryEntryResult(entry_data),
            { entry_data.request_id == query_entry.request_id }
        )))
    }
    let json = result.unwrap();
    log_i!("got result 1: {:?}", json);
    let query_data = unwrap_to!(json => Lib3hServerProtocol::QueryEntryResult);
    let query_result: EntryData = bincode::deserialize(&query_data.query_result).unwrap();
    assert_eq!(query_data.entry_address, ENTRY_ADDRESS_1.clone());
    assert_eq!(query_result.entry_address.clone(), query_data.entry_address);
    assert_eq!(query_result.aspect_list.len(), 1);
    assert_eq!(
        *query_result.aspect_list[0].aspect,
        ASPECT_CONTENT_1.clone()
    );

    // Camille requests 2nd entry
    let query_data = camille.request_entry(ENTRY_ADDRESS_2.clone());
    // #fullsync
    // Have Camille reply
    camille.reply_to_HandleQueryEntry(&query_data).unwrap();

    // Camille should receive the data
    let req_id = query_data.request_id.clone();
    let mut result = camille.find_recv_lib3h_msg(
        0,
        Box::new(one_is_where!(
            Lib3hServerProtocol::QueryEntryResult(entry_data),
            { entry_data.request_id == req_id }
        )),
    );
    if result.is_none() {
        result = camille.wait_lib3h(Box::new(one_is_where!(
            Lib3hServerProtocol::QueryEntryResult(entry_data),
            { entry_data.request_id == query_data.request_id }
        )))
    }
    let json = result.unwrap();
    log_i!("got result 2: {:?}", json);
    let query_data = unwrap_to!(json => Lib3hServerProtocol::QueryEntryResult);
    let query_result: EntryData = bincode::deserialize(&query_data.query_result).unwrap();
    assert_eq!(query_data.entry_address, ENTRY_ADDRESS_2.clone());
    assert_eq!(query_result.entry_address.clone(), query_data.entry_address);
    assert_eq!(query_result.aspect_list.len(), 1);
    assert_eq!(
        *query_result.aspect_list[0].aspect,
        ASPECT_CONTENT_2.clone()
    );

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
    // #fullsync
    // wait for store entry request
    let result = camille.wait_lib3h_with_timeout(
        Box::new(one_is_where!(
            Lib3hServerProtocol::HandleStoreEntryAspect(entry_data),
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
    has_received = alex.wait_HandleQueryEntry_and_reply();
    if !has_received {
        has_received = billy.wait_HandleQueryEntry_and_reply();
        if !has_received {
            has_received = camille.wait_HandleQueryEntry_and_reply();
        }
    }
    log_i!("has_received 'HandleQueryEntry': {}", has_received);
    let time_after_handle_query = SystemTime::now();

    // Camille should receive the data
    log_i!("Waiting for fetch result...\n\n");

    let mut result = camille.find_recv_lib3h_msg(
        0,
        Box::new(one_is_where!(
            Lib3hServerProtocol::QueryEntryResult(entry_data),
            { entry_data.request_id == req_id }
        )),
    );
    if result.is_none() {
        result = camille.wait_lib3h_with_timeout(
            Box::new(one_is_where!(
                Lib3hServerProtocol::QueryEntryResult(entry_data),
                { entry_data.request_id == query_entry.request_id }
            )),
            10000,
        )
    }
    let json = result.unwrap();
    log_i!("got result 1: {:?}", json);
    let query_data = unwrap_to!(json => Lib3hServerProtocol::QueryEntryResult);
    let query_result: EntryData = bincode::deserialize(&query_data.query_result).unwrap();
    assert_eq!(query_data.entry_address, address_42.clone());
    assert_eq!(query_result.entry_address.clone(), query_data.entry_address);
    assert_eq!(query_result.aspect_list.len(), 1);
    assert_eq!(*query_result.aspect_list[0].aspect, entry_42.clone());

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
        time_after_handle_query
            .duration_since(time_after_authoring)
            .unwrap()
            .as_millis() as f32
            / 1000.0
    );
    println!(
        "  - Fetching   : {:?}s",
        time_end
            .duration_since(time_after_handle_query)
            .unwrap()
            .as_millis() as f32
            / 1000.0
    );
    // Done
    Ok(())
}
