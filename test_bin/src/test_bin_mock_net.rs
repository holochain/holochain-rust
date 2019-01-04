#![feature(try_from)]

extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;

use holochain_net_connection::{
    net_connection::NetConnection,
    protocol::Protocol,
    protocol_wrapper::{MessageData, ProtocolWrapper, TrackAppData},
    NetResult,
};

use holochain_net::{p2p_config::P2pConfig, p2p_network::P2pNetwork};

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
    let mut con1 = P2pNetwork::new(
        Box::new(move |r| {
            sender1.send(r?)?;
            Ok(())
        }),
        &P2pConfig::default_mock("TODO give unique string"),
    )?;

    let (sender2, receiver2) = mpsc::channel::<Protocol>();

    let mut con2 = P2pNetwork::new(
        Box::new(move |r| {
            sender2.send(r?)?;
            Ok(())
        }),
        &P2pConfig::default_mock("TODO give unique string"),
    )?;

    con1.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_address: "sandwich".into(),
            agent_id: "node-1".to_string(),
        })
        .into(),
    )?;

    con2.send(
        ProtocolWrapper::TrackApp(TrackAppData {
            dna_address: "sandwich".into(),
            agent_id: "node-2".to_string(),
        })
        .into(),
    )?;

    con1.send(
        ProtocolWrapper::SendMessage(MessageData {
            dna_address: "sandwich".into(),
            to_agent_id: "node-2".to_string(),
            from_agent_id: "node-1".to_string(),
            msg_id: "yada".to_string(),
            data: json!("hello"),
        })
        .into(),
    )?;

    let res = ProtocolWrapper::try_from(receiver2.recv()?)?;
    println!("got: {:?}", res);

    if let ProtocolWrapper::HandleSend(msg) = res {
        con2.send(
            ProtocolWrapper::HandleSendResult(MessageData {
                dna_address: "sandwich".into(),
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

    let res = ProtocolWrapper::try_from(receiver1.recv()?)?;
    println!("got: {:?}", res);

    if let ProtocolWrapper::SendResult(msg) = res {
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
