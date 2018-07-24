extern crate libc;
extern crate net_ipc;
#[macro_use]
extern crate failure;

use net_ipc::IpcClient;

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
fn it_can_send_and_call() {
    let mut n3h_server = prep();
    println!("n3h_server pid: {}", n3h_server.id());

    let mut cli = IpcClient::new().unwrap();
    cli.connect("ipc://echo-server.sock").unwrap();

    let did_send = std::sync::Arc::new(std::sync::Mutex::new(false));
    let did_call = std::sync::Arc::new(std::sync::Mutex::new(false));
    let did_call_resp = std::sync::Arc::new(std::sync::Mutex::new(false));

    let did_send_oth = did_send.clone();
    let did_call_oth = did_call.clone();
    let did_call_resp_oth = did_call_resp.clone();

    cli.send(
        b"ab12",
        b"hello:send",
        Box::new(move |r| {
            match did_send_oth.lock() {
                Ok(mut s) => *s = true,
                Err(_) => bail!("brains"),
            }
            println!("send result: {:?}", r);
            Ok(())
        }),
    ).unwrap();
    cli.call(
        b"ab12",
        b"hello:call",
        Box::new(move |r| {
            match did_call_oth.lock() {
                Ok(mut s) => *s = true,
                Err(_) => bail!("brains"),
            }
            println!("call result: {:?}", r);
            Ok(())
        }),
        Box::new(move |r| {
            match did_call_resp_oth.lock() {
                Ok(mut s) => *s = true,
                Err(_) => bail!("brains"),
            }
            println!("call resp result: {:?}", r);
            Ok(())
        }),
    ).unwrap();
    loop {
        let msg = cli.process(1000).unwrap();
        println!(
            "msg: {:?}, did_send: {:?}, did_call: {:?}, did_call_resp: {:?}",
            msg, did_send, did_call, did_call_resp
        );
        if *did_send.lock().unwrap() && *did_call.lock().unwrap() && *did_call_resp.lock().unwrap()
        {
            break;
        }
    }

    println!("attempting to kill echo server");
    unsafe {
        libc::kill(n3h_server.id() as i32, libc::SIGTERM);
    }
    n3h_server.wait().unwrap();
    println!("echo server off");

    println!("attempting to kill zeromq context");
    cli.close().unwrap();
    net_ipc::context::destroy().unwrap();
    println!("zemomq is off");
}
