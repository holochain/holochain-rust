#![feature(try_from)]

extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;
extern crate tempfile;

use holochain_net_connection::{
    net_connection::NetConnection,
    protocol::Protocol,
    protocol_wrapper::{ConnectData, TrackAppData, ProtocolWrapper},
    NetResult,
};

use holochain_net::p2p_network::P2pNetwork;

use std::{convert::TryFrom, sync::mpsc};

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn usage() {
    println!("Usage: test_bin_ipc <path_to_n3h>");
    std::process::exit(1);
}

struct SpawnResult {
    pub dir_ref: tempfile::TempDir,
    pub dir: String,
    pub con: P2pNetwork,
    pub receiver: mpsc::Receiver<Protocol>,
}

impl SpawnResult {
    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv(&mut self) -> NetResult<ProtocolWrapper> {
        let data = self.receiver.try_recv()?;
        Ok(ProtocolWrapper::try_from(data)?)
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn wait(
        &mut self,
        predicate: Box<dyn Fn(&ProtocolWrapper) -> bool>,
    ) -> NetResult<ProtocolWrapper> {
        loop {
            let mut did_something = false;

            if let Ok(r) = self.try_recv() {
                did_something = true;
                if predicate(&r) {
                    return Ok(r);
                }
            }

            if !did_something {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn drop(self) {
        self.con.stop().unwrap();
    }
}

#[cfg_attr(tarpaulin, skip)]
fn spawn_connection(n3h_path: &str) -> NetResult<SpawnResult> {
    let dir_ref = tempfile::tempdir()?;
    let dir = dir_ref.path().to_string_lossy().to_string();

    let (sender, receiver) = mpsc::channel::<Protocol>();

    let con = P2pNetwork::new(
        Box::new(move |r| {
            sender.send(r?)?;
            Ok(())
        }),
        &json!({
            "backend": "ipc",
            "config": {
                "socketType": "zmq",
                "spawn": {
                    "cmd": format!("{}/packages/n3h/bin/n3h", n3h_path),
                    "args": [],
                    "workDir": dir.clone(),
                    "env": {
                        "N3H_HACK_MODE": "1",
                        "N3H_WORK_DIR": dir.clone(),
                        "N3H_IPC_SOCKET": "tcp://127.0.0.1:*",
                    }
                },
            }
        })
        .into(),
    )?;

    Ok(SpawnResult {
        dir_ref,
        dir,
        con,
        receiver,
    })
}

#[allow(unused)]
#[cfg_attr(tarpaulin, skip)]
fn is_any(_data: &ProtocolWrapper) -> bool {
    return true;
}

#[cfg_attr(tarpaulin, skip)]
fn is_state(data: &ProtocolWrapper) -> bool {
    if let ProtocolWrapper::State(_s) = data {
        return true;
    }
    return false;
}

#[cfg_attr(tarpaulin, skip)]
fn is_peer_connected(data: &ProtocolWrapper) -> bool {
    if let ProtocolWrapper::PeerConnected(_id) = data {
        return true;
    }
    return false;
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn exec() -> NetResult<()> {
    static DNA_HASH: &'static str = "TEST_DNA_HASH";
    static AGENT_1: &'static str = "1_TEST_AGENT_1";
    static AGENT_2: &'static str = "2_TEST_AGENT_2";

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        usage();
    }

    let n3h_path = args[1].clone();

    if n3h_path == "" {
        usage();
    }

    let mut node1 = spawn_connection(&n3h_path)?;
    let mut node2 = spawn_connection(&n3h_path)?;

    println!("node1 path: {}", node1.dir);
    println!("node2 path: {}", node2.dir);

    let node1_state = node1.wait(Box::new(is_state))?;
    let node2_state = node2.wait(Box::new(is_state))?;

    let node1_id;
    let node2_id;
    let node2_binding;

    if let ProtocolWrapper::State(s) = node1_state {
        node1_id = s.id;
    } else {
        unimplemented!()
    }

    if let ProtocolWrapper::State(s) = node2_state {
        node2_id = s.id;
        node2_binding = s.bindings[0].clone();
    } else {
        unimplemented!()
    }

    println!("connect node1 ({}) to node2 ({})", node1_id, node2_binding);

    node1.con.send(
        ProtocolWrapper::Connect(ConnectData {
            address: node2_binding,
        })
        .into(),
    )?;

    let connect_result_1 = node1.wait(Box::new(is_peer_connected))?;
    println!("got connect result 1: {:?}", connect_result_1);

    let connect_result_2 = node2.wait(Box::new(is_peer_connected))?;
    println!("got connect result 2: {:?}", connect_result_2);

    node1.con.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_hash: DNA_HASH.to_string(),
            agent_id: AGENT_1.to_string(),
        })
        .into(),
    )?;

    node2.con.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_hash: DNA_HASH.to_string(),
            agent_id: AGENT_2.to_string(),
        })
        .into(),
    )?;

    for i in (0..10).rev() {
        println!("tick... {}", i);
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    node1.drop();
    node2.drop();

    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    exec().unwrap();
}
