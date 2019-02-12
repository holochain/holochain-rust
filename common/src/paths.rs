use std::path::PathBuf;

pub const QUALIFIER: &'static str = "org";
pub const ORGANIZATION: &'static str = "holochain";
pub const APPLICATION: &'static str = "holochain";
pub const KEYS_DIRECTORY: &'static str = "keys";

/// Returns the path to the root config directory for all of Holochain.
/// If we can get a user directory it will be an XDG compliant path
/// like "/home/peter/.config/holochain".
/// If it can't get a user directory it will default to "/etc/holochain".
pub fn config_root() -> PathBuf {
    directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.config_dir().to_owned())
        .unwrap_or_else(|| PathBuf::from("/etc").join(APPLICATION))
}

/// Returns the path to where agent keys are stored and looked for by default.
/// Something like "~/.config/holochain/keys".
pub fn keys_directory() -> PathBuf {
    config_root().join(KEYS_DIRECTORY)
}
