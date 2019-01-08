#![feature(try_from)]

extern crate holochain_core_types;
extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;
extern crate tempfile;

use holochain_core_types::cas::content::Address;
use holochain_net::{p2p_config::*, p2p_network::P2pNetwork};
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol::Protocol,
    protocol_wrapper::{
        ConnectData, DhtData, DhtMetaData, GetDhtData, GetDhtMetaData, MessageData,
        ProtocolWrapper, TrackAppData,
    },
    NetResult,
};
use std::{convert::TryFrom, sync::mpsc};

// CONSTS
static AGENT_ID_1: &'static str = "DUMMY_AGENT_1";
static AGENT_ID_2: &'static str = "DUMMY_AGENT_2";
static ENTRY_ADDRESS_1: &'static str = "dummy_addr_1";
static ENTRY_ADDRESS_2: &'static str = "dummy_addr_2";
static DNA_ADDRESS: &'static str = "DUMMY_DNA_ADDRESS";
static META_ATTRIBUTE: &'static str = "link__yay";

fn example_dna_address() -> Address {
    DNA_ADDRESS.into()
}


type TwoNodesTestFn = fn(node1: &mut IpcNode, node2: &mut IpcNode, can_test_connect: bool) -> NetResult<()>;

// Do general test with config
fn launch_test_with_config(n3h_path: &str, config_filepath: &str) -> NetResult<()> {
    launch_two_nodes_test(n3h_path, config_filepath, general_test)?;
    launch_two_nodes_test(n3h_path, config_filepath, meta_test)?;
    Ok(())
}

// Do general test with config
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

struct IpcNode {
    pub temp_dir_ref: tempfile::TempDir,
    pub dir: String,
    pub p2p_connection: P2pNetwork,
    pub receiver: mpsc::Receiver<Protocol>,
}

static TIMEOUT_MS: usize = 5000;

impl IpcNode {
    // See if there is a message to receive
    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv(&mut self) -> NetResult<ProtocolWrapper> {
        let data = self.receiver.try_recv()?;
        match ProtocolWrapper::try_from(&data) {
            Ok(r) => Ok(r),
            Err(e) => {
                let s = format!("{:?}", e);
                if !s.contains("Empty") && !s.contains("Pong(PongData") {
                    println!("##### parse error ##### : {} {:?}", s, data);
                }
                Err(e)
            }
        }
    }

    // Wait for a message corresponding to predicate
    #[cfg_attr(tarpaulin, skip)]
    pub fn wait(
        &mut self,
        predicate: Box<dyn Fn(&ProtocolWrapper) -> bool>,
    ) -> NetResult<ProtocolWrapper> {
        let mut time_ms: usize = 0;
        loop {
            let mut did_something = false;

            if let Ok(p2p_msg) = self.try_recv() {
                did_something = true;
                if predicate(&p2p_msg) {
                    return Ok(p2p_msg);
                }
            }

            if !did_something {
                std::thread::sleep(std::time::Duration::from_millis(10));
                time_ms += 10;
                if time_ms > TIMEOUT_MS {
                    panic!("TIMEOUT");
                }
            }
        }
    }

    // Stop node
    #[cfg_attr(tarpaulin, skip)]
    pub fn stop(self) {
        self.p2p_connection.stop().unwrap();
    }
}

// Spawn an IPC node that uses n3h and a temp folder
#[cfg_attr(tarpaulin, skip)]
fn create_config(
    n3h_path: &str,
    maybe_config_filepath: Option<&str>,
    bootstrap_nodes: Vec<String>,
) -> (P2pConfig, tempfile::TempDir) {
    // Create temp directory
    let dir_ref = tempfile::tempdir().expect("Failed to created a temp directory.");
    let dir = dir_ref.path().to_string_lossy().to_string();
    // Create config
    let config = match maybe_config_filepath {
        Some(filepath) => {
            // Get config from file
            let p2p_config = P2pConfig::from_file(filepath);

            // complement missing fields
            serde_json::from_value(json!({
            "backend_kind": String::from(p2p_config.backend_kind),
            "backend_config":
            {
                "socketType": p2p_config.backend_config["socketType"],
                "bootstrapNodes": bootstrap_nodes,
                "spawn":
                {
                    "cmd": p2p_config.backend_config["spawn"]["cmd"],
                    "args": [
                        format!("{}/packages/n3h/bin/n3h", n3h_path)
                    ],
                    "workDir": dir.clone(),
                    "env": {
                        "N3H_MODE": p2p_config.backend_config["spawn"]["env"]["N3H_MODE"],
                        "N3H_WORK_DIR": dir.clone(),
                        "N3H_IPC_SOCKET": p2p_config.backend_config["spawn"]["env"]["N3H_IPC_SOCKET"],
                    }
                },
            }})).unwrap()
        }
        None => {
            // use default config
            serde_json::from_value(json!({
            "backend_kind": "IPC",
            "backend_config":
            {
                "socketType": "zmq",
                "bootstrapNodes": bootstrap_nodes,
                "spawn":
                {
                    "cmd": "node",
                    "args": [
                        format!("{}/packages/n3h/bin/n3h", n3h_path)
                    ],
                    "workDir": dir.clone(),
                    "env": {
                        "N3H_MODE": "HACK",
                        "N3H_WORK_DIR": dir.clone(),
                        "N3H_IPC_SOCKET": "tcp://127.0.0.1:*",
                }
            },
            }}))
            .unwrap()
        }
    };
    return (config, dir_ref);
}

// Create an IPC node that uses an existing n3h process and a temp folder
#[cfg_attr(tarpaulin, skip)]
fn create_borrowed_connection(ipc_binding: &str) -> NetResult<IpcNode> {
    // Create Config
    let p2p_config = P2pConfig::default_ipc_uri(Some(ipc_binding));
    // Create channel
    let (sender, receiver) = mpsc::channel::<Protocol>();
    // Create P2pNetwork
    let p2p_node = P2pNetwork::new(
        Box::new(move |r| {
            sender.send(r?)?;
            Ok(())
        }),
        &p2p_config,
    )?;
    // Create temp directory
    let dir_ref = tempfile::tempdir().expect("Failed to created a temp directory.");
    // Create IpcNode
    Ok(IpcNode {
        dir: dir_ref.path().to_string_lossy().to_string(),
        temp_dir_ref: dir_ref,
        p2p_connection: p2p_node,
        receiver,
    })
}

// Create an IPC node that spawns and uses a n3h process and a temp folder
#[cfg_attr(tarpaulin, skip)]
fn create_spawned_connection(
    n3h_path: &str,
    maybe_config_filepath: Option<&str>,
    bootstrap_nodes: Vec<String>,
) -> NetResult<IpcNode> {
    // Create Config
    let (p2p_config, dir_ref) = create_config(n3h_path, maybe_config_filepath, bootstrap_nodes);
    // Create channel
    let (sender, receiver) = mpsc::channel::<Protocol>();
    // Create P2pNetwork
    let p2p_node = P2pNetwork::new(
        Box::new(move |r| {
            sender.send(r?)?;
            Ok(())
        }),
        &p2p_config,
    )?;
    // Create IpcNode
    Ok(IpcNode {
        dir: dir_ref.path().to_string_lossy().to_string(),
        temp_dir_ref: dir_ref,
        p2p_connection: p2p_node,
        receiver,
    })
}

// do general test with hackmode
fn launch_two_nodes_test_with_ipc_mock(n3h_path: &str, config_filepath: &str, test_fn: TwoNodesTestFn) -> NetResult<()> {
    // Create two nodes
    let mut node1 = create_spawned_connection(
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    )?;
    let mut node2 = create_borrowed_connection(&node1.p2p_connection.endpoint())?;

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
fn launch_two_nodes_test(n3h_path: &str, config_filepath: &str, test_fn: TwoNodesTestFn) -> NetResult<()> {

    // Create two nodes
    let mut node1 = create_spawned_connection(
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    )?;
    let mut node2 = create_spawned_connection(
        n3h_path,
        Some(config_filepath),
        vec!["/ip4/127.0.0.1/tcp/12345/ipfs/blabla".to_string()],
    )?;

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


fn no_track_test(node1: &mut IpcNode, node2: &mut IpcNode, can_test_connect: bool) -> NetResult<()> {
    // FIXME: not calling trackApp should make sends or whatever else fail
    Ok(())
}


// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn meta_test(node1: &mut IpcNode, node2: &mut IpcNode, can_test_connect: bool) -> NetResult<()> {

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
    node1.p2p_connection.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_1.to_string(),
        })
            .into(),
    )?;
    let connect_result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
    node2.p2p_connection.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_ID_2.to_string(),
        })
            .into(),
    )?;
    let connect_result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;

    // Connect nodes between them
    if can_test_connect {
        node1.p2p_connection.send(
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
    send_and_check_data(node1, node2, ENTRY_ADDRESS_1)?;
    send_and_check_metadata(node1, node2,ENTRY_ADDRESS_1)?;

    // Again but now send metadata first
    send_and_check_metadata(node1, node2,ENTRY_ADDRESS_2)?;
    send_and_check_data(node1, node2,ENTRY_ADDRESS_2)?;

    // Done
    Ok(())
}

fn send_and_check_data(node1: &mut IpcNode, node2: &mut IpcNode, address: &str) -> NetResult<()> {
    // Send 'Store DHT data' message on node 1
    node1.p2p_connection.send(
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
    node2.p2p_connection.send(
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
    node2.p2p_connection.send(
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


fn send_and_check_metadata(node1: &mut IpcNode, node2: &mut IpcNode, address: &str) -> NetResult<()>  {
    // Send 'Store DHT metadata' message on node 1
    node1.p2p_connection.send(
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
    node2.p2p_connection.send(
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
    node2.p2p_connection.send(
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
fn general_test(node1: &mut IpcNode, node2: &mut IpcNode, can_test_connect: bool) -> NetResult<()> {
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
    node1.p2p_connection.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_address: example_dna_address(),
            agent_id: AGENT_1.to_string(),
        })
        .into(),
    )?;
    let connect_result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
    println!("self connected result 1: {:?}", connect_result_1);
    node2.p2p_connection.send(
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
        node1.p2p_connection.send(
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
    node1.p2p_connection.send(
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
    node2.p2p_connection.send(
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
    node1.p2p_connection.send(
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
    node2.p2p_connection.send(
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
    node2.p2p_connection.send(
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
    node1.p2p_connection.send(
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
    node2.p2p_connection.send(
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
    node2.p2p_connection.send(
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
    let res = launch_test_with_config(&n3h_path, "test_bin/src/network_config.json");
    assert!(res.is_ok());

    // Launch mock test
    let res = launch_test_with_ipc_mock(&n3h_path, "test_bin/src/mock_network_config.json");
    assert!(res.is_ok());

    // Wait a bit before closing
    for i in (0..4).rev() {
        println!("tick... {}", i);
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
