#![feature(try_from)]

extern crate holochain_core_types;
extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;
extern crate failure;

pub mod p2p_node;

use holochain_net_connection::{
    json_protocol::{JsonProtocol, MessageData, TrackDnaData},
    net_connection::NetSend,
    NetResult,
};
use p2p_node::P2pNode;

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn usage() {
    println!("Usage: test_memory_network");
    std::process::exit(1);
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn exec_memory_network_test() -> NetResult<()> {
    println!("Testing: exec_memory_network_test()");

    let mut alex = P2pNode::new_with_unique_memory_network("alex".to_string());
    let mut billy = P2pNode::new_with_config("billy".to_string(), &alex.config, None);

    alex.send(
        JsonProtocol::TrackDna(TrackDnaData {
            dna_address: "sandwich".into(),
            agent_id: "node-1".to_string(),
        })
        .into(),
    )
    .expect("Failed sending TrackDnaData on node_a");
    billy
        .send(
            JsonProtocol::TrackDna(TrackDnaData {
                dna_address: "sandwich".into(),
                agent_id: "node-2".to_string(),
            })
            .into(),
        )
        .expect("Failed sending TrackDnaData on node_b");

    alex.send(
        JsonProtocol::SendMessage(MessageData {
            dna_address: "sandwich".into(),
            from_agent_id: "node-1".to_string(),
            to_agent_id: "node-2".to_string(),
            request_id: "yada".to_string(),
            content: json!("hello"),
        })
        .into(),
    )
    .expect("Failed sending message to node_b");
    let res = billy
        .wait(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))))
        .unwrap();
    println!("got: {:?}", res);

    if let JsonProtocol::HandleSendMessage(msg) = res {
        billy
            .send(
                JsonProtocol::HandleSendMessageResult(MessageData {
                    dna_address: "sandwich".into(),
                    from_agent_id: "node-2".to_string(),
                    to_agent_id: "node-1".to_string(),
                    request_id: "yada".to_string(),
                    content: json!(format!("echo: {}", msg.data.to_string())),
                })
                .into(),
            )
            .expect("Failed sending HandleSendMessageResult on node_b");;
    } else {
        panic!("bad generic msg");
    }

    let res = alex
        .wait(Box::new(one_is!(JsonProtocol::SendMessageResult(_))))
        .unwrap();
    println!("got response: {:?}", res);

    if let JsonProtocol::SendMessageResult(msg) = res {
        assert_eq!(
            "\"echo: \\\"hello\\\"\"".to_string(),
            msg.content.to_string()
        );
    } else {
        panic!("bad msg");
    }

    // yay, everything worked
    println!("test complete");

    // shut down the nodes
    alex.stop();
    billy.stop();

    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 1 {
        usage();
    }
    let res = exec_memory_network_test();
    assert!(res.is_ok());

    // Wait a bit before closing
    for i in (0..4).rev() {
        println!("tick... {}", i);
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
