#![feature(try_from)]

extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;

use holochain_net_connection::{
    net_connection::NetConnection,
    protocol::Protocol,
    protocol_wrapper::{ConnectData, ProtocolWrapper, SendMessageData, SendResultData},
    NetResult,
};

use holochain_net::p2p_network::P2pNetwork;

use std::{convert::TryFrom, sync::mpsc};

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn usage() {
    println!("Usage: test_bin_ipc <ipc_uri>");
    std::process::exit(1);
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn exec() -> NetResult<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        usage();
    }

    let ipc_uri = args[1].clone();

    if ipc_uri == "" {
        usage();
    }

    println!("testing against uri: {}", ipc_uri);

    // use a mpsc channel for messaging
    let (sender, receiver) = mpsc::channel::<Protocol>();

    // create a new ipc P2pNetwork instance
    let mut con = P2pNetwork::new(
        Box::new(move |r| {
            sender.send(r?)?;
            Ok(())
        }),
        &json!({
            "backend": "ipc",
            "config": {
                "socketType": "zmq",
                "ipcUri": ipc_uri,
            }
        })
        .into(),
    )?;

    let mut id = "".to_string();
    let mut addr = "".to_string();

    // loop until we get a p2p ready message && record our
    // transport identifier and binding address
    loop {
        let z = receiver.recv()?;

        if let Ok(wrap) = ProtocolWrapper::try_from(&z) {
            match wrap {
                ProtocolWrapper::State(s) => {
                    id = s.id;
                    if !s.bindings.is_empty() {
                        addr = s.bindings[0].clone();
                    }
                }
                _ => (),
            }
        }

        if let Protocol::P2pReady = z {
            println!("p2p ready!!");
            break;
        }
    }

    println!("id: {}, addr: {}", id, addr);

    // send a message to connect to ourselves (just for debug / test)
    con.send(
        ProtocolWrapper::Connect(ConnectData {
            address: addr.clone(),
        })
        .into(),
    )?;

    // loop waiting for the message
    loop {
        let z = receiver.recv()?;

        if let Ok(wrap) = ProtocolWrapper::try_from(&z) {
            match wrap {
                ProtocolWrapper::PeerConnected(p) => {
                    println!("got peer connected: {}", p.id);
                    break;
                }
                _ => (),
            }
        }

        println!("got: {:?}", z);
    }

    // now, let's send a message to ourselves (just for debug / test)
    con.send(
        ProtocolWrapper::SendMessage(SendMessageData {
            msg_id: "unique-id".to_string(),
            to_address: id.clone(),
            data: json!("test data"),
        })
        .into(),
    )?;

    let handle_data;

    // loop waiting for the request to handle a message
    loop {
        let z = receiver.recv()?;

        if let Ok(wrap) = ProtocolWrapper::try_from(&z) {
            match wrap {
                ProtocolWrapper::HandleSend(m) => {
                    handle_data = m;
                    break;
                }
                _ => (),
            }
        }

        println!("got: {:?}", z);
    }

    println!("got handleSend: {:?}", handle_data);

    // hey, we got a message from ourselves!
    // let's send ourselves a response
    con.send(
        ProtocolWrapper::HandleSendResult(SendResultData {
            msg_id: handle_data.msg_id,
            data: json!(format!("echo: {}", handle_data.data)),
        })
        .into(),
    )?;

    // wait for the response to our original message
    loop {
        let z = receiver.recv()?;
        println!("got: {:?}", z);

        if let Ok(wrap) = ProtocolWrapper::try_from(&z) {
            match wrap {
                ProtocolWrapper::SendResult(m) => {
                    println!("Got Result! : {:?}", m);

                    assert_eq!(
                        "echo: \"test data\"".to_string(),
                        m.data.as_str().unwrap().to_string(),
                    );

                    break;
                }
                _ => (),
            }
        }

        println!("got: {:?}", z);
    }

    // yay, everything worked
    println!("test complete");

    // shut down the P2pNetwork instance
    con.stop()?;

    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    exec().unwrap();
}
