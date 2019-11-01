pub mod env_vars;
pub mod paths;

// TODO: Remove this as soon as we have keystores that can store and lock multiple keys with a single passphrase.
// (This is just for bootstrapping while still in alpha)
pub const DEFAULT_PASSPHRASE: &str = "convenient and insecure keystore";
