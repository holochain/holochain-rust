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
