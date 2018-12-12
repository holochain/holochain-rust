#![feature(try_from)]

extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;

use holochain_net_connection::{
    net_connection::NetConnection,
    protocol::Protocol,
    protocol_wrapper::{MessageData, P2pProtocol, TrackAppData},
    NetResult,
};

use holochain_net::p2p_network::P2pNetworkNode;

use std::{convert::TryFrom, sync::mpsc};

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn usage() {
    println!("Usage: test_bin_mock_net");
    std::process::exit(1);
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn exec() -> NetResult<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 1 {
        usage();
    }

    // use a mpsc channel for messaging
    let (sender1, receiver1) = mpsc::channel::<Protocol>();

    // create a new ipc P2pNetwork instance
    let mut con1 = P2pNetworkNode::new(
        Box::new(move |r| {
            sender1.send(r?)?;
            Ok(())
        }),
        &json!({
            "backend": "mock"
        })
        .into(),
    )?;

    let (sender2, receiver2) = mpsc::channel::<Protocol>();

    let mut con2 = P2pNetworkNode::new(
        Box::new(move |r| {
            sender2.send(r?)?;
            Ok(())
        }),
        &json!({
            "backend": "mock"
        })
        .into(),
    )?;

    con1.send(
        P2pProtocol::TrackApp(TrackAppData {
            dna_hash: "sandwich".to_string(),
            agent_id: "node-1".to_string(),
        })
        .into(),
    )?;

    con2.send(
        P2pProtocol::TrackApp(TrackAppData {
            dna_hash: "sandwich".to_string(),
            agent_id: "node-2".to_string(),
        })
        .into(),
    )?;

    con1.send(
        P2pProtocol::SendMessage(MessageData {
            dna_hash: "sandwich".to_string(),
            to_agent_id: "node-2".to_string(),
            from_agent_id: "node-1".to_string(),
            msg_id: "yada".to_string(),
            data: json!("hello"),
        })
        .into(),
    )?;

    let res = P2pProtocol::try_from(receiver2.recv()?)?;
    println!("got: {:?}", res);

    if let P2pProtocol::HandleSend(msg) = res {
        con2.send(
            P2pProtocol::HandleSendResult(MessageData {
                dna_hash: "sandwich".to_string(),
                to_agent_id: "node-1".to_string(),
                from_agent_id: "node-2".to_string(),
                msg_id: "yada".to_string(),
                data: json!(format!("echo: {}", msg.data.to_string())),
            })
            .into(),
        )?;
    } else {
        panic!("bad msg");
    }

    let res = P2pProtocol::try_from(receiver1.recv()?)?;
    println!("got: {:?}", res);

    if let P2pProtocol::SendResult(msg) = res {
        assert_eq!("\"echo: \\\"hello\\\"\"".to_string(), msg.data.to_string());
    } else {
        panic!("bad msg");
    }

    // yay, everything worked
    println!("test complete");

    // shut down the P2pNetwork instance
    con1.stop()?;
    con2.stop()?;

    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    exec().unwrap();
}
