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
//! - **HC_N3H_PATH** *string* Path to the [n3h](https://github.com/holochain/n3h) networking module. If set, this will automatically trigger `hc run` to do live networking,
//! instead of mock networking which is the default. The following environment variables are irrelevant if `hc run` is not run with the `--networked`
//! flag, AND this is not set, because they are all configuration for live networking. Default is a subdirectory of your HOME folder, at the path `.hc/net/n3h`.
//! - **HC_N3H_WORK_DIR** *string* Eventually, there will be a directory needed by n3h for persisting data, such as remote node QoS metrics, peer lists, and non-core DHT data items such as peer discovery info.
//! Default is temporary directory which will get removed again once the Conductor process stops. Recommended not to use this at this time.
//! - **HC_N3H_BOOTSTRAP_NODE** *string* Set an external p2p bound ip4 address for another node, to bootstrap the networking discovery process.
//! Without this, a second node will of a network will be unable to find any others. See [configuring networking]([here](https://developer.holochain.org/guide/latest/hc_configuring_networking.html)
//! for details.
//! - **NETWORKING_CONFIG_FILE** *string* Path to a JSON file containing configuration for the n3h networking module. More on this soon. Recommended to
//! not use this as this time.
//!
//! ### Other
//! - **HC_SIMPLE_LOGGER_MUTE** *int* Setting this value to 1 will silence the log output of a SimpleLogger. Use with any Conductor.
//! - **HC_TARGET_PREFIX** *string* used exclusively for the purposes of automated testing on CI

// TODO, add this back in once the only option isn't "HACK"
// - **HC_N3H_MODE** *string* **HACK**

use std::env::VarError;

pub enum EnvVar {
    Agent,
    Interface,
    N3hPath,
    N3hMode,
    N3hWorkDir,
    N3hBootstrapNode,
    SimpleLoggerMute,
    NetworkingConfigFile,
    TargetPrefix,
}

impl EnvVar {
    pub fn as_str(&self) -> &str {
        match self {
            EnvVar::Agent => "HC_AGENT",
            EnvVar::Interface => "HC_INTERFACE",
            EnvVar::N3hPath => "HC_N3H_PATH",
            EnvVar::N3hMode => "HC_N3H_MODE",
            EnvVar::N3hWorkDir => "HC_N3H_WORK_DIR",
            EnvVar::N3hBootstrapNode => "HC_N3H_BOOTSTRAP_NODE",
            EnvVar::NetworkingConfigFile => "NETWORKING_CONFIG_FILE",
            EnvVar::SimpleLoggerMute => "HC_SIMPLE_LOGGER_MUTE",
            EnvVar::TargetPrefix => "HC_TARGET_PREFIX",
        }
    }
    pub fn value(&self) -> Result<String, VarError> {
        std::env::var(self.as_str())
    }
}
