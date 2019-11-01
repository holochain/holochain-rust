use std::path::PathBuf;

pub const QUALIFIER: &str = "org";
pub const ORGANIZATION: &str = "holochain";
pub const APPLICATION: &str = "holochain";
pub const KEYS_DIRECTORY: &str = "keys";
pub const N3H_BINARIES_DIRECTORY: &str = "n3h-binaries";
pub const DNA_EXTENSION: &str = "dna.json";

/// Returns the project root builder for holochain directories.
pub fn project_root() -> Option<directories::ProjectDirs> {
    directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
}

/// Returns the path to the root config directory for all of Holochain.
/// If we can get a user directory it will be an XDG compliant path
/// like "/home/peter/.config/holochain".
/// If it can't get a user directory it will default to "/etc/holochain".
pub fn config_root() -> PathBuf {
    project_root()
        .map(|dirs| dirs.config_dir().to_owned())
        .unwrap_or_else(|| PathBuf::from("/etc").join(APPLICATION))
}

/// Returns the path to the root data directory for all of Holochain.
/// If we can get a user directory it will be an XDG compliant path
/// like "/home/peter/.local/share/holochain".
/// If it can't get a user directory it will default to "/etc/holochain".
pub fn data_root() -> PathBuf {
    project_root()
        .map(|dirs| dirs.data_dir().to_owned())
        .unwrap_or_else(|| PathBuf::from("/etc").join(APPLICATION))
}

/// Returns the path to where agent keys are stored and looked for by default.
/// Something like "~/.config/holochain/keys".
pub fn keys_directory() -> PathBuf {
    config_root().join(KEYS_DIRECTORY)
}

/// Returns the path to where n3h binaries will be downloaded / run
/// Something like "~/.local/share/holochain/n3h-binaries"
pub fn n3h_binaries_directory() -> PathBuf {
    data_root().join(N3H_BINARIES_DIRECTORY)
}
