//! check that n3h is in the path, or download it

use crate::{connection::NetResult, tweetlog::TWEETLOG};
use sha2::Digest;

macro_rules! tlog_d {
    ($($arg:tt)+) => {
        log_dd!("get_verify_n3h", $($arg)+);
    }
}

macro_rules! tlog_e {
    ($($arg:tt)+) => {
        log_ee!("get_verify_n3h", $($arg)+);
    }
}

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::OpenOptionsExt;

static N3H_PIN: &'static str = include_str!("n3h_pin.json");

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Artifact {
    pub url: String,
    pub file: String,
    pub hash: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Arch {
    pub aarch64: Option<Artifact>,
    pub x64: Option<Artifact>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Os {
    pub linux: Option<Arch>,
    pub mac: Option<Arch>,
    pub win: Option<Arch>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct N3hInfo {
    pub release: String,
    pub version: String,
    pub commitish: String,
    pub artifacts: Os,
}

lazy_static! {
    static ref N3H_INFO: N3hInfo =
        { serde_json::from_str(N3H_PIN).expect("bundled json should parse correctly") };
}

static NIX_MKRUNNER: &'static str = r#"cat > n3h-nix-runner.bash <<EOF
#! $(which bash)
exec $(which appimage-run) $(ls $(pwd)/n3h*.AppImage) "\$@"
EOF
chmod a+x n3h-nix-runner.bash
"#;

/// check for n3h in the path. if so, check its version
/// if these aren't correct, download the pinned executable for our os/arch
/// verify the hash and version
/// return a pathbuf containing a runnable n3h executable path
pub fn get_verify_n3h() -> NetResult<std::path::PathBuf> {
    let mut path = std::path::PathBuf::new();
    path.push("n3h");

    let res = check_n3h_version(&path);
    if res.is_ok() {
        return Ok(path);
    } else {
        tlog_e!("{:?}", res);
    }

    let mut path = std::path::PathBuf::new();

    let (os, arch) = get_os_arch()?;
    let artifact = get_artifact_info(os, arch)?;

    path.push(crate::holochain_common::paths::n3h_binaries_directory());
    path.push(&N3H_INFO.version);

    let bin_dir = path.clone();

    std::fs::create_dir_all(&path).expect("could not create n3h-binaries directory");
    path.push(&artifact.file);

    download(path.as_os_str(), &artifact.url, &artifact.hash)?;

    let path = if os == "mac" {
        // we need to extract the dmg into n3h.app
        extract_dmg(path.as_os_str(), &bin_dir)?
    } else if os == "linux" && std::env::var("NIX_STORE").is_ok() {
        // on nix, we need some extra appimage-run magic
        exec_output("bash", vec!["-c", NIX_MKRUNNER], &bin_dir, false)?;
        let mut path = bin_dir.clone();
        path.push("n3h-nix-runner.bash");
        path
    } else {
        path
    };

    check_n3h_version(&path)?;

    Ok(path)
}

fn check_n3h_version(path: &std::path::PathBuf) -> NetResult<bool> {
    let res = exec_output(path, vec!["--version"], ".", false);
    if res.is_ok() {
        let res = res.unwrap();
        let res = res.rsplit('|').next().unwrap_or("");
        if res != &N3H_INFO.version {
            bail!(
                "n3h version mismatch, expected: {}, got: {}",
                &N3H_INFO.version,
                res
            );
        }
        return Ok(true);
    }
    bail!("n3h not found in path");
}

/// check our pinned n3h version urls / hashes for the current os/arch
fn get_artifact_info(os: &str, arch: &str) -> NetResult<&'static Artifact> {
    let os = match os {
        "linux" => &N3H_INFO.artifacts.linux,
        "mac" => &N3H_INFO.artifacts.mac,
        "win" => &N3H_INFO.artifacts.win,
        _ => bail!("os {} not available", os),
    };
    if os.is_none() {
        bail!("os not available");
    }
    let arch = match arch {
        "aarch64" => &os.as_ref().unwrap().aarch64,
        "x64" => &os.as_ref().unwrap().x64,
        _ => bail!("arch {} not available", arch),
    };
    if arch.is_none() {
        bail!("arch not available");
    }
    Ok(&arch.as_ref().unwrap())
}

/// run a command / capture the output
fn exec_output<P, S1, I, S2>(cmd: S1, args: I, dir: P, ignore_errors: bool) -> NetResult<String>
where
    P: AsRef<std::path::Path>,
    S1: AsRef<std::ffi::OsStr>,
    I: IntoIterator<Item = S2>,
    S2: AsRef<std::ffi::OsStr>,
{
    let mut cmd = std::process::Command::new(cmd);
    cmd.args(args).env("N3H_VERSION_EXIT", "1").current_dir(dir);
    tlog_d!("EXEC: {:?}", cmd);
    let res = cmd.output()?;
    if !ignore_errors && !res.status.success() {
        bail!(
            "bad exit {:?} {:?}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&res.stdout).trim().to_string())
}

/// get the current os / arch
fn get_os_arch() -> NetResult<(&'static str, &'static str)> {
    if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Ok(("linux", "x64"))
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        Ok(("linux", "aarch64"))
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        Ok(("win", "x64"))
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        Ok(("mac", "x64"))
    } else {
        bail!("no prebuilt n3h for current os/arch - TODO check for node/npm - dl release zip");
    }
}

/// extract a dmg archive
fn extract_dmg(file: &std::ffi::OsStr, dest: &std::path::PathBuf) -> NetResult<std::path::PathBuf> {
    let mut dest = dest.clone();
    dest.push("n3h.app");
    if !dest.exists() {
        let res = exec_output(
            "hdiutil",
            vec![
                "attach",
                "-mountpoint",
                "./dmg-mount",
                &file.to_string_lossy(),
            ],
            ".",
            false,
        )?;
        tlog_d!("{}", res);
        exec_output(
            "cp",
            vec!["-a", "./dmg-mount/n3h.app", &dest.to_string_lossy()],
            ".",
            true,
        )?;
        let res = exec_output("hdiutil", vec!["detach", "./dmg-mount"], ".", false)?;
        tlog_d!("{}", res);
    }
    dest.push("Contents");
    dest.push("MacOS");
    dest.push("n3h");
    Ok(dest)
}

/// hash a file && compare to expected hash
fn check_hash(file: &std::ffi::OsStr, sha256: &str) -> bool {
    let mut file = match std::fs::File::open(file) {
        Err(_) => return false,
        Ok(v) => v,
    };

    let mut hash = sha2::Sha256::new();

    if std::io::copy(&mut file, &mut hash).is_err() {
        return false;
    }

    let hash = format!("{:x}", hash.result());

    if &hash != sha256 {
        tlog_e!("bad hash, expected {}, got {}", sha256, &hash);
        return false;
    }

    true
}

/// 1 - if file exists - check compare its hash
/// 2 - if file doesn't exist, or hash check fails, download it
/// 3 - compare downloaded file's hash
fn download(dest: &std::ffi::OsStr, url: &str, sha256: &str) -> NetResult<()> {
    if check_hash(dest, sha256) {
        return Ok(());
    }
    {
        tlog_d!("downloading {}...", url);
        let mut open_opts = std::fs::OpenOptions::new();
        open_opts.create(true).write(true);
        if cfg!(not(target_os = "windows")) {
            // make sure the file is executable
            open_opts.mode(0o755);
        }
        let mut file = open_opts.open(dest)?;
        let mut res = reqwest::get(url)?;
        res.copy_to(&mut file)?;
    }
    if !check_hash(dest, sha256) {
        bail!("bad download, hash mismatch ({:?})", dest);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_dl_n3h() {
        {
            let mut tweetlog = TWEETLOG.write().unwrap();
            tweetlog.set(crate::tweetlog::LogLevel::from('t'), None);
            tweetlog.listen(crate::tweetlog::Tweetlog::console);
        }

        get_verify_n3h().unwrap();
        // run again to verify the hash on an existing file
        get_verify_n3h().unwrap();
    }

    #[test]
    fn it_downloads() {
        let dir = tempfile::tempdir().unwrap();
        download(dir.path().join("n3h").as_os_str(), "https://github.com/holochain/node-static-build/releases/download/deps-2019-03-12/node-v8.15.1-linux-aarch64-alpha6.sha256", "b3f5a2f88ddbdcb642caa272afed7bbfbb283189cfa719a401ac8685b890e553").unwrap();
    }

    #[test]
    fn it_downloads_bad_hash() {
        let dir = tempfile::tempdir().unwrap();
        download(dir.path().join("n3h").as_os_str(), "https://github.com/holochain/node-static-build/releases/download/deps-2019-03-12/node-v8.15.1-linux-aarch64-alpha6.sha256", "baaaad").unwrap_err();
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn it_checks_path_true() {
        exec_output("cmd", vec!["/C", "echo"], ".", false).unwrap();
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn it_checks_path_true() {
        exec_output("sh", vec!["-c", "exit"], ".", false).unwrap();
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn it_checks_path_false() {
        let args: Vec<&str> = Vec::new();
        exec_output("badcommand", args, ".", false).unwrap_err();
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn it_checks_path_false() {
        let args: Vec<&str> = Vec::new();
        exec_output("badcommand", args, ".", false).unwrap_err();
    }
}
