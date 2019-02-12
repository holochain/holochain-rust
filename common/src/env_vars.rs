use std::{env::VarError, ffi::OsStr};

pub enum EnvVar {
    Agent,
    Interface,
    N3hPath,
    N3hMode,
    N3hWorkDir,
    N3hBootstrapNode,
    SimpleLoggerMute,
    TargetPrefix,
}

impl AsRef<OsStr> for EnvVar {
    fn as_ref(&self) -> &OsStr {
        match &self {
            EnvVar::Agent => &OsStr::new("HC_AGENT"),
            EnvVar::Interface => &OsStr::new("HC_INTERFACE"),
            EnvVar::N3hPath => &OsStr::new("HC_N3H_PATH"),
            EnvVar::N3hMode => &OsStr::new("HC_N3H_MODE"),
            EnvVar::N3hWorkDir => &OsStr::new("HC_N3H_WORK_DIR"),
            EnvVar::N3hBootstrapNode => &OsStr::new("HC_N3H_BOOTSTRAP_NODE"),
            EnvVar::SimpleLoggerMute => &OsStr::new("HC_SIMPLE_LOGGER_MUTE"),
            EnvVar::TargetPrefix => &OsStr::new("HC_TARGET_PREFIX"),
        }
    }
}

pub fn get_env_var_value(env_var: EnvVar) -> Result<String, VarError> {
    std::env::var(env_var)
}
