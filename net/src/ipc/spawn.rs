//! This is a helper function to manage spawning an IPC sub-process
//! This process is expected to output some specific messages on its stdout
//! that we can process to know its launch state

use crate::{
    connection::{net_connection::NetShutdown, NetResult},
    tweetlog::TWEETLOG,
};

use std::{
    collections::HashMap,
    io::{Read, Write},
};

use sha2::Digest;

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
    end_user_config: String,
    env: HashMap<String, String>,
    block_connect: bool,
) -> NetResult<SpawnResult> {
    let mut child = std::process::Command::new(cmd);

    child
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .args(&args)
        .envs(&env)
        .current_dir(work_dir);

    let mut child = child.spawn()?;

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

    // transport info (uri) for connecting to the ipc socket
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

    log_i!("READY! {} {:?}", out.ipc_binding, out.p2p_bindings);

    // Set shutdown function to kill the sub-process
    out.kill = Some(Box::new(move || {
        match child.kill() {
            Ok(()) => (),
            Err(e) => println!("failed to kill ipc sub-process: {:?}", e),
        };
    }));

    Ok(out)
}

/// check if a command is available in the path
fn is_in_path (cmd: &str) -> bool {
    let mut args: Vec<&str> = cmd.split(" ").collect();
    let cmd = args.remove(0);
    let res = std::process::Command::new(cmd)
        .args(args)
        .status();
    if res.is_err() {
        return false;
    }
    let res = res.unwrap().code();
    if res.is_none() {
        return false;
    }
    res.unwrap() == 0
}

fn get_verify_n3h() -> NetResult<std::path::PathBuf> {
    let mut path = std::path::PathBuf::new();
    if is_in_path("n3h --version") {
        path.push("n3h");
        return Ok(path);
    }
    let (url, hash) = get_n3h_info()?;
    path.push(".");
    path.push("n3h");
    println!("downloading {}...", url);
    download(path.as_os_str(), url, hash)?;
    Ok(path)
}

fn get_n3h_info() -> NetResult<(&'static str, &'static str)> {
    if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        return Ok((
            "https://github.com/holochain/node-static-build/releases/download/deps-2019-03-12/node-v8.15.1-linux-x64-alpha6",
            "1e31b2e916608218e09ef9ef9dc48eba0cc58557225f8d81020b7e1b33144cef"
        ))
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        return Ok((
            "https://github.com/holochain/node-static-build/releases/download/deps-2019-03-12/node-v8.15.1-linux-aarch64-alpha6",
            "baaad"
        ))
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        return Ok((
            "",
            "baaad"
        ))
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        return Ok((
            "",
            "baaad"
        ))
    } else {
        bail!("no prebuilt n3h for current os/arch - TODO check for node/npm - dl release zip");
    }
}

fn download (dest: &std::ffi::OsStr, url: &str, sha256: &str) -> NetResult<()> {
    {
        let mut file = std::fs::File::create(dest)?;
        let mut res = reqwest::get(url)?;
        res.copy_to(&mut file)?;
    }
    {
        let mut file = std::fs::File::open(dest)?;
        let mut hash = sha2::Sha256::new();
        std::io::copy(&mut file, &mut hash)?;
        let hash = format!("{:x}", hash.result());
        if &hash != sha256 {
            bail!("bad download, hash mismatch ({:?}:{}) was {}", dest, sha256, hash);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_dl_n3h () {
        let n3h = get_verify_n3h().unwrap();
    }

    #[test]
    fn it_downloads () {
        let dir = tempfile::tempdir().unwrap();
        download(dir.path().join("n3h").as_os_str(), "https://github.com/holochain/node-static-build/releases/download/deps-2019-03-12/node-v8.15.1-linux-aarch64-alpha6.sha256", "b3f5a2f88ddbdcb642caa272afed7bbfbb283189cfa719a401ac8685b890e553").unwrap();
    }

    #[test]
    fn it_downloads_bad_hash () {
        let dir = tempfile::tempdir().unwrap();
        download(dir.path().join("n3h").as_os_str(), "https://github.com/holochain/node-static-build/releases/download/deps-2019-03-12/node-v8.15.1-linux-aarch64-alpha6.sha256", "baaaad").unwrap_err();
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn it_checks_path_true () {
        assert_eq!(true, is_in_path("cmd /C echo"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn it_checks_path_true () {
        assert_eq!(true, is_in_path("sh -c exit"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn it_checks_path_false () {
        assert_eq!(false, is_in_path("badcommand"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn it_checks_path_false () {
        assert_eq!(false, is_in_path("badcommand"));
    }
}
