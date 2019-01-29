#![feature(try_from)]

extern crate holochain_core_types;
extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;
extern crate tempfile;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate unwrap_to;

#[macro_use]
pub mod predicate;
pub mod basic_workflows;
pub mod constants;
pub mod p2p_node;
pub mod publish_hold_workflows;

use holochain_net_connection::NetResult;
use p2p_node::P2pNode;

type TwoNodesTestFn =
    fn(node1: &mut P2pNode, node2: &mut P2pNode, can_test_connect: bool) -> NetResult<()>;

lazy_static! {
    // List of tests
    pub static ref TEST_FNS: Vec<TwoNodesTestFn> = vec![
        basic_workflows::setup_normal,
        basic_workflows::send_test,
        basic_workflows::dht_test,
        basic_workflows::meta_test,
        publish_hold_workflows::empty_publish_data_list_test,
        publish_hold_workflows::publish_list_test,
        publish_hold_workflows::publish_meta_list_test,
        publish_hold_workflows::hold_list_test,
        publish_hold_workflows::hold_meta_list_test,
    ];
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn usage() {
    println!("Usage: holochain_test_bin <path_to_n3h>");
    std::process::exit(1);
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    // Check args
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        usage();
    }
    let n3h_path = args[1].clone();
    if n3h_path == "" {
        usage();
    }

    // Launch tests on each setup
    for test_fn in TEST_FNS.clone() {
        launch_two_nodes_test_with_memory_network(test_fn).unwrap();
        launch_two_nodes_test_with_ipc_mock(
            &n3h_path,
            "test_bin/data/mock_ipc_network_config.json",
            test_fn,
        )
        .unwrap();
        launch_two_nodes_test(&n3h_path, "test_bin/data/network_config.json", test_fn).unwrap();
    }

    // Wait a bit before closing
    for i in (0..4).rev() {
        println!("tick... {}", i);
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

// do general test with hackmode
#[cfg_attr(tarpaulin, skip)]
fn launch_two_nodes_test_with_ipc_mock(
    n3h_path: &str,
    config_filepath: &str,
    test_fn: TwoNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut alex = P2pNode::new_with_spawn_ipc_network(
        "alex".to_string(),
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );
    let mut billy = P2pNode::new_with_uri_ipc_network("billy".to_string(), &alex.endpoint());

    println!("IPC-MOCK TWO NODE TEST");
    println!("======================");
    test_fn(&mut alex, &mut billy, false)?;
    println!("===================");
    println!("IPC-MOCKED TEST END\n");
    // Kill nodes
    alex.stop();
    billy.stop();

    Ok(())
}

// Do general test with config
#[cfg_attr(tarpaulin, skip)]
fn launch_two_nodes_test_with_memory_network(test_fn: TwoNodesTestFn) -> NetResult<()> {
    let mut alex = P2pNode::new_with_unique_memory_network("alex".to_string());
    let mut billy = P2pNode::new_with_config("billy".to_string(), &alex.config, None);

    println!("IN-MEMORY TWO NODE TEST");
    println!("=======================");
    test_fn(&mut alex, &mut billy, false)?;
    println!("==================");
    println!("IN-MEMORY TEST END\n");
    // Kill nodes
    alex.stop();
    billy.stop();

    Ok(())
}

// Do general test with config
#[cfg_attr(tarpaulin, skip)]
fn launch_two_nodes_test(
    n3h_path: &str,
    config_filepath: &str,
    test_fn: TwoNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut alex = P2pNode::new_with_spawn_ipc_network(
        "alex".to_string(),
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );
    let mut billy = P2pNode::new_with_spawn_ipc_network(
        "billy".to_string(),
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );

    println!("NORMAL TWO NODE TEST");
    println!("====================");
    test_fn(&mut alex, &mut billy, true)?;
    println!("===============");
    println!("NORMAL TEST END\n");
    // Kill nodes
    alex.stop();
    billy.stop();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_tests_with_in_memory_network() {
        for test_fn in TEST_FNS.clone() {
            launch_two_nodes_test_with_memory_network(test_fn).unwrap();
        }
    }
}