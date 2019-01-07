//! This is a helper function to manage spawning an IPC sub-process
//! This process is expected to output some specific messages on its stdout
//! that we can process to know its launch state

use holochain_net_connection::{net_connection::NetShutdown, NetResult};

use std::{collections::HashMap, io::Read};

pub struct SpawnResult {
    pub kill: NetShutdown,
    pub ipc_binding: String,
    pub p2p_bindings: Vec<String>,
}

/// spawn a holochain networking ipc sub-process
pub fn ipc_spawn(
    cmd: String,
    args: Vec<String>,
    work_dir: String,
    env: HashMap<String, String>,
    block_connect: bool,
) -> NetResult<SpawnResult> {
    let mut child = std::process::Command::new(cmd);

    child
        .stdout(std::process::Stdio::piped())
        .args(&args)
        .envs(&env)
        .current_dir(work_dir);

    println!("SPAWN ({:?})", child);

    let mut child = child.spawn()?;

    let mut out = SpawnResult {
        kill: None,
        ipc_binding: String::new(),
        p2p_bindings: Vec::new(),
    };

    // transport info (zmq uri) for connecting to the ipc socket
    let re_ipc = regex::Regex::new("(?m)^#IPC-BINDING#:(.+)$")?;

    // transport info (multiaddr) for any p2p interface bindings
    let re_p2p = regex::Regex::new("(?m)^#P2P-BINDING#:(.+)$")?;

    // the child process is ready for connections
    let re_ready = regex::Regex::new("#IPC-READY#")?;

    // we need to know when our child process is ready for IPC connections
    // it will run some startup algorithms, and then output some binding
    // info on stdout and finally a `#IPC-READY#` message.
    // collect the binding info, and proceed when `#IPC-READY#`
    if let Some(ref mut stdout) = child.stdout {
        let mut data: Vec<u8> = Vec::new();
        loop {
            let mut buf: [u8; 4096] = [0; 4096];
            let size = stdout.read(&mut buf)?;
            if size > 0 {
                data.extend_from_slice(&buf[..size]);

                let tmp = String::from_utf8_lossy(&data);
                if re_ready.is_match(&tmp) {
                    for m in re_ipc.captures_iter(&tmp) {
                        out.ipc_binding = m[1].to_string();
                        break;
                    }
                    for m in re_p2p.captures_iter(&tmp) {
                        out.p2p_bindings.push(m[1].to_string());
                    }
                    break;
                }
            } else {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            if !block_connect {
                break;
            }
        }
    } else {
        bail!("pipe fail");
    }

    // close the pipe since we can never read from it again...
    child.stdout = None;

    println!("READY! {} {:?}", out.ipc_binding, out.p2p_bindings);

    out.kill = Some(Box::new(move || {
        child.kill().expect("failed to kill ipc sub-process")
    }));

    Ok(out)
}
