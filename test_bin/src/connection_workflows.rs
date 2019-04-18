use crate::{basic_workflows, print_two_nodes_test_name, TwoNodesTestFn};
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

// Do general test with config
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
    let count = billy.listen(5000);
    log_i!("#### billy got: {}\n\n\n\n", count);

    // re-enable alex
    alex = P2pNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        // TODO test bootstrap with billy's endpoint
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        // None,
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
