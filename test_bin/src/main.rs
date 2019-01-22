#![feature(try_from)]
#![allow(non_snake_case)]

extern crate holochain_core_types;
extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;
extern crate tempfile;

pub mod p2p_node;
pub mod publish_hold_workflows;

use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    json_protocol::{
        ConnectData, DhtData, DhtMetaData, FetchDhtData, FetchDhtMetaData, JsonProtocol, MessageData,
        TrackDnaData, HandleDhtResultData, HandleDhtMetaResultData,
    },
    net_connection::NetSend,
    NetResult,
};

use p2p_node::P2pNode;

// CONSTS
static ALEX_AGENT_ID: &'static str = "alex";
static BILLY_AGENT_ID: &'static str = "billy";
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

    // List of tests
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let test_fns: Vec<TwoNodesTestFn> = vec![
        setup_normal,
        send_test,
        dht_test,
        meta_test,
    ];

    // Launch tests on each setup
    for test_fn in test_fns.clone() {
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

// do general test with hackmode
#[cfg_attr(tarpaulin, skip)]
fn launch_two_nodes_test_with_ipc_mock(
    n3h_path: &str,
    config_filepath: &str,
    test_fn: TwoNodesTestFn,
) -> NetResult<()> {
    // Create two nodes
    let mut node1 = P2pNode::new_with_spawn_ipc_network(
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );
    let mut node2 = P2pNode::new_with_uri_ipc_network(&node1.endpoint());

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
#[cfg_attr(tarpaulin, skip)]
fn launch_two_nodes_test_with_memory_network(test_fn: TwoNodesTestFn) -> NetResult<()> {
    let mut node_a = P2pNode::new_with_unique_memory_network();
    let mut node_b = P2pNode::new_with_config(&node_a.config, None);

    println!("IN-MEMORY TWO NODE TEST");
    println!("=======================");
    test_fn(&mut node_a, &mut node_b, false)?;
    println!("==================");
    println!("IN-MEMORY TEST END\n");
    // Kill nodes
    node_a.stop();
    node_b.stop();

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
    let mut node1 = P2pNode::new_with_spawn_ipc_network(
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    );
    let mut node2 = P2pNode::new_with_spawn_ipc_network(
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

// TODO make test: Sending a Message before doing a 'TrackApp' should fail
//fn no_track_test(
//    node1: &mut P2pNode,
//    node2: &mut P2pNode,
//    can_test_connect: bool,
//) -> NetResult<()> {
//    // FIXME: not calling trackApp should make sends or whatever else fail
//    Ok(())
//}

// TODO make test: Sending a Message before doing a 'Connect' should fail.
// fn no_connect_test()

/// Tests if we can get back data published on the network
#[cfg_attr(tarpaulin, skip)]
fn confirm_published_data(alex: &mut P2pNode, billy: &mut P2pNode, address: &str) -> NetResult<()> {
    // Alex publishs data on the network
    alex.send(
        JsonProtocol::PublishDhtData(DhtData {
            dna_address: example_dna_address(),
            provider_agent_id: ALEX_AGENT_ID.to_string(),
            data_address: address.to_string(),
            data_content: json!("hello"),
        })
        .into(),
    )?;
    // Check if both nodes received a HandleStore command.
    let result_a = alex.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtData(_))))?;
    println!(" got store result A: {:?}\n", result_a);
    let result_b = billy.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtData(_))))?;
    println!("got store result B: {:?}\n", result_b);

    // Billy asks for that data on the network.
    billy.send(
        JsonProtocol::FetchDhtData(FetchDhtData {
            request_id: "testGetEntry".to_string(),
            dna_address: example_dna_address(),
            requester_agent_id: BILLY_AGENT_ID.to_string(),
            data_address: address.to_string(),
        })
        .into(),
    )?;
    // Alex having that data, sends it to the network.
    alex.send(
        JsonProtocol::HandleFetchDhtDataResult(HandleDhtResultData {
            request_id: "testGetEntryResult".to_string(),
            requester_agent_id: BILLY_AGENT_ID.to_string(),
            dna_address: example_dna_address(),
            provider_agent_id: ALEX_AGENT_ID.to_string(),
            data_address: address.to_string(),
            data_content: json!("hello"),
        })
        .into(),
    )?;
    // Alex should receive the data it requested from the netowrk
    // FIXME: Should be Billy instead!
    let result = alex.wait(Box::new(one_is!(JsonProtocol::FetchDhtDataResult(_))))?;
    println!("got dht data result: {:?}", result);

    Ok(())
}

/// Tests if we can get back metadata published on the network
#[cfg_attr(tarpaulin, skip)]
fn confirm_published_metadata(
    alex: &mut P2pNode,
    billy: &mut P2pNode,
    address: &str,
) -> NetResult<()> {
    // Alex publishs metadata on the network
    alex.send(
        JsonProtocol::PublishDhtMeta(DhtMetaData {
            dna_address: example_dna_address(),
            provider_agent_id: ALEX_AGENT_ID.to_string(),
            data_address: address.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
            content: json!("hello-meta"),
        })
        .into(),
    )?;
    // Check if both nodes received a HandleStore command.
    let result_a = alex.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtMeta(_))))?;
    println!("got store meta result 1: {:?}", result_a);
    let result_b = billy.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtMeta(_))))?;
    println!("got store meta result 2: {:?}", result_b);

    // Billy asks for that metadata on the network.
    billy.send(
        JsonProtocol::FetchDhtMeta(FetchDhtMetaData {
            request_id: "testGetMeta".to_string(),
            dna_address: example_dna_address(),
            requester_agent_id: BILLY_AGENT_ID.to_string(),
            data_address: address.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
        })
        .into(),
    )?;
    // Alex having that metadata, sends it to the network.
    alex.send(
        JsonProtocol::HandleFetchDhtMetaResult(HandleDhtMetaResultData {
            request_id: "testGetMetaResult".to_string(),
            dna_address: example_dna_address(),
            requester_agent_id: ALEX_AGENT_ID.to_string(),
            provider_agent_id: BILLY_AGENT_ID.to_string(),
            data_address: address.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    // Alex should receive the metadata it requested from the netowrk
    // FIXME: Billy should be the one asking instead!
    let result = alex.wait(Box::new(one_is!(JsonProtocol::FetchDhtMetaResult(_))))?;
    println!("got dht meta result: {:?}", result);

    Ok(())
}

/// Do normal setup: 'TrackDna' & 'Connect',
/// and check that we received 'PeerConnected'
#[cfg_attr(tarpaulin, skip)]
fn setup_normal(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Send TrackDna message on both nodes
    alex.send(
        JsonProtocol::TrackDna(TrackDnaData {
            dna_address: example_dna_address(),
            agent_id: ALEX_AGENT_ID.to_string(),
        })
        .into(),
    )
    .expect("Failed sending TrackDnaData on alex");
    let connect_result_1 = alex.wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))?;
    println!("self connected result 1: {:?}", connect_result_1);
    billy
        .send(
            JsonProtocol::TrackDna(TrackDnaData {
                dna_address: example_dna_address(),
                agent_id: BILLY_AGENT_ID.to_string(),
            })
            .into(),
        )
        .expect("Failed sending TrackDnaData on billy");
    let connect_result_2 = billy.wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))?;
    println!("self connected result 2: {:?}", connect_result_2);

    // get ipcServer IDs for each node from the IpcServer's state
    if can_connect {
        let mut _node1_id = String::new();
        let mut node2_binding = String::new();

        alex.send(JsonProtocol::GetState.into())
            .expect("Failed sending RequestState on alex");
        let node_state_A = alex.wait(Box::new(one_is!(JsonProtocol::GetStateResult(_))))?;
        billy
            .send(JsonProtocol::GetState.into())
            .expect("Failed sending RequestState on billy");
        let node_state_B = billy.wait(Box::new(one_is!(JsonProtocol::GetStateResult(_))))?;

        one_let!(JsonProtocol::GetStateResult(state) = node_state_A {
            _node1_id = state.id
        });
        one_let!(JsonProtocol::GetStateResult(state) = node_state_B {
            if !state.bindings.is_empty() {
                node2_binding = state.bindings[0].clone();
            }
        });

        // Connect nodes between them
        println!("connect: node2_binding = {}", node2_binding);
        alex.send(
            JsonProtocol::Connect(ConnectData {
                peer_address: node2_binding.into(),
            })
            .into(),
        )?;

        // Make sure Peers are connected
        let result_a = alex.wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))?;
        println!("got connect result A: {:?}", result_a);
        one_let!(JsonProtocol::PeerConnected(d) = result_a {
            assert_eq!(d.agent_id, BILLY_AGENT_ID);
        });
        let result_b = billy.wait(Box::new(one_is!(JsonProtocol::PeerConnected(_))))?;
        println!("got connect result B: {:?}", result_b);
        one_let!(JsonProtocol::PeerConnected(d) = result_b {
            assert_eq!(d.agent_id, ALEX_AGENT_ID);
        });
    }

    // Done
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
fn send_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: send_test()");
    setup_normal(alex, billy, can_connect)?;

    println!("setup done");

    // Send a message from alex to billy
    alex.send(
        JsonProtocol::SendMessage(MessageData {
            dna_address: example_dna_address(),
            to_agent_id: BILLY_AGENT_ID.to_string(),
            from_agent_id: ALEX_AGENT_ID.to_string(),
            msg_id: "yada".to_string(),
            data: json!("hello"),
        })
        .into(),
    )
    .expect("Failed sending SendMessage to billy");

    println!("SendMessage done");

    // Check if billy received it
    let res = billy.wait(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))))?;
    println!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::HandleSendMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!("\"hello\"".to_string(), msg.data.to_string());

    // Send a message back from billy to alex
    billy
        .send(
            JsonProtocol::HandleSendMessageResult(MessageData {
                dna_address: example_dna_address(),
                to_agent_id: ALEX_AGENT_ID.to_string(),
                from_agent_id: BILLY_AGENT_ID.to_string(),
                msg_id: "yada".to_string(),
                data: json!(format!("echo: {}", msg.data.to_string())),
            })
            .into(),
        )
        .expect("Failed sending HandleSendResult on billy");
    // Check if alex received it
    let res = alex.wait(Box::new(one_is!(JsonProtocol::SendMessageResult(_))))?;
    println!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::SendMessageResult(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!("\"echo: \\\"hello\\\"\"".to_string(), msg.data.to_string());

    // Done
    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn meta_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: meta_test()");
    setup_normal(alex, billy, can_connect)?;

    // Send data & metadata on same address
    confirm_published_data(alex, billy, ENTRY_ADDRESS_1)?;
    confirm_published_metadata(alex, billy, ENTRY_ADDRESS_1)?;

    // Again but now send metadata first
    confirm_published_metadata(alex, billy, ENTRY_ADDRESS_2)?;
    confirm_published_data(alex, billy, ENTRY_ADDRESS_2)?;

    // Again but 'wait' at the end
    // Alex publishs data & meta on the network
    alex.send(
        JsonProtocol::PublishDhtData(DhtData {
            dna_address: example_dna_address(),
            provider_agent_id: ALEX_AGENT_ID.to_string(),
            data_address: ENTRY_ADDRESS_3.to_string(),
            data_content: json!("hello"),
        })
        .into(),
    )?;
    alex.send(
        JsonProtocol::PublishDhtMeta(DhtMetaData {
            dna_address: example_dna_address(),
            provider_agent_id: ALEX_AGENT_ID.to_string(),
            data_address: ENTRY_ADDRESS_3.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
            content: json!("hello-meta"),
        })
        .into(),
    )?;
    // Billy sends GetDhtData message
    billy.send(
        JsonProtocol::FetchDhtData(FetchDhtData {
            request_id: "testGetEntry".to_string(),
            dna_address: example_dna_address(),
            requester_agent_id: BILLY_AGENT_ID.to_string(),
            data_address: ENTRY_ADDRESS_3.to_string(),
        })
        .into(),
    )?;
    // Billy sends HandleGetDhtDataResult message
    billy.send(
        JsonProtocol::HandleFetchDhtDataResult(HandleDhtResultData {
            request_id: "testGetEntryResult".to_string(),
            requester_agent_id: ALEX_AGENT_ID.to_string(),
            dna_address: example_dna_address(),
            provider_agent_id: BILLY_AGENT_ID.to_string(),
            data_address: ENTRY_ADDRESS_3.to_string(),
            data_content: json!("hello"),
        })
        .into(),
    )?;
    // Billy sends GetDhtMeta message
    billy.send(
        JsonProtocol::FetchDhtMeta(FetchDhtMetaData {
            request_id: "testGetMeta".to_string(),
            dna_address: example_dna_address(),
            requester_agent_id: BILLY_AGENT_ID.to_string(),
            data_address: ENTRY_ADDRESS_3.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
        })
        .into(),
    )?;
    // Alex sends HandleGetDhtMetaResult message
    alex.send(
        JsonProtocol::HandleFetchDhtMetaResult(HandleDhtMetaResultData {
            request_id: "testGetMetaResult".to_string(),
            dna_address: example_dna_address(),
            requester_agent_id: ALEX_AGENT_ID.to_string(),
            provider_agent_id: BILLY_AGENT_ID.to_string(),
            data_address: ENTRY_ADDRESS_3.to_string(),
            attribute: META_ATTRIBUTE.to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;
    // Alex should receive requested metadata
    // FIXME: Billy should be the one asking instead!
    let result = alex.wait(Box::new(one_is!(JsonProtocol::FetchDhtMetaResult(_))))?;
    println!("got GetDhtMetaResult: {:?}", result);
    // Done
    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn dht_test(alex: &mut P2pNode, billy: &mut P2pNode, can_connect: bool) -> NetResult<()> {
    // Setup
    println!("Testing: dht_test()");
    setup_normal(alex, billy, can_connect)?;

    // Alex publish data on the network
    alex.send(
        JsonProtocol::PublishDhtData(DhtData {
            dna_address: example_dna_address(),
            provider_agent_id: ALEX_AGENT_ID.to_string(),
            data_address: ENTRY_ADDRESS_1.to_string(),
            data_content: json!("hello"),
        })
        .into(),
    )?;
    // Check if both nodes are asked to store it
    let result_a = alex.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtData(_))))?;
    println!("got HandleStoreDhtData on node A: {:?}", result_a);
    let result_b = billy.wait(Box::new(one_is!(JsonProtocol::HandleStoreDhtData(_))))?;
    println!("got HandleStoreDhtData on node B: {:?}", result_b);

    // Billy asks for that data
    billy.send(
        JsonProtocol::FetchDhtData(FetchDhtData {
            request_id: "testGet".to_string(),
            dna_address: example_dna_address(),
            requester_agent_id: BILLY_AGENT_ID.to_string(),
            data_address: ENTRY_ADDRESS_1.to_string(),
        })
        .into(),
    )?;
    // Alex sends that data back to the network
    alex.send(
        JsonProtocol::HandleFetchDhtDataResult(HandleDhtResultData {
            request_id: "testGetResult".to_string(),
            requester_agent_id: BILLY_AGENT_ID.to_string(),
            dna_address: example_dna_address(),
            provider_agent_id: ALEX_AGENT_ID.to_string(),
            data_address: ENTRY_ADDRESS_1.to_string(),
            data_content: json!("hello"),
        })
        .into(),
    )?;
    // Alex should receive requested data
    // FIXME: Billy should be the one asking instead!
    let result = alex.wait(Box::new(one_is!(JsonProtocol::FetchDhtDataResult(_))))?;
    println!("got GetDhtDataResult: {:?}", result);
    // Done
    Ok(())
}
