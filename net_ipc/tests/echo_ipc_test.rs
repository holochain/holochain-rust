extern crate libc;
extern crate holochain_net_ipc as net_ipc;
#[macro_use]
extern crate failure;

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
            .current_dir("./tests/n3h")
            .status()
            .expect("failed running npm install")
            .success(),
        "failed running npm install"
    );
    std::process::Command::new("node")
        .args(&["./tests/n3h/examples/ipc/echo-server.js"])
        .spawn()
        .expect("failed running npm install")
}

#[test]
fn it_can_send_call_and_call_resp() {
    let mut n3h_server = prep();
    println!("n3h_server pid: {}", n3h_server.id());

    let message_id: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    let cli = Arc::new(Mutex::new(ZmqIpcClient::new().unwrap()));
    cli.lock()
        .unwrap()
        .connect("ipc://echo-server.sock")
        .unwrap();
    //cli.lock().unwrap().connect("ipc:///home/neonphog/projects/n3h/echo-server.sock").unwrap();

    {
        let cli_clone = cli.clone();
        let fu = |mut done: Box<FnMut(Result<Option<net_ipc::message::Message>>) -> bool>| loop {
            let msg = cli_clone.lock().unwrap().process(10);
            if done(msg) {
                break;
            }
        };

        let state: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

        println!("# TRY SET FAIL");

        let state_clone = state.clone();
        cli.lock()
            .unwrap()
            .send(
                b"",
                b"$$ctrl$$:FAIL",
                Some(Box::new(move |r| {
                    if let Err(_r) = r {
                        panic!("failed to set echo server MODE");
                    }
                    state_clone
                        .lock()
                        .unwrap()
                        .insert("send_result".to_string());
                    Ok(())
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        fu(Box::new(move |_msg| state_clone.lock().unwrap().len() >= 1));

        state.lock().unwrap().clear();

        println!("# SET FAIL - success");

        println!("# - sending a `send`... it should fail - #");

        let state_clone = state.clone();
        cli.lock()
            .unwrap()
            .send(
                b"",
                b"test",
                Some(Box::new(move |r| {
                    if let Ok(r) = r {
                        panic!("expected error, but got success: {:?}", r);
                    }
                    state_clone
                        .lock()
                        .unwrap()
                        .insert("send_result".to_string());
                    Ok(())
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        fu(Box::new(move |msg| {
            if let Ok(msg) = msg {
                panic!("expected error, but got success: {:?}", msg);
            }
            state_clone
                .lock()
                .unwrap()
                .insert("send_result_msg".to_string());
            state_clone.lock().unwrap().len() >= 2
        }));

        state.lock().unwrap().clear();

        println!("# - sending a `call`... it should fail - #");

        let state_clone = state.clone();
        let state_clone2 = state.clone();
        cli.lock()
            .unwrap()
            .call(
                b"",
                b"test",
                Some(Box::new(move |r| {
                    if let Ok(r) = r {
                        panic!("expected error, but got success: {:?}", r);
                    }
                    state_clone
                        .lock()
                        .unwrap()
                        .insert("call_result".to_string());
                    Ok(())
                })),
                Some(Box::new(move |r| {
                    if let Ok(r) = r {
                        panic!("expected error, but got success: {:?}", r);
                    }
                    state_clone2
                        .lock()
                        .unwrap()
                        .insert("call_resp_result".to_string());
                    Ok(())
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        fu(Box::new(move |msg| {
            if let Ok(msg) = msg {
                if let None = msg {
                    return false;
                }
                panic!("expected error, but got success: {:?}", msg);
            }
            state_clone
                .lock()
                .unwrap()
                .insert("call_result_msg".to_string());
            state_clone.lock().unwrap().len() >= 3
        }));

        state.lock().unwrap().clear();

        println!("# - sending a `call_resp`... it should fail - #");

        let state_clone = state.clone();
        cli.lock()
            .unwrap()
            .call_resp(
                b"",
                b"",
                b"test",
                Some(Box::new(move |r| {
                    if let Ok(r) = r {
                        panic!("expected error, but got success: {:?}", r);
                    }
                    state_clone
                        .lock()
                        .unwrap()
                        .insert("call_resp_result".to_string());
                    Ok(())
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        fu(Box::new(move |msg| {
            if let Ok(msg) = msg {
                panic!("expected error, but got success: {:?}", msg);
            }
            state_clone
                .lock()
                .unwrap()
                .insert("call_resp_result_msg".to_string());
            state_clone.lock().unwrap().len() >= 2
        }));

        state.lock().unwrap().clear();

        println!("# TRY SET ECHO");

        let state_clone = state.clone();
        cli.lock()
            .unwrap()
            .send(
                b"",
                b"$$ctrl$$:ECHO",
                Some(Box::new(move |r| {
                    if let Err(_r) = r {
                        panic!("failed to set echo server MODE");
                    }
                    state_clone
                        .lock()
                        .unwrap()
                        .insert("send_result".to_string());
                    Ok(())
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        fu(Box::new(move |_msg| state_clone.lock().unwrap().len() >= 1));

        state.lock().unwrap().clear();

        println!("# SET ECHO - success");

        println!("# - sending a `send`... it should succeed - #");

        let state_clone = state.clone();
        cli.lock()
            .unwrap()
            .send(
                b"",
                b"test",
                Some(Box::new(move |r| {
                    if let Err(r) = r {
                        panic!("erroneous error: {:?}", r);
                    }
                    state_clone
                        .lock()
                        .unwrap()
                        .insert("send_result".to_string());
                    Ok(())
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        fu(Box::new(move |msg| {
            let msg = match msg {
                Err(e) => panic!("erroneous error: {:?}", e),
                Ok(v) => v,
            };
            match msg {
                Some(v) => match v {
                    net_ipc::message::Message::SrvRespOk(_s) => {
                        state_clone
                            .lock()
                            .unwrap()
                            .insert("send_result_msg".to_string());
                        state_clone.lock().unwrap().len() >= 3
                    }
                    net_ipc::message::Message::SrvRecvSend(_s) => {
                        state_clone.lock().unwrap().insert("send_echo".to_string());
                        state_clone.lock().unwrap().len() >= 3
                    }
                    _ => false,
                },
                None => false,
            }
        }));

        state.lock().unwrap().clear();

        println!("# - sending a `call`... it should succeed - #");

        let state_clone = state.clone();
        let state_clone2 = state.clone();
        let message_id_clone = message_id.clone();
        cli.lock()
            .unwrap()
            .call(
                b"",
                b"test",
                Some(Box::new(move |r| {
                    if let Err(r) = r {
                        panic!("erroneous error: {:?}", r);
                    } else if let Ok(mut r) = r {
                        message_id_clone.lock().unwrap().clear();
                        message_id_clone.lock().unwrap().append(&mut r.0);
                    }
                    state_clone
                        .lock()
                        .unwrap()
                        .insert("call_result".to_string());
                    Ok(())
                })),
                Some(Box::new(move |r| {
                    if let Err(r) = r {
                        panic!("erroneous error: {:?}", r);
                    }
                    state_clone2
                        .lock()
                        .unwrap()
                        .insert("HANDLED-LATER".to_string());
                    Ok(())
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        fu(Box::new(move |msg| {
            let msg = match msg {
                Err(e) => panic!("erroneous error: {:?}", e),
                Ok(v) => v,
            };
            match msg {
                Some(v) => match v {
                    net_ipc::message::Message::SrvRespOk(_s) => {
                        state_clone
                            .lock()
                            .unwrap()
                            .insert("call_result_msg".to_string());
                        state_clone.lock().unwrap().len() >= 3
                    }
                    net_ipc::message::Message::SrvRecvCall(_s) => {
                        state_clone.lock().unwrap().insert("call_echo".to_string());
                        state_clone.lock().unwrap().len() >= 3
                    }
                    _ => false,
                },
                None => false,
            }
        }));

        println!("got id: {:?}", &message_id.lock().unwrap().as_slice());

        state.lock().unwrap().clear();

        println!("# - sending a `call_resp`... it should succeed - #");

        let state_clone = state.clone();
        cli.lock()
            .unwrap()
            .call_resp(
                message_id.lock().unwrap().as_slice(),
                b"",
                b"test",
                Some(Box::new(move |r| {
                    if let Err(r) = r {
                        panic!("erroneous error: {:?}", r);
                    }
                    state_clone
                        .lock()
                        .unwrap()
                        .insert("call_resp_result".to_string());
                    Ok(())
                })),
            )
            .unwrap();

        let state_clone = state.clone();
        fu(Box::new(move |msg| {
            let msg = match msg {
                Err(e) => panic!("erroneous error: {:?}", e),
                Ok(v) => v,
            };
            match msg {
                Some(v) => match v {
                    net_ipc::message::Message::SrvRespOk(_s) => {
                        state_clone
                            .lock()
                            .unwrap()
                            .insert("call_resp_result_msg".to_string());
                        state_clone.lock().unwrap().len() >= 4
                    }
                    net_ipc::message::Message::SrvRecvCallResp(_s) => {
                        state_clone
                            .lock()
                            .unwrap()
                            .insert("call_resp_echo_echo".to_string());
                        state_clone.lock().unwrap().len() >= 4
                    }
                    _ => false,
                },
                None => false,
            }
        }));
    }

    println!("attempting to kill echo server");
    unsafe {
        libc::kill(n3h_server.id() as i32, libc::SIGTERM);
    }
    n3h_server.wait().unwrap();
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
