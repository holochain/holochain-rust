//! check that n3h is in the path, or download it

use crate::{connection::NetResult, tweetlog::TWEETLOG};
use sha2::Digest;
use std::io::Write;

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

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

static N3H_PIN: &'static str = include_str!("n3h_pin.json");

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Artifact {
    pub url: String,
    pub file: String,
    pub hash: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Type {
    pub appimage: Option<Artifact>,
    pub tar: Option<Artifact>,
    pub dmg: Option<Artifact>,
    pub exe: Option<Artifact>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Arch {
    pub ia32: Option<Type>,
    pub x64: Option<Type>,
    pub arm: Option<Type>,
    pub arm64: Option<Type>,
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

static NIX_EXTRACT: &'static [u8] = br#"
#! /usr/bin/env nix-shell
#! nix-shell -i sh -p coreutils
exec tar xf "$@"
"#;

/// check for n3h in the path. if so, check its version
/// if these aren't correct, download the pinned executable for our os/arch
/// verify the hash and version
/// return a pathbuf containing a runnable n3h executable path
pub fn get_verify_n3h() -> NetResult<(std::path::PathBuf, Vec<String>)> {
    let mut path = std::path::PathBuf::new();
    path.push("n3h");

    let res = check_n3h_version(&path);
    if res.is_ok() {
        return Ok((path, res?));
    }

    let mut path = std::path::PathBuf::new();

    let (os, arch, pkg_type) = get_os_arch()?;
    let artifact = get_artifact_info(os, arch, pkg_type)?;

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
        // on nix, we need some extra magic to make prebuilt binaries work

        let mut extract_path = bin_dir.clone();
        extract_path.push("n3h-nix-extract.sh");

        {
            let mut open_opts = std::fs::OpenOptions::new();
            open_opts.create(true).write(true);
            set_executable(&mut open_opts);
            let mut file = open_opts.open(&extract_path)?;
            file.write_all(NIX_EXTRACT)?;
        }

        exec_output(
            "nix-shell",
            vec![extract_path.as_os_str(), path.as_os_str()],
            &bin_dir,
            false,
        )?;
        let mut path = bin_dir.clone();
        path.push(&artifact.file[0..artifact.file.len() - 7]);
        path.push("n3h-nix.bash");
        path
    } else {
        path
    };

    let res = check_n3h_version(&path)?;
    Ok((path, res))
}

fn check_n3h_version(path: &std::path::PathBuf) -> NetResult<Vec<String>> {
    let res = sub_check_n3h_version(&path, &["--version"]);
    if res.is_ok() {
        return Ok(vec![]);
    } else {
        tlog_e!("{:?}", res);
        let res = sub_check_n3h_version(&path, &["--appimage-extract-and-run", "--version"]);
        if res.is_ok() {
            return Ok(vec!["--appimage-extract-and-run".to_string()]);
        } else {
            tlog_e!("{:?}", res);
            bail!("{:?}", res);
        }
    }
}

fn sub_check_n3h_version(path: &std::path::PathBuf, out_args: &[&str]) -> NetResult<bool> {
    let res = exec_output(path, out_args, ".", false);
    if res.is_ok() {
        let res = res.unwrap();
        let re = regex::Regex::new(r#"(?m)#\s+n3h\s+version:\s+"([^"]+)"\s+#$"#)?;

        let mut version = None;
        for m in re.captures_iter(&res) {
            version = Some(m[1].to_string());
        }

        if version.is_none() || version.as_ref().unwrap() != &N3H_INFO.version {
            bail!(
                "n3h version mismatch, expected: {}, got: {:?}",
                &N3H_INFO.version,
                version
            );
        }
        return Ok(true);
    }
    println!("response: {:?}", res);
    bail!(format!("n3h not found in path: {:?}", &path));
}

/// check our pinned n3h version urls / hashes for the current os/arch
fn get_artifact_info(os: &str, arch: &str, pkg_type: &str) -> NetResult<&'static Artifact> {
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
        "ia32" => &os.as_ref().unwrap().ia32,
        "x64" => &os.as_ref().unwrap().x64,
        "arm" => &os.as_ref().unwrap().arm,
        "arm64" => &os.as_ref().unwrap().arm64,
        _ => bail!("arch {} not available", arch),
    };
    if arch.is_none() {
        bail!("arch not available");
    }
    let pkg_type = match pkg_type {
        "appimage" => &arch.as_ref().unwrap().appimage,
        "tar" => &arch.as_ref().unwrap().tar,
        "dmg" => &arch.as_ref().unwrap().dmg,
        "exe" => &arch.as_ref().unwrap().exe,
        _ => bail!("pkg_type {} not available", pkg_type),
    };
    Ok(&pkg_type.as_ref().unwrap())
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
    cmd.args(args)
        .env("N3H_VERSION_EXIT", "1")
        .env("NO_CLEANUP", "1")
        .current_dir(dir);
    tlog_d!("EXEC: {:?}", cmd);
    let res = cmd
        .output()
        .map_err(|e| format_err!("Failed to execute {:?}: {:?}", cmd, e))?;
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
fn get_os_arch() -> NetResult<(&'static str, &'static str, &'static str)> {
    let linux_pkg_type = if std::env::var("NIX_STORE").is_ok() {
        "tar"
    } else {
        "appimage"
    };
    if cfg!(target_os = "linux") && cfg!(target_arch = "x86") {
        Ok(("linux", "ia32", linux_pkg_type))
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Ok(("linux", "x64", linux_pkg_type))
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "arm") {
        Ok(("linux", "arm", linux_pkg_type))
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        Ok(("linux", "arm64", linux_pkg_type))
    } else if cfg!(windows) && cfg!(target_arch = "x86_64") {
        Ok(("win", "x64", "exe"))
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        Ok(("mac", "x64", "dmg"))
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

#[cfg(windows)]
fn set_executable(_o: &mut std::fs::OpenOptions) {}

#[cfg(unix)]
fn set_executable(o: &mut std::fs::OpenOptions) {
    // make sure the file is executable
    o.mode(0o755);
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
        set_executable(&mut open_opts);
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

    /// XXX (david.b) - windows electron is super funky... even with
    /// --enable-logging, it seems to spawn a separate terminal output
    /// and we can't capture it as a sub-process
    #[test]
    #[cfg(not(windows))]
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
    #[cfg(windows)]
    fn it_checks_path_true() {
        exec_output("cmd", vec!["/C", "echo"], ".", false).unwrap();
    }

    #[test]
    #[cfg(not(windows))]
    fn it_checks_path_true() {
        exec_output("sh", vec!["-c", "exit"], ".", false).unwrap();
    }

    #[test]
    #[cfg(windows)]
    fn it_checks_path_false() {
        let args: Vec<&str> = Vec::new();
        exec_output("badcommand", args, ".", false).unwrap_err();
    }

    #[test]
    #[cfg(not(windows))]
    fn it_checks_path_false() {
        let args: Vec<&str> = Vec::new();
        exec_output("badcommand", args, ".", false).unwrap_err();
    }
}
