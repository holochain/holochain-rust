use std::path::PathBuf;

pub const CONFIG_DIRECTORY: &'static str = "holochain";
pub const KEYS_DIRECTORY: &'static str = "keys";

/// Returns the path to the root config directory for all of Holochain.
/// If we can get a user directory it will be a dot-directory in ~, like "/home/peter/.holochain".
/// If it can't get a user directory it will default to /etc, i.e. "/etc/holochain".
pub fn config_root() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|user_dirs| Some(user_dirs.home_dir().join(format!(".{}", CONFIG_DIRECTORY))))
        .or(Some(PathBuf::new().join("/etc").join(CONFIG_DIRECTORY)))
        .unwrap()
}

/// Returns the path to where agent keys are stored and looked for by default.
/// Something like "~/.holochain/keys".
pub fn keys_directory() -> PathBuf {
    config_root().join(KEYS_DIRECTORY)
}