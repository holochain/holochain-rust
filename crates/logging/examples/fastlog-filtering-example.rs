use holochain_logging::prelude::*;

fn main() {
    let toml = r#"
    [logger]
    level = "debug"

        [[logger.rules]]
        pattern = ".*"
        exclude = true

        [[logger.rules]]
        pattern = "^holochain"
        exclude = false

        [[logger.rules]]
        pattern = "Cyan"
        exclude = false
        color = "Cyan"

        [[logger.rules]]
        pattern = "app-5"
        exclude = false
        color = "Green"
    "#;

    // We need a guard here in order to gracefully shutdown
    // the logging thread
    let mut guard = FastLoggerBuilder::from_toml(toml)
        .expect("Fail to instantiate the logger from toml.")
        .build()
        .expect("Fail to build logger from toml.");

    debug!("Should be logged 'Cyan' thanks to a rule.");
    debug!(target: "holochain-app-5", "Should be 'Green' thanks to the last rule.");
    debug!(target: "rpc-app-5", "Should be 'Green' thanks to the last rule as well.");

    // Should NOT be logged
    debug!(target: "rpc", "This is our dependency log filtering.");

    // Should be logged each in different color. We avoid filtering by prefixing using the 'target'
    // argument.
    info!(target: "holochain", "Log message from Holochain Core.");
    info!(target: "holochain-app-2", "Log message from Holochain Core with instance ID 2");
    info!(target: "holochain-app-4", "Log message from Holochain Core with instance ID 4");

    // This next one will not be logged according to our defined rule
    warn!("Discarded warning message here.");
    // All 'warning' and 'error' message have their own color
    warn!(target: "holochain", "You've been warned Sir!");
    error!(target: "holochain", "Abort the mission!!");

    // Flushes any buffered records
    guard.flush();
    // Flush and shutdown gracefully the logging thread
    guard.shutdown();
}
