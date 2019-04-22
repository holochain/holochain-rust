use crate::{
    basic_workflows, print_three_nodes_test_name, print_two_nodes_test_name, ThreeNodesTestFn,
    TwoNodesTestFn,
};
use constants::*;
use holochain_net::{
    connection::{
        json_protocol::{ConnectData, JsonProtocol},
        net_connection::NetSend,
        NetResult,
    },
    tweetlog::*,
};
use p2p_node::P2pNode;

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
    let mut alex = P2pNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        Some(alex_dir_path.clone()),
    );
    let mut billy = P2pNode::new_with_spawn_ipc_network(
        BILLY_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
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
    alex = P2pNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        // TODO test bootstrap with billy's endpoint
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        Some(alex_dir_path),
    );
    basic_workflows::setup_one_node(&mut alex, &mut billy, true)?;

    let alex_binding_2 = alex.p2p_binding.clone();
    log_i!("#### alex_binding_2: {}", alex_binding_2);

    // Connect nodes between them
    log_i!("connect: billy.p2p_binding = {}", billy.p2p_binding);
    alex.send(
        JsonProtocol::Connect(ConnectData {
            peer_address: billy.p2p_binding.clone().into(),
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
    let mut alex = P2pNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        Some(alex_dir_path.clone()),
    );
    // Create billy & temp dir
    let billy_dir = tempfile::tempdir().expect("Failed to created a temp directory.");
    let billy_dir_path = billy_dir.path().to_string_lossy().to_string();
    let mut billy = P2pNode::new_with_spawn_ipc_network(
        BILLY_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        Some(billy_dir_path),
    );
    // Create camille & temp dir
    let camille_dir = tempfile::tempdir().expect("Failed to created a temp directory.");
    let camille_dir_path = camille_dir.path().to_string_lossy().to_string();
    let mut camille = P2pNode::new_with_spawn_ipc_network(
        CAMILLE_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
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
    camille.author_entry(&ENTRY_ADDRESS_3, &ENTRY_CONTENT_3, true)?;
    let count = billy.listen(1000);
    log_i!("#### billy got alex camille authoring: {}\n\n\n\n", count);

    // re-enable alex
    alex = P2pNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec![billy.p2p_binding.clone()],
        Some(alex_dir_path),
    );
    alex.track_dna().expect("Failed sending TrackDna on alex");
    log_i!("#### alex reborn ({})", alex.p2p_binding.clone());

    let count = alex.listen(500);
    log_i!("#### alex got reconnecting: {}\n\n", count);

    // Make sure Peers are connected
    let fetch_entry = alex.request_entry(ENTRY_ADDRESS_3.clone());
    // Alex or billy or Camille might receive HandleFetchEntry request as this moment
    let has_received = alex.wait_HandleFetchEntry_and_reply();
    if !has_received {
        let has_received = billy.wait_HandleFetchEntry_and_reply();
        if !has_received {
            let _has_received = camille.wait_HandleFetchEntry_and_reply();
        }
    }

    // Alex should receive the data
    let req_id = fetch_entry.request_id.clone();
    let mut result = alex.find_recv_msg(
        0,
        Box::new(one_is_where!(JsonProtocol::FetchEntryResult(entry_data), {
            entry_data.request_id == req_id
        })),
    );
    if result.is_none() {
        result = alex.wait(Box::new(one_is_where!(
            JsonProtocol::FetchEntryResult(entry_data),
            { entry_data.request_id == fetch_entry.request_id }
        )))
    }
    let json = result.unwrap();
    log_i!("got result 3: {:?}", json);
    let entry_data = unwrap_to!(json => JsonProtocol::FetchEntryResult);
    assert_eq!(entry_data.entry_address, ENTRY_ADDRESS_3.clone());
    assert_eq!(entry_data.entry_content, ENTRY_CONTENT_3.clone());

    log_i!("============");
    print_three_nodes_test_name("N3H three_nodes_disconnect_test END: ", test_fn);
    // Kill nodes
    alex.stop();
    billy.stop();
    camille.stop();

    Ok(())
}
