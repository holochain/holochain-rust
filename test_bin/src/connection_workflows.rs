use crate::{
    basic_workflows, print_three_nodes_test_name, print_two_nodes_test_name, ThreeNodesTestFn,
    TwoNodesTestFn,
};
use constants::*;
use holochain_net::{
    connection::{net_connection::NetSend, NetResult},
    tweetlog::*,
};

use lib3h_protocol::{
    data_types::{ConnectData, EntryData},
    protocol_client::Lib3hClientProtocol,
    protocol_server::Lib3hServerProtocol,
};

use p2p_node::test_node::TestNode;

// Disconnect & reconnect a Node in a two nodes scenario
#[cfg_attr(tarpaulin, skip)]
pub(crate) fn two_nodes_disconnect_test(
    config_filepath: &str,
    maybe_end_user_config_filepath: Option<String>,
    test_fn: TwoNodesTestFn,
) -> NetResult<()> {
    // Create alex temp dir
    let alex_dir = tempfile::tempdir().expect("Failed to created a temp directory.");
    let alex_dir_path = alex_dir.path().to_string_lossy().to_string();
    // Create two nodes
    let mut alex = TestNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        Some(alex_dir_path.clone()),
    );
    let mut billy = TestNode::new_with_spawn_ipc_network(
        BILLY_AGENT_ID.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );

    log_i!("");
    print_two_nodes_test_name("N3H two_nodes_disconnect_test: ", test_fn);
    log_i!("=================");
    test_fn(&mut alex, &mut billy, true)?;
    let _ = billy.listen(200);
    // kill alex
    let alex_binding = alex.p2p_binding.clone();
    log_i!("#### alex_binding: {}", alex_binding);
    alex.stop();
    // check if billy is still alive or screaming
    let count = billy.listen(2000);
    log_i!("#### billy got: {}\n\n\n\n", count);

    // re-enable alex
    alex = TestNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        // TODO test bootstrap with billy's endpoint
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        Some(alex_dir_path),
    );
    basic_workflows::setup_one_node(&mut alex, &mut billy, &DNA_ADDRESS_A, true)?;

    let alex_binding_2 = alex.p2p_binding.clone();
    log_i!("#### alex_binding_2: {}", alex_binding_2);

    // Connect nodes between them
    log_i!("connect: billy.p2p_binding = {}", billy.p2p_binding);
    alex.send(
        // TODO BLOCKER determine correct values
        Lib3hClientProtocol::Connect(ConnectData {
            request_id: "alex_connect_billy_request_id".into(),
            peer_uri: url::Url::parse(billy.p2p_binding.clone().as_str())
                .expect("well-formed billy p2p binding uri"),
            network_id: "alex_connect_billy_network_id".into(),
        }),
    )?;
    // Make sure Peers are connected
    let result_a = alex
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::Connected(_))))
        .unwrap();
    log_i!("got connect result A: {:?}", result_a);
    one_let!(Lib3hServerProtocol::Connected(d) = result_a {
        assert_eq!(d.request_id, "alex_connect_billy_request_id");
        assert_eq!(d.uri.to_string(), billy.p2p_binding);
    });
    let result_b = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::Connected(_))))
        .unwrap();
    log_i!("got connect result B: {:?}", result_b);
    one_let!(Lib3hServerProtocol::Connected(d) = result_b {
        assert_eq!(d.request_id, "alex_connect_billy_request_id");
        assert_eq!(d.uri.to_string(), alex.p2p_binding);
    });

    // see what alex is receiving
    let count = alex.listen(2000);
    log_i!("#### alex got: {}", count);

    log_i!("============");
    print_two_nodes_test_name("N3H two_nodes_disconnect_test END: ", test_fn);
    // Kill nodes
    alex.stop();
    billy.stop();

    Ok(())
}

// Disconnect & reconnect a Node in a three nodes scenario
#[cfg_attr(tarpaulin, skip)]
pub(crate) fn three_nodes_disconnect_test(
    config_filepath: &str,
    maybe_end_user_config_filepath: Option<String>,
    test_fn: ThreeNodesTestFn,
) -> NetResult<()> {
    log_i!("");
    print_three_nodes_test_name("N3H three_nodes_disconnect_test: ", test_fn);
    log_i!("=================");
    // Create alex & temp dir
    let alex_dir = tempfile::tempdir().expect("Failed to created a temp directory.");
    let alex_dir_path = alex_dir.path().to_string_lossy().to_string();
    let mut alex = TestNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        Some(alex_dir_path.clone()),
    );
    // Create billy & temp dir
    let billy_dir = tempfile::tempdir().expect("Failed to created a temp directory.");
    let billy_dir_path = billy_dir.path().to_string_lossy().to_string();
    let mut billy = TestNode::new_with_spawn_ipc_network(
        BILLY_AGENT_ID.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        Some(billy_dir_path),
    );
    // Create camille & temp dir
    let camille_dir = tempfile::tempdir().expect("Failed to created a temp directory.");
    let camille_dir_path = camille_dir.path().to_string_lossy().to_string();
    let mut camille = TestNode::new_with_spawn_ipc_network(
        CAMILLE_AGENT_ID.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        Some(camille_dir_path),
    );

    // Perform some other test function
    test_fn(&mut alex, &mut billy, &mut camille, true)?;
    let _ = billy.listen(200);
    // kill alex
    log_i!("#### Stopping alex ({})", alex.p2p_binding);
    alex.stop();
    // check if billy is still alive or screaming
    let count = billy.listen(1000);
    log_i!("#### billy got after alex shutdown: {}\n\n\n\n", count);

    // Have Camille author something while alex is offline
    camille.author_entry(&ENTRY_ADDRESS_3, vec![ASPECT_CONTENT_3.clone()], true)?;
    let count = billy.listen(1000);
    log_i!("#### billy got alex camille authoring: {}\n\n\n\n", count);

    // re-enable alex
    alex = TestNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec![billy.p2p_binding.clone()],
        Some(alex_dir_path),
    );
    alex.track_dna(&DNA_ADDRESS_A, true)
        .expect("Failed sending TrackDna on alex");
    log_i!("#### alex reborn ({})", alex.p2p_binding.clone());

    let count = alex.listen(3000);
    log_i!("#### alex got reconnecting: {}\n\n", count);

    // Make sure Peers are connected
    let query_entry = alex.request_entry(ENTRY_ADDRESS_3.clone());
    // Alex or billy or Camille might receive HandleFetchEntry request as this moment
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    if !has_received {
        let has_received = billy.wait_HandleFetchEntry_and_reply();
        if !has_received {
            let _has_received = camille.wait_HandleFetchEntry_and_reply();
        }
    }

    // Alex should receive the data
    let req_id = query_entry.request_id.clone();
    let mut result = alex.find_recv_lib3h_msg(
        0,
        Box::new(one_is_where!(
            Lib3hServerProtocol::QueryEntryResult(entry_data),
            { entry_data.request_id == req_id }
        )),
    );
    if result.is_none() {
        result = alex.wait_lib3h(Box::new(one_is_where!(
            Lib3hServerProtocol::QueryEntryResult(entry_data),
            { entry_data.request_id == query_entry.request_id }
        )))
    }
    let json = result.unwrap();
    log_i!("got result 3: {:?}", json);
    let query_data = unwrap_to!(json => Lib3hServerProtocol::QueryEntryResult);
    let query_result: EntryData = bincode::deserialize(&query_data.query_result).unwrap();
    assert_eq!(query_data.entry_address, ENTRY_ADDRESS_3.clone());
    assert_eq!(query_result.entry_address.clone(), query_data.entry_address);
    assert_eq!(query_result.aspect_list.len(), 1);
    assert_eq!(query_result.aspect_list[0].aspect, ASPECT_CONTENT_3.clone());

    log_i!("============");
    print_three_nodes_test_name("N3H three_nodes_disconnect_test END: ", test_fn);
    // Kill nodes
    alex.stop();
    billy.stop();
    camille.stop();

    Ok(())
}
