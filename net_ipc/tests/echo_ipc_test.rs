extern crate failure;
extern crate holochain_net_ipc as net_ipc;
extern crate libc;

use std::collections::HashSet;

use net_ipc::{errors::*, ZmqIpcClient};
use std::sync::{Arc, Mutex};

fn prep() -> std::process::Child {
    assert!(
        std::process::Command::new("git")
            .args(&["submodule", "update", "--init", "--recursive"])
            .status()
            .expect("failed updating git submodules")
            .success(),
        "failed updating git submodules"
    );
    assert!(
        std::process::Command::new("npm")
            .args(&["install", "--production"])
            .current_dir("./tests/node-p2p-ipc")
            .status()
            .expect("failed running npm install")
            .success(),
        "failed running npm install"
    );
    std::process::Command::new("node")
        .args(&["./tests/node-p2p-ipc/examples/echo-server.js"])
        .spawn()
        .expect("failed running the echo server")
}

#[test]
fn it_can_send_call_and_call_resp() {
    let mut node_ipc_server = prep();
    println!("node_ipc_server pid: {}", node_ipc_server.id());

    let cli = Arc::new(Mutex::new(ZmqIpcClient::new().unwrap()));
    cli.lock()
        .unwrap()
        .connect("ipc://echo-server.sock")
        .unwrap();

    {
        let cli_clone = cli.clone();
        let fu = |mut done: Box<FnMut(Result<net_ipc::message::Message>) -> bool>| loop {
            let msg = cli_clone.lock().unwrap().process(1000);
            if match msg {
                Err(e) => done(Err(e)),
                Ok(m) => match m {
                    Some(_m) => done(Ok(_m)),
                    None => panic!("Timeout awaiting data!")
                }
            } {
                break;
            }
        };

        let state: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

        println!("# try send `hello`");

        let state_clone = state.clone();
        cli.lock()
            .unwrap()
            .call(
                b"hello",
                Some(Box::new(move |r| {
                    if let Err(_r) = r {
                        panic!("failed to call hello");
                    }
                    state_clone
                        .lock()
                        .unwrap()
                        .insert(String::from_utf8_lossy(&r.unwrap().1).to_string());
                    Ok(())
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        fu(Box::new(move |msg| {
            let m = &msg.unwrap();
            match m {
                net_ipc::message::Message::Pong(_p) => {
                    return false;
                }
                net_ipc::message::Message::CallOk(_m) => {
                    let s = String::from_utf8_lossy(&_m.1);
                    if s != String::from("echo: hello") {
                        panic!("bad server message: {:?}", s);
                    }
                }
                _ => panic!("unexpected msg type: {:?}", m),
            }

            let count = state_clone.lock().unwrap().len();
            if count != 1 {
                panic!("bad state entry count: {:?}", count);
            }

            let r = state_clone.lock().unwrap().clone();
            if !r.contains(&String::from("echo: hello")) {
                panic!("bad server message: {:?}", r);
            }

            true
        }));

        state.lock().unwrap().clear();

        println!("# try send `hello` - success");

        println!("# try send `error`");

        let state_clone = state.clone();
        cli.lock()
            .unwrap()
            .call(
                b"error",
                Some(Box::new(move |r| {
                    if let Err(_r) = r {
                        let _r = format!("{}", _r);
                        if !_r.contains("Error: echo: error") {
                            panic!("bad error response: {:?}", _r);
                        }
                        state_clone.lock().unwrap().insert(_r);
                        return Ok(());
                    }
                    panic!("expected error, got: {:?}", r);
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        fu(Box::new(move |msg| {
            let msg = msg.expect_err("expected Err");
            let msg = format!("{}", msg);
            if !msg.contains("Error: echo: error") {
                panic!("bad error response: {:?}", msg);
            }

            let count = state_clone.lock().unwrap().len();
            if count != 1 {
                panic!("bad state entry count: {:?}", count);
            }

            let r = state_clone.lock().unwrap().clone();
            let r = format!("{:?}", r);
            if !r.contains("Error: echo: error") {
                panic!("bad server message: {:?}", r);
            }

            true
        }));

        state.lock().unwrap().clear();

        println!("# try send `error` - success");

        println!("# try send `call-hello`");

        let state_clone = state.clone();
        cli.lock()
            .unwrap()
            .call(
                b"call-hello",
                Some(Box::new(move |r| {
                    if let Err(_r) = r {
                        panic!("failed to call call-hello");
                    }
                    state_clone
                        .lock()
                        .unwrap()
                        .insert(String::from_utf8_lossy(&r.unwrap().1).to_string());
                    Ok(())
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        let cli_clone = cli.clone();
        fu(Box::new(move |msg| {
            let m = &msg.unwrap();
            match m {
                net_ipc::message::Message::Pong(_p) => {
                    return false;
                }
                net_ipc::message::Message::Call(_m) => {
                    let mut s = String::from_utf8_lossy(&_m.1).to_string();
                    s.insert_str(0, "echo: ");
                    println!("got call, about to respond: ({:?})", s);
                    cli_clone.lock().unwrap().respond(&_m.0, Ok(s.as_bytes()))
                        .unwrap();
                    return false;
                }
                net_ipc::message::Message::CallOk(_m) => {
                    let s = String::from_utf8_lossy(&_m.1);
                    if s != String::from("server successfully received `echo: srv-hello`") {
                        panic!("bad server message: {:?}", s);
                    }
                }
                _ => panic!("unexpected msg type: {:?}", m),
            }

            let count = state_clone.lock().unwrap().len();
            if count != 1 {
                panic!("bad state entry count: {:?}", count);
            }

            let r = state_clone.lock().unwrap().clone();
            if !r.contains(&String::from("server successfully received `echo: srv-hello`")) {
                panic!("bad server message: {:?}", r);
            }

            true
        }));

        state.lock().unwrap().clear();

        println!("# try send `call-hello` - success");

        println!("# try send `call-error`");

        let state_clone = state.clone();
        cli.lock()
            .unwrap()
            .call(
                b"call-error",
                Some(Box::new(move |r| {
                    if let Err(_r) = r {
                        panic!("failed to call call-error {:?}", _r);
                    }
                    state_clone
                        .lock()
                        .unwrap()
                        .insert(String::from_utf8_lossy(&r.unwrap().1).to_string());
                    Ok(())
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        let cli_clone = cli.clone();
        fu(Box::new(move |msg| {
            let m = &msg.unwrap();
            match m {
                net_ipc::message::Message::Pong(_p) => {
                    return false;
                }
                net_ipc::message::Message::Call(_m) => {
                    let mut s = String::from_utf8_lossy(&_m.1).to_string();
                    s.insert_str(0, "echo: ");
                    println!("got call, about to respond: ({:?})", s);
                    cli_clone.lock().unwrap().respond(&_m.0, Err(
                        IpcError::GenericError {
                            error: s
                        }.into()
                    )).unwrap();
                    return false;
                }
                net_ipc::message::Message::CallOk(_m) => {
                    let s = String::from_utf8_lossy(&_m.1);
                    if s != String::from("server successfully got error `Error: IpcError: echo: srv-error`") {
                        panic!("bad server message: {:?}", s);
                    }
                }
                _ => panic!("unexpected msg type: {:?}", m),
            }

            let count = state_clone.lock().unwrap().len();
            if count != 1 {
                panic!("bad state entry count: {:?}", count);
            }

            let r = state_clone.lock().unwrap().clone();
            if !r.contains(&String::from("server successfully got error `Error: IpcError: echo: srv-error`")) {
                panic!("bad server message: {:?}", r);
            }

            true
        }));

        state.lock().unwrap().clear();

        println!("# try send `call-error` - success");
    }

    println!("attempting to kill echo server");
    unsafe {
        libc::kill(node_ipc_server.id() as i32, libc::SIGTERM);
    }
    node_ipc_server.wait().unwrap();
    println!("echo server off");

    println!("attempting to kill zeromq context");
    match Arc::try_unwrap(cli) {
        Ok(cli) => {
            let cli = cli.into_inner().unwrap();
            cli.close().unwrap();
        }
        _ => panic!("couldn't un-Arc"),
    }
    ZmqIpcClient::destroy_context().unwrap();
    println!("zemomq is off");
}
