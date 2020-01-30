//! This is a helper function to manage spawning an IPC sub-process
//! This process is expected to output some specific messages on its stdout
//! that we can process to know its launch state

use crate::{
    NEW_RELIC_LICENSE_KEY,
    connection::{net_connection::NetShutdown, NetResult},
    tweetlog::TWEETLOG,
};

use super::n3h::get_verify_n3h;

use std::{
    collections::HashMap,
    io::{Read, Write},
};

pub struct SpawnResult {
    pub kill: NetShutdown,
    pub ipc_binding: String,
    pub p2p_bindings: Vec<String>,
}

pub const DEFAULT_TIMEOUT_MS: usize = 5000;

/// Spawn a holochain networking ipc sub-process
/// Will block for IPC connection until timeout_ms is reached.
/// Can also block for P2P connection
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_NET)]
pub fn ipc_spawn(
    work_dir: String,
    end_user_config: String,
    mut env: HashMap<String, String>,
    timeout_ms: usize,
    can_wait_for_p2p: bool,
) -> NetResult<SpawnResult> {
    let (n3h, n3h_args) = get_verify_n3h()?;

    env.insert("NO_CLEANUP".to_string(), "1".to_string());

    let mut child = std::process::Command::new(n3h);

    child
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .args(&n3h_args)
        .envs(&env)
        .current_dir(work_dir);

    let mut child = child.spawn()?;
    let mut real_pid = String::new();

    if let Some(ref mut child_stdin) = child.stdin {
        child_stdin.write(&end_user_config.into_bytes())?;
    }

    // close the pipe so the process can proceed
    child.stdin = None;

    let mut out = SpawnResult {
        kill: None,
        ipc_binding: String::new(),
        p2p_bindings: Vec::new(),
    };

    // PID
    let re_pid = regex::Regex::new("(?m)^#PID#:(.+)$")?;
    let re_pid_ready = regex::Regex::new("#PID-READY#")?;

    // transport info (uri) for connecting to the ipc socket
    let re_ipc = regex::Regex::new("(?m)^#IPC-BINDING#:(.+)$")?;

    // transport info (multiaddr) for any p2p interface bindings
    let re_p2p = regex::Regex::new("(?m)^#P2P-BINDING#:(.+)$")?;

    // the child process is ready for ipc connections
    let re_ipc_ready = regex::Regex::new("#IPC-READY#")?;

    // the child process is ready for p2p connections
    let re_p2p_ready = regex::Regex::new("#P2P-READY#")?;

    // we need to know when our child process is ready for IPC connections
    // and possibily P2P connections.
    // It will run some startup algorithms, and then output some binding
    // info on stdout and finally an `#IPC-READY#` message and a `#P2P-READY#` message.
    // collect the binding info, and proceed when `#IPC-READY#`,
    // and `#P2P-READY#` if `can_wait_for_p2p` is set
    if let Some(ref mut stdout) = child.stdout {
        let mut has_ipc = false;
        let mut has_pid = false;
        let mut has_p2p = !can_wait_for_p2p;
        let mut wait_ms = 0;
        let mut data: Vec<u8> = Vec::new();
        while !(has_ipc.clone() && has_p2p.clone()) {
            // read stdout
            let mut buf: [u8; 4096] = [0; 4096];
            let size = stdout.read(&mut buf)?;
            if size > 0 {
                data.extend_from_slice(&buf[..size]);
                let tmp = String::from_utf8_lossy(&data);

                // look for PID
                if !has_pid.clone() {
                    if re_pid_ready.is_match(&tmp) {
                        for m in re_pid.captures_iter(&tmp) {
                            real_pid = m[1].to_string();
                            break;
                        }
                        has_pid = true
                    }
                }

                // look for IPC-READY
                if !has_ipc.clone() {
                    if re_ipc_ready.is_match(&tmp) {
                        for m in re_ipc.captures_iter(&tmp) {
                            out.ipc_binding = m[1].to_string();
                            break;
                        }
                        has_ipc = true
                    }
                }
                // look for P2P-READY
                if !has_p2p.clone() {
                    if re_p2p_ready.is_match(&tmp) {
                        for m in re_p2p.captures_iter(&tmp) {
                            out.p2p_bindings.push(m[1].to_string());
                        }
                        has_p2p = true
                    }
                }
            }
            if wait_ms >= timeout_ms.clone() {
                bail!("ipc_spawn() timed out. N3H might need more time or something is dysfunctional in the network interface");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            wait_ms += 10;
        }
    } else {
        bail!("pipe fail");
    }

    // close the pipe since we can never read from it again...
    child.stdout = None;
    log_i!(
        "READY! {} {:?} [{}]",
        out.ipc_binding,
        out.p2p_bindings,
        real_pid
    );

    // Set shutdown function to kill the sub-process
    out.kill = Some(Box::new(move || {
        let mut wait_ms = 0;
        while wait_ms < 500 {
            match child.try_wait() {
                Ok(None) => {}
                Ok(Some(_status)) => return,
                Err(e) => {
                    log_e!("error attempting to wait: {}", e);
                    return;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            wait_ms += 10;
        }
        if term_child(child.id()) {
            return;
        }
        match child.kill() {
            Ok(()) => kill_child(&real_pid),
            Err(e) => println!("failed to kill ipc sub-process: {:?}", e),
        };
    }));

    Ok(out)
}

#[cfg(windows)]
fn term_child(_pid: u32) -> bool {
    false
}

#[cfg(not(windows))]
fn term_child(pid: u32) -> bool {
    unsafe {
        if libc::kill(pid as i32, libc::SIGTERM) == 0 {
            libc::waitpid(pid as i32, std::ptr::null_mut(), 0);
            return true;
        }
    }
    false
}

#[cfg(windows)]
fn kill_child(pid: &str) {
    let mut child_killer = std::process::Command::new("taskkill");
    child_killer.args(&["/pid", pid, "/f", "/t"]);
    let _ = child_killer.status();
}

#[cfg(not(windows))]
fn kill_child(_pid: &str) {}
