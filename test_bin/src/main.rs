#![feature(try_from)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate failure;
extern crate holochain_core_types;
#[macro_use]
extern crate holochain_net;
extern crate lib3h_protocol;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate unwrap_to;
extern crate backtrace;
extern crate crossbeam_channel;
extern crate multihash;

#[macro_use]
pub mod predicate;
pub mod basic_workflows;
pub mod connection_workflows;
pub mod constants;
pub mod lib3h_workflows;
pub mod multidna_workflows;
pub mod p2p_node;
pub mod publish_hold_workflows;
pub mod three_workflows;

use constants::*;
use holochain_net::{connection::NetResult, tweetlog::*};
use p2p_node::test_node::TestNode;
use std::{collections::HashMap, fs::File};

pub(crate) type TwoNodesTestFn =
    fn(alex: &mut TestNode, billy: &mut TestNode, can_test_connect: bool) -> NetResult<()>;

type ThreeNodesTestFn = fn(
    alex: &mut TestNode,
    billy: &mut TestNode,
    camille: &mut TestNode,
    can_test_connect: bool,
) -> NetResult<()>;

type MultiNodesTestFn = fn(nodes: &mut Vec<TestNode>, can_test_connect: bool) -> NetResult<()>;

lazy_static! {
    // List of tests
    pub static ref TWO_NODES_BASIC_TEST_FNS: Vec<TwoNodesTestFn> = vec![
        basic_workflows::no_setup_test,
        basic_workflows::send_test,
        basic_workflows::untrack_alex_test,
        basic_workflows::untrack_billy_test,
        basic_workflows::retrack_test,
        basic_workflows::dht_test,
        basic_workflows::meta_test,
        basic_workflows::no_meta_test,
        basic_workflows::shutdown_test,
    ];
    pub static ref TWO_NODES_LIST_TEST_FNS: Vec<TwoNodesTestFn> = vec![
        publish_hold_workflows::empty_publish_entry_list_test,
        publish_hold_workflows::publish_entry_list_test,
        publish_hold_workflows::publish_meta_list_test,
        publish_hold_workflows::hold_meta_list_test,
        publish_hold_workflows::double_publish_entry_list_test,
        publish_hold_workflows::double_publish_meta_list_test,
        publish_hold_workflows::many_meta_test,
    ];
    pub static ref THREE_NODES_TEST_FNS: Vec<ThreeNodesTestFn> = vec![
        // thread 'tests::run_three_nodes_tests_with_in_memory_network' panicked at 'called `Option::unwrap()` on a `None` value', src/libcore/option.rs:345:21
        // three_workflows::hold_and_publish_test,
        three_workflows::publish_entry_stress_test,
        multidna_workflows::send_test,
        // Error occured in p2p network module, on receive: ErrorMessage { msg: "(memory-auto-puid-0-0) No sender channel found for DNA_A::billy" }
        // multidna_workflows::dht_test,
        multidna_workflows::meta_test,
    ];
    pub static ref MULTI_NODES_TEST_FNS: Vec<MultiNodesTestFn> = vec![
    ];
    pub static ref TWO_NODES_LIB3H_TEST_FNS: Vec<TwoNodesTestFn> = vec![
        lib3h_workflows::send_test,
    ];
}

#[cfg_attr(tarpaulin, skip)]
fn print_three_nodes_test_name(print_str: &str, test_fn: ThreeNodesTestFn) {
    print_test_name(print_str, test_fn as *mut std::os::raw::c_void);
}

#[cfg_attr(tarpaulin, skip)]
pub(crate) fn print_two_nodes_test_name(print_str: &str, test_fn: TwoNodesTestFn) {
    print_test_name(print_str, test_fn as *mut std::os::raw::c_void);
}

/// Print name of test function
#[cfg_attr(tarpaulin, skip)]
fn print_test_name(print_str: &str, test_fn: *mut std::os::raw::c_void) {
    backtrace::resolve(test_fn, |symbol| {
        let mut full_name = symbol.name().unwrap().as_str().unwrap().to_string();
        let mut test_name = full_name.split_off("holochain_test_bin::".to_string().len());
        test_name.push_str("()");
        log_i!("{}{}", print_str, test_name);
    });
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn load_config_file(filepath: &str) -> serde_json::Value {
    let config_file =
        File::open(filepath).expect("Failed to open filepath on Network Test config.");
    serde_json::from_reader(config_file).expect("file is not proper JSON")
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    // Check args ; get config filepath
    let args: Vec<String> = std::env::args().collect();
    let mut config_path = String::new();
    if args.len() == 2 {
        config_path = args[1].clone();
    }
    if config_path == "" {
        println!(
            "Usage: No config file supplied. Using default config: test_bin/data/test_config.json"
        );
        config_path = format!(
            "test_bin{0}data{0}test_config.json",
            std::path::MAIN_SEPARATOR
        )
        .to_string();
    }

    // Load config
    let config = load_config_file(&config_path);

    // Configure logger
    {
        let mut tweetlog = TWEETLOG.write().unwrap();
        let default_level = LogLevel::from(
            config["log"]["default"]
                .as_str()
                .unwrap()
                .chars()
                .next()
                .unwrap(),
        );
        tweetlog.set(default_level, None);
        // set level per tag
        let tag_map: HashMap<String, String> =
            serde_json::from_value(config["log"]["tags"].clone())
                .expect("missing/bad 'tags' config");
        for (tag, level_str) in tag_map {
            let level = LogLevel::from(level_str.as_str().chars().next().unwrap());
            tweetlog.set(level, Some(tag.clone()));
            tweetlog.listen_to_tag(&tag, Tweetlog::console);
        }
        tweetlog.listen(Tweetlog::console);
    }

    // Merge two nodes test suites
    let mut test_fns = Vec::new();
    if config["suites"]["BASIC_WORKFLOWS"].as_bool().unwrap() {
        test_fns.append(&mut TWO_NODES_BASIC_TEST_FNS.clone());
    }
    if config["suites"]["LIST_WORKFLOWS"].as_bool().unwrap() {
        test_fns.append(&mut TWO_NODES_LIST_TEST_FNS.clone());
    }
    // Launch tests on each setup
    for test_fn in test_fns {
        if config["modes"]["IN_MEMORY"].as_bool().unwrap() {
            launch_two_nodes_test_with_memory_network(test_fn).unwrap();
        }
        if config["modes"]["IPC_MOCK"].as_bool().unwrap() {
            launch_two_nodes_test_with_ipc_mock(
                "test_bin/data/mock_ipc_network_config.json",
                None,
                test_fn,
            )
            .unwrap();
        }
        if config["modes"]["N3H"].as_bool().unwrap() {
            launch_two_nodes_test(
                "test_bin/data/n3h_config.json",
                Some("test_bin/data/end_user_net_config.json".to_string()),
                test_fn,
            )
            .unwrap();
        }
    }

    // Launch LIB3H tests
    if config["modes"]["LIB3H"].as_bool().unwrap() {
        for test_fn in TWO_NODES_LIB3H_TEST_FNS.iter() {
            launch_two_nodes_test_with_lib3h(
                "test_bin/data/lib3h_config.json",
                Some("test_bin/data/end_user_net_config.json".to_string()),
                *test_fn,
            )
            .unwrap();
        }
    }
    // Launch THREE_WORKFLOWS tests on each setup
    if config["suites"]["THREE_WORKFLOWS"].as_bool().unwrap() {
        for test_fn in THREE_NODES_TEST_FNS.clone() {
            if config["modes"]["IN_MEMORY"].as_bool().unwrap() {
                launch_three_nodes_test_with_memory_network(test_fn).unwrap();
            }
            if config["modes"]["IPC_MOCK"].as_bool().unwrap() {
                launch_three_nodes_test_with_ipc_mock(
                    "test_bin/data/mock_ipc_network_config.json",
                    None,
                    test_fn,
                )
                .unwrap();
            }
            if config["modes"]["N3H"].as_bool().unwrap() {
                launch_three_nodes_test(
                    "test_bin/data/n3h_config.json",
                    Some("test_bin/data/end_user_net_config.json".to_string()),
                    test_fn,
                )
                .unwrap();
            }
        }
    }

    // CONNECTION_WORKFLOWS
    if config["suites"]["CONNECTION_WORKFLOWS"].as_bool().unwrap() {
        if config["modes"]["N3H"].as_bool().unwrap() {
            connection_workflows::two_nodes_disconnect_test(
                "test_bin/data/n3h_config.json",
                Some("test_bin/data/end_user_net_config.json".to_string()),
                basic_workflows::dht_test,
            )
            .unwrap();

            connection_workflows::three_nodes_disconnect_test(
                "test_bin/data/n3h_config.json",
                Some("test_bin/data/end_user_net_config.json".to_string()),
                three_workflows::hold_and_publish_test,
            )
            .unwrap();
        }
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
    let mut alex = TestNode::new_with_unique_memory_network(ALEX_AGENT_ID.to_string());
    let mut billy = TestNode::new_with_config(BILLY_AGENT_ID.to_string(), &alex.config, None);

    log_i!("");
    print_two_nodes_test_name("IN-MEMORY TWO NODE TEST: ", test_fn);
    log_i!("=======================");
    test_fn(&mut alex, &mut billy, false)?;
    log_i!("==================");
    print_two_nodes_test_name("IN-MEMORY TEST END: ", test_fn);
    // Kill nodes
    alex.stop();
    billy.stop();

    Ok(())
}

// do general test with hackmode
#[cfg_attr(tarpaulin, skip)]
fn launch_two_nodes_test_with_ipc_mock(
    config_filepath: &str,
    maybe_end_user_config_filepath: Option<String>,
    test_fn: TwoNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut alex = TestNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath,
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );
    let mut billy =
        TestNode::new_with_uri_ipc_network(BILLY_AGENT_ID.to_string(), &alex.endpoint());

    log_i!("");
    print_two_nodes_test_name("IPC-MOCK TWO NODE TEST: ", test_fn);
    log_i!("======================");
    test_fn(&mut alex, &mut billy, false)?;
    log_i!("===================");
    print_two_nodes_test_name("IPC-MOCKED TEST END: ", test_fn);
    // Kill nodes
    alex.stop();
    billy.stop();

    Ok(())
}

// Do general test with config
#[cfg_attr(tarpaulin, skip)]
fn launch_two_nodes_test(
    config_filepath: &str,
    maybe_end_user_config_filepath: Option<String>,
    test_fn: TwoNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut alex = TestNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );
    let mut billy = TestNode::new_with_spawn_ipc_network(
        BILLY_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath,
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );

    log_i!("");
    print_two_nodes_test_name("N3H TWO NODE TEST: ", test_fn);
    log_i!("=================");
    test_fn(&mut alex, &mut billy, true)?;
    log_i!("============");
    print_two_nodes_test_name("N3H TEST END: ", test_fn);
    // Kill nodes
    alex.stop();
    billy.stop();

    Ok(())
}

/// Do test with default lib3hcd ..
#[cfg_attr(tarpaulin, skip)]
fn launch_two_nodes_test_with_lib3h(
    config_filepath: &str,
    maybe_end_user_config_filepath: Option<String>,
    test_fn: TwoNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut alex = TestNode::new_with_lib3h(
        ALEX_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );
    let mut billy = TestNode::new_with_lib3h(
        BILLY_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );
    log_i!("");
    print_two_nodes_test_name("LIB3H TWO NODE TEST: ", test_fn);
    log_i!("=======================");
    test_fn(&mut alex, &mut billy, false)?;
    log_i!("==================");
    print_two_nodes_test_name("LIB3H TEST END: ", test_fn);
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
    let mut alex = TestNode::new_with_unique_memory_network(ALEX_AGENT_ID.to_string());
    let mut billy = TestNode::new_with_config(BILLY_AGENT_ID.to_string(), &alex.config, None);
    let mut camille = TestNode::new_with_config(CAMILLE_AGENT_ID.to_string(), &alex.config, None);

    // Launch test
    log_i!("");
    print_three_nodes_test_name("IN-MEMORY THREE NODE TEST: ", test_fn);
    log_i!("=========================");
    test_fn(&mut alex, &mut billy, &mut camille, false)?;
    log_i!("==================");
    print_three_nodes_test_name("IN-MEMORY TEST END: ", test_fn);

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
    config_filepath: &str,
    maybe_end_user_config_filepath: Option<String>,
    test_fn: ThreeNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut alex = TestNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath,
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );
    let mut billy =
        TestNode::new_with_uri_ipc_network(BILLY_AGENT_ID.to_string(), &alex.endpoint());
    let mut camille =
        TestNode::new_with_uri_ipc_network(CAMILLE_AGENT_ID.to_string(), &alex.endpoint());

    log_i!("");
    print_three_nodes_test_name("IPC-MOCK THREE NODE TEST: ", test_fn);
    log_i!("========================");
    test_fn(&mut alex, &mut billy, &mut camille, false)?;
    log_i!("===================");
    print_three_nodes_test_name("IPC-MOCKED TEST END: ", test_fn);
    // Kill nodes
    alex.stop();
    billy.stop();
    camille.stop();

    Ok(())
}

// Do general test with config
#[cfg_attr(tarpaulin, skip)]
fn launch_three_nodes_test(
    config_filepath: &str,
    maybe_end_user_config_filepath: Option<String>,
    test_fn: ThreeNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut alex = TestNode::new_with_spawn_ipc_network(
        ALEX_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );
    let mut billy = TestNode::new_with_spawn_ipc_network(
        BILLY_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );
    let mut camille = TestNode::new_with_spawn_ipc_network(
        CAMILLE_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath,
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );

    log_i!("");
    print_three_nodes_test_name("N3H THREE NODE TEST: ", test_fn);
    log_i!("===================");
    test_fn(&mut alex, &mut billy, &mut camille, true)?;
    log_i!("============");
    print_three_nodes_test_name("N3H TEST END: ", test_fn);
    // Kill nodes
    alex.stop();
    billy.stop();
    camille.stop();

    // Done
    Ok(())
}

// Do general test with config
#[cfg_attr(tarpaulin, skip)]
#[allow(dead_code)]
fn launch_three_nodes_test_with_lib3h(
    config_filepath: &str,
    maybe_end_user_config_filepath: Option<String>,
    test_fn: ThreeNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut alex = TestNode::new_with_lib3h(
        ALEX_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );
    let mut billy = TestNode::new_with_lib3h(
        BILLY_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath.clone(),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );
    let mut camille = TestNode::new_with_lib3h(
        CAMILLE_AGENT_ID.to_string(),
        Some(config_filepath),
        maybe_end_user_config_filepath,
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
        None,
    );

    log_i!("");
    print_three_nodes_test_name("LIB3H THREE NODE TEST: ", test_fn);
    log_i!("===================");
    test_fn(&mut alex, &mut billy, &mut camille, true)?;
    log_i!("============");
    print_three_nodes_test_name("LIB3H TEST END: ", test_fn);
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
