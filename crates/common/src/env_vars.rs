//! Holochain uses a number of environment variables that can be set, for easy configuration for aspects of the system.
//! Below is the complete list of them, and what they are used for.
//!
//! ### `hc run`
//! Read more about the use of these environment variables [here](https://developer.holochain.org/guide/latest/hc_configuring_networking.html).
//! - **HC_AGENT** *string* Set an alternative name for the agent for the development instance.
//! Default value is `testAgent`.
//! Useful for changing the agent while running multiple instances.
//! - **HC_INTERFACE** *string* **websocket** OR **http** Set an interface type to use. Setting this as an environment variable will override the
//! value of the `--interface` option for `hc run`. The default interface if neither is set is `websocket`.
//! Default is temporary directory which will get removed again once the Conductor process stops. Recommended not to use this at this time.
//! Without this, a second node will of a network will be unable to find any others. See [configuring networking]([here](https://developer.holochain.org/guide/latest/hc_configuring_networking.html)
//! for details.
//! - **NETWORKING_CONFIG_FILE** *string* Path to a JSON file containing configuration for the networking module. More on this soon. Recommended to
//! not use this as this time.
//!
//! ### `hc generate`
//! - HC_SCAFFOLD_VERSION allows you to set a string value to be used in the generated Cargo.toml.  We use this override the default which points to the current version tag, which a pointer to the develop branch for our CI tests, so for example in CI we can run our tests with: `HC_SCAFFOLD_VERSION='branch="develop"'` and that overrides the default.
//!

use std::env::VarError;

pub enum EnvVar {
    Agent,
    Interface,
    NetworkingConfigFile,
    ScaffoldVersion,
}

impl EnvVar {
    pub fn as_str(&self) -> &str {
        match self {
            EnvVar::Agent => "HC_AGENT",
            EnvVar::Interface => "HC_INTERFACE",
            EnvVar::NetworkingConfigFile => "NETWORKING_CONFIG_FILE",
            EnvVar::ScaffoldVersion => "HC_SCAFFOLD_VERSION",
        }
    }

    pub fn value(&self) -> Result<String, VarError> {
        std::env::var(self.as_str())
    }
}

#[macro_export]
macro_rules! new_relic_setup {
    ($x:expr) => {
        lazy_static::lazy_static! {
            static ref NEW_RELIC_LICENSE_KEY: Option<String> =
                option_env!($x).map(|s| s.to_string());
        }
    };
}

#[test]
fn test_macro() {
    new_relic_setup!("NEW_RELIC_LICENSE_KEY");
    assert_eq!(*NEW_RELIC_LICENSE_KEY, None);
}
