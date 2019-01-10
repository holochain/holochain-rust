#![feature(try_from)]

extern crate holochain_core_types;
extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;
extern crate tempfile;

pub mod p2p_node;

use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    net_connection::NetConnection,
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

// Do normal tests according to config
fn launch_test_with_config(n3h_path: &str, config_filepath: &str) -> NetResult<()> {
    launch_two_nodes_test(n3h_path, config_filepath, general_test)?;
    launch_two_nodes_test(n3h_path, config_filepath, meta_test)?;
    Ok(())
}

// Do ipc-mock tests according to config
fn launch_test_with_ipc_mock(n3h_path: &str, config_filepath: &str) -> NetResult<()> {
    launch_two_nodes_test_with_ipc_mock(n3h_path, config_filepath, general_test)?;
    launch_two_nodes_test_with_ipc_mock(n3h_path, config_filepath, meta_test)?;
    Ok(())
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

    println!("MOCKED TWO NODE TEST");
    println!("====================");
    test_fn(&mut node1, &mut node2, false)?;
    println!("===============");
    println!("MOCKED TEST END\n");
    // Kill nodes
    node1.stop();
    node2.stop();

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

//fn no_track_test(
//    node1: &mut P2pNode,
//    node2: &mut P2pNode,
//    can_test_connect: bool,
//) -> NetResult<()> {
//    // FIXME: not calling trackApp should make sends or whatever else fail
//    Ok(())
//}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn meta_test(node1: &mut P2pNode, node2: &mut P2pNode, can_test_connect: bool) -> NetResult<()> {
    // Get each node's current state
    let node1_state = node1.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;
    let node2_state = node2.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;

    // get ipcServer IDs for each node from the IpcServer's state
    let node1_id;
    let mut node2_binding = String::new();
    if can_test_connect {
        one_let!(ProtocolWrapper::State(state) = node1_state {
            node1_id = state.id
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
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
        })
        .into(),
    )?;
    let connect_result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
    node2.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_2.to_string(),
        })
        .into(),
    )?;
    let connect_result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;

    // Connect nodes between them
    if can_test_connect {
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

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn general_test(node1: &mut P2pNode, node2: &mut P2pNode, can_test_connect: bool) -> NetResult<()> {
    static AGENT_1: &'static str = "1_TEST_AGENT_1";
    static AGENT_2: &'static str = "2_TEST_AGENT_2";

    fn example_dna_address() -> Address {
        "TEST_DNA_ADDRESS".into()
    }

    // Get each node's current state
    let node1_state = node1.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;
    let node2_state = node2.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;

    // get ipcServer IDs for each node from the IpcServer's state
    let node1_id;
    let mut node2_binding = String::new();
    one_let!(ProtocolWrapper::State(state) = node1_state {
        node1_id = state.id
    });
    one_let!(ProtocolWrapper::State(state) = node2_state {
        // No bindings in mock mode
        if !state.bindings.is_empty() {
            node2_binding = state.bindings[0].clone();
        }
    });

    // Send TrackApp message on both nodes
    node1.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_1.to_string(),
        })
        .into(),
    )?;
    let connect_result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
    println!("self connected result 1: {:?}", connect_result_1);
    node2.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_2.to_string(),
        })
        .into(),
    )?;
    let connect_result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
    println!("self connected result 2: {:?}", connect_result_2);

    // Connect nodes between them
    if can_test_connect {
        println!("connect node1 ({}) to node2 ({})", node1_id, node2_binding);
        node1.send(
            ProtocolWrapper::Connect(ConnectData {
                address: node2_binding.into(),
            })
            .into(),
        )?;
        let result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
        println!("got connect result 1: {:?}", result_1);
        one_let!(ProtocolWrapper::PeerConnected(d) = result_1 {
            assert_eq!(d.agent_id, AGENT_2);
        });
        let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
        println!("got connect result 2: {:?}", result_2);
        one_let!(ProtocolWrapper::PeerConnected(d) = result_2 {
            assert_eq!(d.agent_id, AGENT_1);
        });
    }
    // Send a generic message
    node1.send(
        ProtocolWrapper::SendMessage(MessageData {
            msg_id: "test".to_string(),
            dna_address: example_dna_address(),
            to_agent_id: AGENT_2.to_string(),
            from_agent_id: AGENT_1.to_string(),
            data: json!("hello"),
        })
        .into(),
    )?;
    // Check if node2 received it
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::HandleSend(_))))?;
    println!("got handle send 2: {:?}", result_2);
    node2.send(
        ProtocolWrapper::HandleSendResult(MessageData {
            msg_id: "test".to_string(),
            dna_address: example_dna_address(),
            to_agent_id: AGENT_1.to_string(),
            from_agent_id: AGENT_2.to_string(),
            data: json!("echo: hello"),
        })
        .into(),
    )?;
    let result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::SendResult(_))))?;
    println!("got send result 1: {:?}", result_1);

    // Send store DHT data
    node1.send(
        ProtocolWrapper::PublishDht(DhtData {
            msg_id: "testPub".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_1.to_string(),
            address: "test_addr".to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    // Check if both nodes received it
    let result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::StoreDht(_))))?;
    println!("got store result 1: {:?}", result_1);
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::StoreDht(_))))?;
    println!("got store result 2: {:?}", result_2);

    // Send get DHT data
    node2.send(
        ProtocolWrapper::GetDht(GetDhtData {
            msg_id: "testGet".to_string(),
            dna_address: example_dna_address(),
            from_agent_id: AGENT_2.to_string(),
            address: "test_addr".to_string(),
        })
        .into(),
    )?;
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDht(_))))?;
    println!("got dht get: {:?}", result_2);

    // Send get DHT data result
    node2.send(
        ProtocolWrapper::GetDhtResult(DhtData {
            msg_id: "testGetResult".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_1.to_string(),
            address: "test_addr".to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDhtResult(_))))?;
    println!("got dht get result: {:?}", result_2);

    // Send store DHT metadata
    node1.send(
        ProtocolWrapper::PublishDhtMeta(DhtMetaData {
            msg_id: "testPubMeta".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_1.to_string(),
            from_agent_id: AGENT_1.to_string(),
            address: "test_addr_meta".to_string(),
            attribute: "link__yay".to_string(),
            content: json!("hello-meta"),
        })
        .into(),
    )?;
    // Check if both nodes received it
    let result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::StoreDhtMeta(_))))?;
    println!("got store meta result 1: {:?}", result_1);
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::StoreDhtMeta(_))))?;
    println!("got store meta result 2: {:?}", result_2);

    // Send get DHT metadata
    node2.send(
        ProtocolWrapper::GetDhtMeta(GetDhtMetaData {
            msg_id: "testGetMeta".to_string(),
            dna_address: example_dna_address(),
            from_agent_id: AGENT_2.to_string(),
            address: "test_addr".to_string(),
            attribute: "link:yay".to_string(),
        })
        .into(),
    )?;
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDhtMeta(_))))?;
    println!("got dht get: {:?}", result_2);

    // Send get DHT metadata result
    node2.send(
        ProtocolWrapper::GetDhtMetaResult(DhtMetaData {
            msg_id: "testGetMetaResult".to_string(),
            dna_address: example_dna_address(),
            agent_id: AGENT_1.to_string(),
            from_agent_id: AGENT_2.to_string(),
            address: "test_addr".to_string(),
            attribute: "link:yay".to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDhtMetaResult(_))))?;
    println!("got dht get result: {:?}", result_2);

    // Done
    Ok(())
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
    // Launch hackmode test
    let res = launch_test_with_config(&n3h_path, "test_bin/data/network_config.json");
    assert!(res.is_ok());

    // Launch mock test
    let res = launch_test_with_ipc_mock(&n3h_path, "test_bin/data/mock_network_config.json");
    assert!(res.is_ok());

    // Wait a bit before closing
    for i in (0..4).rev() {
        println!("tick... {}", i);
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
