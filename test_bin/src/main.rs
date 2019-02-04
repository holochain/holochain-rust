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
extern crate backtrace;

#[macro_use]
pub mod predicate;
pub mod basic_workflows;
pub mod constants;
pub mod p2p_node;
pub mod publish_hold_workflows;
pub mod three_workflows;

use constants::*;
use holochain_net_connection::NetResult;
use p2p_node::P2pNode;

type TwoNodesTestFn =
    fn(alex: &mut P2pNode, billy: &mut P2pNode, can_test_connect: bool) -> NetResult<()>;

type ThreeNodesTestFn = fn(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    camille: &mut P2pNode,
    can_test_connect: bool,
) -> NetResult<()>;

type MultiNodesTestFn = fn(nodes: &mut Vec<P2pNode>, can_test_connect: bool) -> NetResult<()>;

lazy_static! {
    // List of tests
    pub static ref TWO_NODES_BASIC_TEST_FNS: Vec<TwoNodesTestFn> = vec![
        basic_workflows::setup_two_nodes,
        basic_workflows::send_test,
        basic_workflows::dht_test,
        basic_workflows::meta_test,
    ];
    pub static ref TWO_NODES_LIST_TEST_FNS: Vec<TwoNodesTestFn> = vec![
        publish_hold_workflows::empty_publish_entry_list_test,
        publish_hold_workflows::publish_entry_list_test,
        publish_hold_workflows::publish_meta_list_test,
        publish_hold_workflows::hold_entry_list_test,
        publish_hold_workflows::hold_meta_list_test,
        publish_hold_workflows::double_publish_entry_list_test,
        publish_hold_workflows::double_publish_meta_list_test,
    ];
    pub static ref THREE_NODES_TEST_FNS: Vec<ThreeNodesTestFn> = vec![
        three_workflows::setup_three_nodes,
        three_workflows::hold_and_publish_test,
    ];
    pub static ref MULTI_NODES_TEST_FNS: Vec<MultiNodesTestFn> = vec![
    ];
}

fn print_three_nodes_test_name(print_str: &str, test_fn: ThreeNodesTestFn) {
    print_test_name(print_str, test_fn as *mut std::os::raw::c_void);
}

fn print_two_nodes_test_name(print_str: &str, test_fn: TwoNodesTestFn) {
    print_test_name(print_str, test_fn as *mut std::os::raw::c_void);
}

/// Print name of test function
fn print_test_name(print_str: &str, test_fn: *mut std::os::raw::c_void) {
    backtrace::resolve(test_fn, |symbol| {
        let mut full_name = symbol.name().unwrap().as_str().unwrap().to_string();
        let mut test_name = full_name.split_off("holochain_test_bin::".to_string().len());
        test_name.push_str("()");
        println!("{}{}", print_str, test_name);
    });
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

    // Merge two nodes tests
    let mut test_fns = TWO_NODES_BASIC_TEST_FNS.clone();
    test_fns.append(&mut TWO_NODES_LIST_TEST_FNS.clone());

    // Launch tests on each setup
    for test_fn in test_fns {
        launch_two_nodes_test_with_memory_network(test_fn).unwrap();
        launch_two_nodes_test_with_ipc_mock(
            &n3h_path,
            "test_bin/data/mock_ipc_network_config.json",
            test_fn,
        )
        .unwrap();
        launch_two_nodes_test(&n3h_path, "test_bin/data/network_config.json", test_fn).unwrap();
    }

    // Launch tests on each setup
    for test_fn in THREE_NODES_TEST_FNS.clone() {
        launch_three_nodes_test_with_memory_network(test_fn).unwrap();
        launch_three_nodes_test_with_ipc_mock(
            &n3h_path,
            "test_bin/data/mock_ipc_network_config.json",
            test_fn,
        )
        .unwrap();
        launch_three_nodes_test(&n3h_path, "test_bin/data/network_config.json", test_fn).unwrap();
    }

    // Wait a bit before closing
    for i in (0..4).rev() {
        println!("tick... {}", i);
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

//--------------------------------------------------------------------------------------------------
// TWO NODES LAUNCHERS
//--------------------------------------------------------------------------------------------------

// Do general test with config
#[cfg_attr(tarpaulin, skip)]
fn launch_two_nodes_test_with_memory_network(test_fn: TwoNodesTestFn) -> NetResult<()> {
    let mut alex =
        P2pNode::new_with_unique_memory_network(ALEX_AGENT_ID.to_string(), DNA_ADDRESS.clone());
    let mut billy = P2pNode::new_with_config(
        BILLY_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        &alex.config,
        None,
    );

    println!("");
    print_two_nodes_test_name("IN-MEMORY TWO NODE TEST", test_fn);
    println!("=======================");
    test_fn(&mut alex, &mut billy, false)?;
    println!("==================");
    print_two_nodes_test_name("IN-MEMORY TEST END: ", test_fn);
    println!("");
    // Kill nodes
    alex.stop();
    billy.stop();

    Ok(())
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
        ALEX_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );
    let mut billy = P2pNode::new_with_uri_ipc_network(
        BILLY_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        &alex.endpoint(),
    );

    println!("");
    print_two_nodes_test_name("IPC-MOCK TWO NODE TEST: ", test_fn);
    println!("======================");
    test_fn(&mut alex, &mut billy, false)?;
    println!("===================");
    print_two_nodes_test_name("IPC-MOCKED TEST END: ", test_fn);
    println!("");
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
        ALEX_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );
    let mut billy = P2pNode::new_with_spawn_ipc_network(
        BILLY_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );

    println!("");
    print_two_nodes_test_name("N3H TWO NODE TEST: ", test_fn);
    println!("=================");
    test_fn(&mut alex, &mut billy, true)?;
    println!("============");
    print_two_nodes_test_name("N3H TEST END: ", test_fn);
    println!("");
    // Kill nodes
    alex.stop();
    billy.stop();

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// THREE NODES LAUNCHERS
//--------------------------------------------------------------------------------------------------

// Do general test with config
#[cfg_attr(tarpaulin, skip)]
fn launch_three_nodes_test_with_memory_network(test_fn: ThreeNodesTestFn) -> NetResult<()> {
    // Create nodes
    let mut alex =
        P2pNode::new_with_unique_memory_network(ALEX_AGENT_ID.to_string(), DNA_ADDRESS.clone());
    let mut billy = P2pNode::new_with_config(
        BILLY_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        &alex.config,
        None,
    );
    let mut camille = P2pNode::new_with_config(
        CAMILLE_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        &alex.config,
        None,
    );

    // Launch test
    println!("");
    print_three_nodes_test_name("IN-MEMORY THREE NODE TEST: ", test_fn);
    println!("=========================");
    test_fn(&mut alex, &mut billy, &mut camille, false)?;
    println!("==================");
    print_three_nodes_test_name("IN-MEMORY TEST END: ", test_fn);
    println!("");

    // Kill nodes
    alex.stop();
    billy.stop();
    camille.stop();

    // Done
    Ok(())
}

// do general test with hackmode
#[cfg_attr(tarpaulin, skip)]
fn launch_three_nodes_test_with_ipc_mock(
    n3h_path: &str,
    config_filepath: &str,
    test_fn: ThreeNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut alex = P2pNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );
    let mut billy = P2pNode::new_with_uri_ipc_network(
        BILLY_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        &alex.endpoint(),
    );
    let mut camille = P2pNode::new_with_uri_ipc_network(
        CAMILLE_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        &alex.endpoint(),
    );

    println!("");
    print_three_nodes_test_name("IPC-MOCK THREE NODE TEST: ", test_fn);
    println!("========================");
    test_fn(&mut alex, &mut billy, &mut camille, false)?;
    println!("===================");
    print_three_nodes_test_name("IPC-MOCKED TEST END: ", test_fn);
    println!("");
    // Kill nodes
    alex.stop();
    billy.stop();
    camille.stop();

    Ok(())
}

// Do general test with config
#[cfg_attr(tarpaulin, skip)]
fn launch_three_nodes_test(
    n3h_path: &str,
    config_filepath: &str,
    test_fn: ThreeNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut alex = P2pNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );
    let mut billy = P2pNode::new_with_spawn_ipc_network(
        BILLY_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );
    let mut camille = P2pNode::new_with_spawn_ipc_network(
        CAMILLE_AGENT_ID.to_string(),
        DNA_ADDRESS.clone(),
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );

    println!("");
    print_three_nodes_test_name("N3H THREE NODE TEST: ", test_fn);
    println!("===================");
    test_fn(&mut alex, &mut billy, &mut camille, true)?;
    println!("============");
    print_three_nodes_test_name("N3H TEST END: ", test_fn);
    println!("");
    // Kill nodes
    alex.stop();
    billy.stop();
    camille.stop();

    // Done
    Ok(())
}

//--------------------------------------------------------------------------------------------------
// TEST MOD
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_two_nodes_basic_tests_with_in_memory_network() {
        for test_fn in TWO_NODES_BASIC_TEST_FNS.clone() {
            launch_two_nodes_test_with_memory_network(test_fn).unwrap();
        }
    }

    #[test]
    fn run_two_nodes_list_tests_with_in_memory_network() {
        for test_fn in TWO_NODES_LIST_TEST_FNS.clone() {
            launch_two_nodes_test_with_memory_network(test_fn).unwrap();
        }
    }

    #[test]
    fn run_three_nodes_tests_with_in_memory_network() {
        for test_fn in THREE_NODES_TEST_FNS.clone() {
            launch_three_nodes_test_with_memory_network(test_fn).unwrap();
        }
    }
}
