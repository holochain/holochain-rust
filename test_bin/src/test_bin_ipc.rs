#![feature(try_from)]

extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;
extern crate tempfile;

use holochain_net_connection::{
    net_connection::NetConnection,
    protocol::Protocol,
    protocol_wrapper::{
        ConnectData, DhtData, DhtMetaData, GetDhtData, GetDhtMetaData, MessageData,
        ProtocolWrapper, TrackAppData,
    },
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
    pub fn stop(self) {
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
                    "cmd": "node",
                    "args": [
                        format!("{}/packages/n3h/bin/n3h", n3h_path)
                    ],
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

    let node1_state = node1.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;
    let node2_state = node2.wait(Box::new(one_is!(ProtocolWrapper::State(_))))?;

    let node1_id;
    //let node2_id;
    let node2_binding;

    one_let!(ProtocolWrapper::State(s) = node1_state {
        node1_id = s.id
    });

    one_let!(ProtocolWrapper::State(s) = node2_state {
        //node2_id = s.id;
        node2_binding = s.bindings[0].clone();
    });

    node1.con.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_hash: DNA_HASH.to_string(),
            agent_id: AGENT_1.to_string(),
        })
        .into(),
    )?;
    let connect_result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
    println!("self connected result 1: {:?}", connect_result_1);

    node2.con.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_hash: DNA_HASH.to_string(),
            agent_id: AGENT_2.to_string(),
        })
        .into(),
    )?;
    let connect_result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::PeerConnected(_))))?;
    println!("self connected result 2: {:?}", connect_result_2);

    println!("connect node1 ({}) to node2 ({})", node1_id, node2_binding);

    node1.con.send(
        ProtocolWrapper::Connect(ConnectData {
            address: node2_binding,
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

    node1.con.send(
        ProtocolWrapper::SendMessage(MessageData {
            msg_id: "test".to_string(),
            dna_hash: DNA_HASH.to_string(),
            to_agent_id: AGENT_2.to_string(),
            from_agent_id: AGENT_1.to_string(),
            data: json!("hello"),
        })
        .into(),
    )?;

    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::HandleSend(_))))?;
    println!("got handle send 2: {:?}", result_2);

    node2.con.send(
        ProtocolWrapper::HandleSendResult(MessageData {
            msg_id: "test".to_string(),
            dna_hash: DNA_HASH.to_string(),
            to_agent_id: AGENT_1.to_string(),
            from_agent_id: AGENT_2.to_string(),
            data: json!("echo: hello"),
        })
        .into(),
    )?;

    let result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::SendResult(_))))?;
    println!("got send result 1: {:?}", result_1);

    node1.con.send(
        ProtocolWrapper::PublishDht(DhtData {
            msg_id: "testPub".to_string(),
            dna_hash: DNA_HASH.to_string(),
            agent_id: AGENT_1.to_string(),
            address: "test_addr".to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;

    let result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::StoreDht(_))))?;
    println!("got store result 1: {:?}", result_1);

    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::StoreDht(_))))?;
    println!("got store result 2: {:?}", result_2);

    node2.con.send(
        ProtocolWrapper::GetDht(GetDhtData {
            msg_id: "testGet".to_string(),
            dna_hash: DNA_HASH.to_string(),
            from_agent_id: AGENT_2.to_string(),
            address: "test_addr".to_string(),
        })
        .into(),
    )?;

    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDht(_))))?;
    println!("got dht get: {:?}", result_2);

    node2.con.send(
        ProtocolWrapper::GetDhtResult(DhtData {
            msg_id: "testGetResult".to_string(),
            dna_hash: DNA_HASH.to_string(),
            agent_id: AGENT_1.to_string(),
            address: "test_addr".to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;

    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDhtResult(_))))?;
    println!("got dht get result: {:?}", result_2);

    node1.con.send(
        ProtocolWrapper::PublishDhtMeta(DhtMetaData {
            msg_id: "testPubMeta".to_string(),
            dna_hash: DNA_HASH.to_string(),
            agent_id: AGENT_1.to_string(),
            address: "test_addr_meta".to_string(),
            attribute: "link:yay".to_string(),
            content: json!("hello-meta"),
        })
        .into(),
    )?;

    let result_1 = node1.wait(Box::new(one_is!(ProtocolWrapper::StoreDhtMeta(_))))?;
    println!("got store meta result 1: {:?}", result_1);

    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::StoreDhtMeta(_))))?;
    println!("got store meta result 2: {:?}", result_2);

    node2.con.send(
        ProtocolWrapper::GetDhtMeta(GetDhtMetaData {
            msg_id: "testGetMeta".to_string(),
            dna_hash: DNA_HASH.to_string(),
            from_agent_id: AGENT_2.to_string(),
            address: "test_addr".to_string(),
            attribute: "link:yay".to_string(),
        })
        .into(),
    )?;

    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDhtMeta(_))))?;
    println!("got dht get: {:?}", result_2);

    node2.con.send(
        ProtocolWrapper::GetDhtMetaResult(DhtMetaData {
            msg_id: "testGetMetaResult".to_string(),
            dna_hash: DNA_HASH.to_string(),
            agent_id: AGENT_1.to_string(),
            address: "test_addr".to_string(),
            attribute: "link:yay".to_string(),
            content: json!("hello"),
        })
        .into(),
    )?;

    let result_2 = node2.wait(Box::new(one_is!(ProtocolWrapper::GetDhtMetaResult(_))))?;
    println!("got dht get result: {:?}", result_2);

    for i in (0..4).rev() {
        println!("tick... {}", i);
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    node1.stop();
    node2.stop();

    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    exec().unwrap();
}
