#![feature(try_from)]
#![allow(non_snake_case)]

extern crate holochain_core_types;
extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;
extern crate tempfile;

pub mod p2p_node;

use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    net_connection::NetSend,
    protocol_wrapper::{
        ConnectData, DhtData, DhtMetaData, GetDhtData, GetDhtMetaData, MessageData,
        ProtocolWrapper, TrackAppData,
    },
    NetResult,
};

use p2p_node::P2pNode;

// CONSTS
static AGENT_ID_1: &'static str = "DUMMY_AGENT_1";
static AGENT_ID_2: &'static str = "DUMMY_AGENT_2";
static ENTRY_ADDRESS_1: &'static str = "dummy_addr_1";
static ENTRY_ADDRESS_2: &'static str = "dummy_addr_2";
static ENTRY_ADDRESS_3: &'static str = "dummy_addr_3";
static DNA_ADDRESS: &'static str = "DUMMY_DNA_ADDRESS";
static META_ATTRIBUTE: &'static str = "link__yay";

fn example_dna_address() -> Address {
    DNA_ADDRESS.into()
}

type TwoNodesTestFn =
fn(node1: &mut P2pNode, node2: &mut P2pNode, can_test_connect: bool) -> NetResult<()>;


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

    // List of tests
    let test_fns: Vec<TwoNodesTestFn> = vec![
        //meta_test,
        //dht_test,
        generic_message_test,
    ];

    // Launch test for each setup
    for test_fn in test_fns.clone() {
        // launch_two_nodes_rust_mock_test(test_fn).unwrap();
        //launch_two_nodes_test_with_ipc_mock(&n3h_path, "test_bin/data/mock_network_config.json", test_fn).unwrap();
        launch_two_nodes_test(&n3h_path, "test_bin/data/network_config.json", test_fn).unwrap();
    }

    // Wait a bit before closing
    for i in (0..4).rev() {
        println!("tick... {}", i);
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}


// MACROS
macro_rules! one_let {
    ($p:pat = $enum:ident $code:tt) => {
        if let $p = $enum {
            $code
        } else {
            unimplemented!();
        }
    };
}

/// Macro for transforming a type check into a predicate
macro_rules! one_is {
    ($p:pat) => {
        |d| {
            if let $p = d {
                return true;
            }
            return false;
        }
    };
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn usage() {
    println!("Usage: test_bin_ipc <path_to_n3h>");
    std::process::exit(1);
}

// do general test with hackmode
fn launch_two_nodes_test_with_ipc_mock(
    n3h_path: &str,
    config_filepath: &str,
    test_fn: TwoNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut node1 = P2pNode::new_ipc_spawn(
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );
    let mut node2 = P2pNode::new_ipc_with_uri(&node1.endpoint());

    println!("IPC-MOCK TWO NODE TEST");
    println!("======================");
    test_fn(&mut node1, &mut node2, false)?;
    println!("===================");
    println!("IPC-MOCKED TEST END\n");
    // Kill nodes
    node1.stop();
    node2.stop();

    Ok(())
}

// Do general test with config
fn launch_two_nodes_rust_mock_test(test_fn: TwoNodesTestFn) -> NetResult<()> {

    let mut node_a = P2pNode::new_mock();
    let mut node_b = P2pNode::new_mock();

    println!("RUST-MOCK TWO NODE TEST");
    println!("=======================");
    test_fn(&mut node_a, &mut node_b, false)?;
    println!("==================");
    println!("RUST-MOCK TEST END\n");
    // Kill nodes
    node_a.stop();
    node_b.stop();

    Ok(())
}


// Do general test with config
fn launch_two_nodes_test(
    n3h_path: &str,
    config_filepath: &str,
    test_fn: TwoNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut node1 = P2pNode::new_ipc_spawn(
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );
    let mut node2 = P2pNode::new_ipc_spawn(
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );

    println!("NORMAL TWO NODE TEST");
    println!("====================");
    test_fn(&mut node1, &mut node2, true)?;
    println!("===============");
    println!("NORMAL TEST END\n");
    // Kill nodes
    node1.stop();
    node2.stop();

    Ok(())
}

// TODO make test: No TrackApp must fail
//fn no_track_test(
//    node1: &mut P2pNode,
//    node2: &mut P2pNode,
//    can_test_connect: bool,
//) -> NetResult<()> {
//    // FIXME: not calling trackApp should make sends or whatever else fail
//    Ok(())
//}

// TODO make test: No connect must fail
// fn no_connect_test()


// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn meta_test(node1: &mut P2pNode, node2: &mut P2pNode, can_connect: bool) -> NetResult<()> {

    // Get each node's current state
    let node1_state = node1.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;
    let node2_state = node2.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;

    // get ipcServer IDs for each node from the IpcServer's state
    let _node1_id;
    let mut node2_binding = String::new();
    if can_connect {
        one_let!(ProtocolWrapper::State(state) = node1_state {
            _node1_id = state.id
        });
        one_let!(ProtocolWrapper::State(state) = node2_state {
            // No bindings in mock mode
            if !state.bindings.is_empty() {
            node2_binding = state.bindings[0].clone();
            }
        });
    }
    // Send TrackApp message on both nodes
    node1.send(
        ProtocolWrapper::TrackDna(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
        })
        .into(),
    )?;
    let connect_result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
    node2.send(
        ProtocolWrapper::TrackDna(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_2.to_string(),
        })
        .into(),
    )?;
    let connect_result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;

    // Connect nodes between them
    if can_connect {
        node1.send(
            ProtocolWrapper::Connect(ConnectData {
                address: node2_binding.into(),
            })
            .into(),
        )?;
        let result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
        one_let!(ProtocolWrapper::PeerConnected(d) = result_1 {
            assert_eq!(d.agent_id, AGENT_ID_2);
        });
        let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
        one_let!(ProtocolWrapper::PeerConnected(d) = result_2 {
            assert_eq!(d.agent_id, AGENT_ID_1);
        });
    }

    // Send data & metadata on same address
    send_and_confirm_data(node1, node2, ENTRY_ADDRESS_1)?;
    send_and_confirm_metadata(node1, node2, ENTRY_ADDRESS_1)?;

    // Again but now send metadata first
    send_and_confirm_metadata(node1, node2, ENTRY_ADDRESS_2)?;
    send_and_confirm_data(node1, node2, ENTRY_ADDRESS_2)?;

    // Again but wait at the end
    // Send 'Store DHT data' message on node 1
    node1.send(
        ProtocolWrapper::PublishDht(DhtData {
            msg_id: "testPublishEntry".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            address: ENTRY_ADDRESS_3.to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    // Send 'Store DHT metadata' message on node 1
    node1.send(
        ProtocolWrapper::PublishDhtMeta(DhtMetaData {
            msg_id: "testPublishMeta".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            from_agent_id: AGENT_ID_1.to_string(),
            address: ENTRY_ADDRESS_3.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
            content: json!("hello-meta"),
        })
        .into(),
    )?;
    // Send 'get DHT data' message on node 2
    node2.send(
        ProtocolWrapper::GetDht(GetDhtData {
            msg_id: "testGetEntry".to_string(),
            dna_address: example_dna_address(),
            from_agent_id: AGENT_ID_2.to_string(),
            address: ENTRY_ADDRESS_3.to_string(),
        })
        .into(),
    )?;
    // Send 'Get DHT data result' message on node 2
    node2.send(
        ProtocolWrapper::GetDhtResult(DhtData {
            msg_id: "testGetEntryResult".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            address: ENTRY_ADDRESS_3.to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    // Send a 'Get DHT metadata' message on node 2
    node2.send(
        ProtocolWrapper::GetDhtMeta(GetDhtMetaData {
            msg_id: "testGetMeta".to_string(),
            dna_address: example_dna_address(),
            from_agent_id: AGENT_ID_2.to_string(),
            address: ENTRY_ADDRESS_3.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
        })
        .into(),
    )?;
    // Send a 'Get DHT metadata result' message on node 2
    node2.send(
        ProtocolWrapper::GetDhtMetaResult(DhtMetaData {
            msg_id: "testGetMetaResult".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            from_agent_id: AGENT_ID_2.to_string(),
            address: ENTRY_ADDRESS_3.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDhtMetaResult(_))))?;
    println!("got GetDhtMetaResult: {:?}", result_2);

    // Done
    Ok(())
}

fn send_and_confirm_data(node1: &mut P2pNode, node2: &mut P2pNode, address: &str) -> NetResult<()> {
    // Send 'Store DHT data' message on node 1
    node1.send(
        ProtocolWrapper::PublishDht(DhtData {
            msg_id: "testPublishEntry".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            address: address.to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    // Check if both nodes received a Store it
    let result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::StoreDht(_))))?;
    println!("got store result 1: {:?}", result_1);
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::StoreDht(_))))?;
    println!("got store result 2: {:?}", result_2);

    // Send 'get DHT data' message on node 2
    node2.send(
        ProtocolWrapper::GetDht(GetDhtData {
            msg_id: "testGetEntry".to_string(),
            dna_address: example_dna_address(),
            from_agent_id: AGENT_ID_2.to_string(),
            address: address.to_string(),
        })
        .into(),
    )?;
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDht(_))))?;
    println!("got dht get: {:?}", result_2);

    // Send 'Get DHT data result' message on node 2
    node2.send(
        ProtocolWrapper::GetDhtResult(DhtData {
            msg_id: "testGetEntryResult".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            address: address.to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDhtResult(_))))?;
    println!("got dht get result: {:?}", result_2);

    Ok(())
}

fn send_and_confirm_metadata(
    node1: &mut P2pNode,
    node2: &mut P2pNode,
    address: &str,
) -> NetResult<()> {
    // Send 'Store DHT metadata' message on node 1
    node1.send(
        ProtocolWrapper::PublishDhtMeta(DhtMetaData {
            msg_id: "testPublishMeta".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            from_agent_id: AGENT_ID_1.to_string(),
            address: address.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
            content: json!("hello-meta"),
        })
        .into(),
    )?;
    // Check if both nodes received a 'Store DHT Metadata' message
    let result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::StoreDhtMeta(_))))?;
    println!("got store meta result 1: {:?}", result_1);
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::StoreDhtMeta(_))))?;
    println!("got store meta result 2: {:?}", result_2);

    // Send a 'Get DHT metadata' message on node 2
    node2.send(
        ProtocolWrapper::GetDhtMeta(GetDhtMetaData {
            msg_id: "testGetMeta".to_string(),
            dna_address: example_dna_address(),
            from_agent_id: AGENT_ID_2.to_string(),
            address: address.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
        })
        .into(),
    )?;
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDhtMeta(_))))?;
    println!("got dht get: {:?}", result_2);

    // Send a 'Get DHT metadata result' message on node 2
    node2.send(
        ProtocolWrapper::GetDhtMetaResult(DhtMetaData {
            msg_id: "testGetMetaResult".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            from_agent_id: AGENT_ID_2.to_string(),
            address: address.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDhtMetaResult(_))))?;
    println!("got dht get result: {:?}", result_2);

    Ok(())
}

fn setup_normal(node_a: &mut P2pNode, node_b: &mut P2pNode) {
    node_a.send(
        ProtocolWrapper::TrackDna(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
        })
            .into(),
    ).expect("Failed sending TrackAppData on node_a");


    node_b.send(
        ProtocolWrapper::TrackDna(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_2.to_string(),
        })
            .into(),
    ).expect("Failed sending TrackAppData on node_b");
}

#[cfg_attr(tarpaulin, skip)]
fn generic_message_test(node_a: &mut P2pNode, node_b: &mut P2pNode, _can_test_connect: bool) -> NetResult<()> {
    println!("generic_message_test() START");

    setup_normal(node_a, node_b);

    // Todo wait for peer connected
    let res = node_b.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
    println!("#### got: {:?}", res);

    println!("#### sending: hello");

    node_a.send(
        ProtocolWrapper::GenericMessage(MessageData {
            dna_address: example_dna_address(),
            to_agent_id: AGENT_ID_2.to_string(),
            from_agent_id: AGENT_ID_1.to_string(),
            msg_id: "yada".to_string(),
            data: json!("hello"),
        })
            .into(),
    ).expect("Failed sending GenericMessage to node_b");

    let res = node_b.wait(Box::new(one_is!(ProtocolWrapper::HandleGenericMessage(_))))?;
    println!("#### got: {:?}", res);

    let msg = match res {
        ProtocolWrapper::HandleGenericMessage(msg) => msg,
        _ => unreachable!(),
    };

    node_b.send(
        ProtocolWrapper::HandleGenericMessageResponse(MessageData {
            dna_address: example_dna_address(),
            to_agent_id: AGENT_ID_1.to_string(),
            from_agent_id: AGENT_ID_2.to_string(),
            msg_id: "yada".to_string(),
            data: json!(format!("echo: {}", msg.data.to_string())),
        })
            .into(),
    ).expect("Failed sending HandleGenericMessageResponse on node_b");


    let res = node_a.wait(Box::new(one_is!(ProtocolWrapper::GenericMessageResponse(_))))?;
    println!("#### got: {:?}", res);

    let msg = match res {
        ProtocolWrapper::GenericMessageResponse(msg) => msg,
        _ => unreachable!(),
    };

    assert_eq!("\"echo: \\\"hello\\\"\"".to_string(), msg.data.to_string());

    println!("generic_message_test() END");
    Ok(())
}


// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn dht_test(node_a: &mut P2pNode, node_b: &mut P2pNode, can_connect: bool) -> NetResult<()> {

    println!("dht_test() START");

    // Get each node's current state
    // let node_state_A = node_a.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;
    // let node_state_B = node_b.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;

    // Send TrackApp message on both nodes
    node_a.send(
        ProtocolWrapper::TrackDna(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
        })
            .into(),
    ).expect("Failed sending TrackAppData on node_a");
    println!("general_test() after track a");
    let connect_result_1 = node_a.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
    println!("self connected result 1: {:?}", connect_result_1);

    node_b.send(
        ProtocolWrapper::TrackDna(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_2.to_string(),
        })
            .into(),
    ).expect("Failed sending TrackAppData on node_b");
    let connect_result_2 = node_b.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
    println!("self connected result 2: {:?}", connect_result_2);

    // get ipcServer IDs for each node from the IpcServer's state
    if can_connect {
        let mut _node1_id = String::new();
        let mut node2_binding = String::new();

        node_a.send(ProtocolWrapper::RequestState.into())
            .expect("Failed sending RequestState on node_a");
        let node_state_A = node_a.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;
        node_b.send(ProtocolWrapper::RequestState.into())
              .expect("Failed sending RequestState on node_b");
        let node_state_B = node_b.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;

        one_let!(ProtocolWrapper::State(state) = node_state_A {
            _node1_id = state.id
        });
        one_let!(ProtocolWrapper::State(state) = node_state_B {
            // No bindings in mock mode
            if !state.bindings.is_empty() {
                node2_binding = state.bindings[0].clone();
            }
        });

        // Connect nodes between them
        println!("connect: node2_binding = {}", node2_binding);
        node_a.send(
            ProtocolWrapper::Connect(ConnectData {
                address: node2_binding.into(),
            })
                .into(),
        )?;

        // Make sure Peers are connected?
        let result_1 = node_a.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
        println!("got connect result 1: {:?}", result_1);
        one_let!(ProtocolWrapper::PeerConnected(d) = result_1 {
            assert_eq!(d.agent_id, AGENT_ID_2);
        });
        let result_2 = node_b.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
        println!("got connect result 2: {:?}", result_2);
        one_let!(ProtocolWrapper::PeerConnected(d) = result_2 {
            assert_eq!(d.agent_id, AGENT_ID_1);
        });
    }

    // Send store DHT data
    node_a.send(
        ProtocolWrapper::PublishDht(DhtData {
            msg_id: "testPub".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            address: ENTRY_ADDRESS_1.to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    // Check if both nodes received it
    let result_1 = node_a.wait(Box::new(one_is!(ProtocolWrapper::StoreDht(_))))?;
    println!("got store result 1: {:?}", result_1);
    let result_2 = node_b.wait(Box::new(one_is!(ProtocolWrapper::StoreDht(_))))?;
    println!("got store result 2: {:?}", result_2);

    // Send get DHT data
    node_b.send(
        ProtocolWrapper::GetDht(GetDhtData {
            msg_id: "testGet".to_string(),
            dna_address: example_dna_address(),
            from_agent_id: AGENT_ID_2.to_string(),
            address: ENTRY_ADDRESS_1.to_string(),
        })
        .into(),
    )?;
    // let result_2 = node_b.wait(Box::new(one_is!(ProtocolWrapper::GetDht(_))))?;
    let result_2 = node_a.wait(Box::new(one_is!(ProtocolWrapper::GetDht(_))))?;
    println!("got dht get: {:?}", result_2);

    // Send get DHT data result
    node_a.send(
        ProtocolWrapper::GetDhtResult(DhtData {
            msg_id: "testGetResult".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            address: ENTRY_ADDRESS_1.to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    let result_2 = node_b.wait(Box::new(one_is!(ProtocolWrapper::GetDhtResult(_))))?;
    println!("got dht get result: {:?}", result_2);

    // DHT metadata
    // ============

    // Send store DHT metadata
    node_a.send(
        ProtocolWrapper::PublishDhtMeta(DhtMetaData {
            msg_id: "testPubMeta".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            from_agent_id: AGENT_ID_1.to_string(),
            address: "test_addr_meta".to_string(),
            attribute: "link__yay".to_string(),
            content: json!("hello-meta"),
        })
        .into(),
    )?;
    // Check if both nodes received it
    let result_1 = node_a.wait(Box::new(one_is!(ProtocolWrapper::StoreDhtMeta(_))))?;
    println!("got store meta result 1: {:?}", result_1);
    let result_2 = node_b.wait(Box::new(one_is!(ProtocolWrapper::StoreDhtMeta(_))))?;
    println!("got store meta result 2: {:?}", result_2);

    // Send get DHT metadata
    node_b.send(
        ProtocolWrapper::GetDhtMeta(GetDhtMetaData {
            msg_id: "testGetMeta".to_string(),
            dna_address: example_dna_address(),
            from_agent_id: AGENT_ID_2.to_string(),
            address: "test_addr".to_string(),
            attribute: "link__yay".to_string(),
        })
        .into(),
    )?;
    let result_2 = node_b.wait(Box::new(one_is!(ProtocolWrapper::GetDhtMeta(_))))?;
    println!("got dht get: {:?}", result_2);

    // Send get DHT metadata result
    node_b.send(
        ProtocolWrapper::GetDhtMetaResult(DhtMetaData {
            msg_id: "testGetMetaResult".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
            from_agent_id: AGENT_ID_2.to_string(),
            address: "test_addr".to_string(),
            attribute: "link:yay".to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    let result_2 = node_b.wait(Box::new(one_is!(ProtocolWrapper::GetDhtMetaResult(_))))?;
    println!("got dht get result: {:?}", result_2);

    // Done
    Ok(())
}
