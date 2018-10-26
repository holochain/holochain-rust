//! Excercises the IPC client against the nodejs example echo-server

extern crate failure;
extern crate holochain_net_ipc as net_ipc;
extern crate libc;

use net_ipc::{errors::*, ZmqIpcClient};
use std::sync::{Arc, Mutex};

/// do prep work and run the nodejs example echo-server.js
fn run_nodejs_echo_server() -> std::process::Child {
    // make sure the git submodule is initialized
    assert!(
        std::process::Command::new("git")
            .args(&["submodule", "update", "--init", "--recursive"])
            .status()
            .expect("failed updating git submodules")
            .success(),
        "failed updating git submodules"
    );

    // make sure the npm dependencies are installed
    assert!(
        std::process::Command::new("npm")
            .args(&["install", "--production"])
            .current_dir("./tests/node-p2p-ipc")
            .status()
            .expect("failed running npm install")
            .success(),
        "failed running npm install"
    );

    // spawn the actual echo-server process
    std::process::Command::new("node")
        .args(&["./tests/node-p2p-ipc/examples/echo-server.js"])
        .spawn()
        .expect("failed running the echo server")
}

/// struct to help hold context for the callbacks
struct TestFrame {
    srv: std::process::Child,
    cli: ZmqIpcClient,
}

impl TestFrame {
    /// create a new test frame
    fn new() -> TestFrame {
        let srv = run_nodejs_echo_server();

        let mut cli = ZmqIpcClient::new().unwrap();
        cli.connect("ipc://echo-server.sock").unwrap();

        TestFrame { srv, cli }
    }

    /// the ipc client has multiple methods for processing requests
    /// - (1) - callback when invoking the `call` method
    /// - (2) - wait for messages from `process()`
    /// this executor makes sure those paths are equivalent
    /// and translates all data (both successses and failures) to utf8 strings
    fn execute(&mut self, data: &[u8]) -> std::result::Result<String, String> {
        let cb_result: Arc<Mutex<Option<Result<Vec<u8>>>>> = Arc::new(Mutex::new(None));

        let cli_ref = &mut self.cli;

        let cb_result_clone = cb_result.clone();

        // make the actual `call`
        cli_ref
            .call(
                data,
                Some(Box::new(move |r| {
                    // store the cb result in our Arc variable
                    *cb_result_clone.lock().unwrap() = Some(r);
                    Ok(())
                })),
            )
            .unwrap();

        // loop until we get a result
        let msg = loop {
            match cli_ref.process(1000) {
                Err(e) => break Err(e),
                Ok(m) => match m {
                    Some(_m) => match _m {
                        net_ipc::message::Message::Pong(_p) => {
                            continue;
                        }
                        net_ipc::message::Message::CallOk(_r) => {
                            break Ok(_r.1);
                        }
                        net_ipc::message::Message::Call(_m) => {
                            let mut s = String::from_utf8_lossy(&_m.1).to_string();
                            if s == "srv-hello" {
                                println!("got srv-hello");
                                s.insert_str(0, "echo: ");
                                cli_ref.respond(&_m.0, Ok(s.as_bytes())).unwrap();
                            } else if s == "srv-error" {
                                println!("got srv-error");
                                s.insert_str(0, "echo: ");
                                cli_ref
                                    .respond(&_m.0, Err(IpcError::GenericError { error: s }.into()))
                                    .unwrap();
                            } else {
                                panic!("unexpected server call: {:?}", s);
                            }
                            continue;
                        }
                        _ => panic!("not handled"),
                    },
                    None => panic!("timeout wating for data"),
                },
            }
        };

        // de-arc the cb_result variable
        let cb_result = match Arc::try_unwrap(cb_result) {
            Ok(c) => c.into_inner().unwrap().unwrap(),
            _ => panic!("couldn't un-Arc"),
        };

        // if they are both errors - return a stringified error
        if let Err(ref cb_e) = cb_result {
            if let Err(ref msg_e) = msg {
                let cb_e = format!("{}", cb_e);
                let msg_e = format!("{}", msg_e);
                if cb_e == msg_e {
                    return Err(msg_e.lines().next().unwrap().to_string());
                }
            }
        }

        // if they are both successes - return a stringified success
        if let Ok(ref cb_o) = cb_result {
            if let Ok(ref msg_o) = msg {
                let cb_o = String::from_utf8_lossy(&cb_o).to_string();
                let msg_o = String::from_utf8_lossy(&msg_o).to_string();
                if cb_o == msg_o {
                    return Ok(msg_o);
                }
            }
        }

        // if they do not match, we need to panic!
        panic!(
            "callback and event did not match {:?} / {:?}",
            cb_result, msg
        );
    }

    /// cleanup both the nodejs echo-server and the ipc client connection
    fn destroy(mut self) {
        println!("attempting to kill echo server");
        unsafe {
            libc::kill(self.srv.id() as i32, libc::SIGTERM);
        }
        self.srv.wait().unwrap();
        println!("echo server off");

        println!("attempting to kill zeromq context");
        self.cli.close().unwrap();
        ZmqIpcClient::destroy_context().unwrap();
        println!("zemomq is off");
    }
}

#[test]
fn it_can_send_call_and_call_resp() {
    let mut frame = TestFrame::new();

    println!("### BEGIN TEST SUITE ###");

    println!("--- making `hello` call ---");
    assert_eq!(Ok("echo: hello".to_string()), frame.execute(b"hello"));

    println!("--- making `error` call ---");
    assert_eq!(
        Err("IpcError: Error: echo: error".to_string()),
        frame.execute(b"error")
    );

    println!("--- making `call-hello` call ---");
    assert_eq!(
        Ok("srv-got: `echo: srv-hello`".to_string()),
        frame.execute(b"call-hello")
    );

    println!("--- making `call-error` call ---");
    assert_eq!(
        Ok("srv-got: `Error: IpcError: echo: srv-error`".to_string()),
        frame.execute(b"call-error")
    );

    println!("### TEST SUITE FINISHED - CLEANUP ###");

    frame.destroy();

    println!("### END TEST SUITE ###");
}
