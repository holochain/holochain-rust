//! Mainly this is where the log macros helper are defined.

/// Helper macro to automatically fill the log's target expression record. This is how the log
/// facility we use filter the logs from our dependendies.
///
/// Once a logger has been registered you can use those macros like this:
/// ```ignore
///
/// // Let's init a new context
/// let ctx = Context::new(...);
///
/// // This will automatically use the instance_name of a context as a log target
/// log_debug!(ctx, "'{}' log level with Context target.", "Debug");
///
/// // Or you can use a custom target (this is used by holochain to filter the log from its dependencies
/// log_info!(target: "holochain-custom-log-target", "Custom target '{}' log here.", "Info");
/// ```

#[allow(unused_imports)]
use holochain_logging::prelude::*;

/// Helper macro for the [`Trace`](log::Level::Trace) log verbosity level.
#[macro_export]
macro_rules! log_trace {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, log::Level::Trace, $($arg)+);
    );
    ($ctx:expr, $($arg:tt)+) => (
        log!(target: &format!("holochain::{}", $ctx.get_instance_name()), log::Level::Trace, $($arg)+);
    );
    ($($arg:tt)+) => (
        log!(target: "holochain", log::Level::Trace, $($arg)+);
    )
}

/// Helper macro for the [`Debug`](log::Level::Debug) log verbosity level.
#[macro_export]
macro_rules! log_debug {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, Level::Debug, $($arg)+);
    );
    ($ctx:expr, $($arg:tt)+) => (
        log!(target: &format!("holochain::{}", $ctx.get_instance_name()), log::Level::Debug, $($arg)+);
    );
    ($($arg:tt)+) => (
        log!(target: "holochain", Level::Debug, $($arg)+);
    )
}

/// Helper macro for the [`Info`](log::Level::Info) log verbosity level.
#[macro_export]
macro_rules! log_info {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, log::Level::Info, $($arg)+);
    );
    ($ctx:expr, $($arg:tt)+) => (
        log!(target: &format!("holochain::{}", $ctx.get_instance_name()), log::Level::Info, $($arg)+);
    );
    ($($arg:tt)+) => (
        log!(target: "holochain", log::Level::Info, $($arg)+);
    )
}

/// Helper macro for the [`Warning`](log::Level::Warn) log verbosity level.
#[macro_export]
macro_rules! log_warn {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, log::Level::Warn, $($arg)+);
    );
    ($ctx:expr, $($arg:tt)+) => (
        log!(target: &format!("holochain::{}", $ctx.get_instance_name()), log::Level::Warn, $($arg)+);
    );
    ($($arg:tt)+) => (
        log!(target: "holochain", log::Level::Warn, $($arg)+);
    )
}

/// Helper macro for the [`Error`](log::Level::Error) log verbosity level.
#[macro_export]
macro_rules! log_error {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, log::Level::Error, $($arg)+);
    );
    ($ctx:expr, $($arg:tt)+) => (
        log!(target: &format!("holochain::{}", $ctx.get_instance_name()), log::Level::Error, $($arg)+);
    );
    ($($arg:tt)+) => (
        log!(target: "holochain", log::Level::Error, $($arg)+);
    )
}

#[test]
fn context_log_macro_test() {
    use crate::{context::Context, persister::SimplePersister};
    use holochain_core_types::{agent::AgentId, sync::HcRwLock as RwLock};
    use holochain_net::p2p_config::P2pConfig;
    use holochain_persistence_file::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
    use std::sync::Arc;
    use tempfile::tempdir;

    let file_storage = Arc::new(RwLock::new(
        FilesystemStorage::new(tempdir().unwrap().path()).unwrap(),
    ));
    let ctx = Context::new(
        "LOG-TEST-ID",
        AgentId::generate_fake("Bilbo"),
        Arc::new(RwLock::new(SimplePersister::new(file_storage.clone()))),
        file_storage.clone(),
        file_storage.clone(),
        Arc::new(RwLock::new(
            EavFileStorage::new(tempdir().unwrap().path()).unwrap(),
        )),
        P2pConfig::new_with_unique_memory_backend(),
        None,
        None,
        false,
    );

    // Somehow we need to build our own logging instance for this test to show logs
    use holochain_logging::prelude::*;
    let _guard = FastLoggerBuilder::new()
        .set_level_from_str("Trace")
        .build()
        .expect("Fail to init logger.");

    // Tests if the context logger can be customized by poassing a target value
    log_info!(target: "holochain-custom-log-target", "Custom target '{}' log here.", "Debug");

    // Tests if the context logger fills its target with the instance ID
    log_trace!(ctx, "'{}' log level with Context target.", "Trace");
    log_debug!(ctx, "'{}' log level with Context target.", "Debug");
    log_info!(ctx, "'{}' log level with Context target.", "Info");
    log_warn!(ctx, "'{}' log level with Context target.", "Warning");
    log_error!(ctx, "'{}' log level with Context target.", "Error");

    _guard.flush();
}
