use holochain_logging::prelude::*;

fn main() {

    // We need a guard here in order to gracefully shutdown
    // the logging thread
    let mut guard = FastLoggerBuilder::new()
        .timestamp_format("%Y-%m-%d %H:%M:%S%.6f")
        .set_level_from_str("Trace")
        .add_rule_filter(RuleFilter::new("bug", false, "Magenta"))
        .add_rule_filter(RuleFilter::new("twice", true, "Yellow"))
        .build()
        .expect("Fail to instanciate the logging factory.");

    trace!("Track me if you can.");
    debug!("What's bugging you today?");
    info!(target: "Simple_example_instance_id", "Some interesting info here");
    warn!("You've been warned Sir!");
    // This next one will not be logged according to our defined rule
    warn!("Let's not warn twice about the same stuff.");
    // And this one will be printed in red
    error!("Abort the mission!!");

    info!(target: "rpc", "Message from the parity crate.");
    info!(target: "main", "Message from main.");

    debug!(target: "Level::Debug", "Level::Debug ? {:?}", Level::Debug);

    // Flushes any buffered records
    guard.flush();
    // Flush and shutdown gracefully the logging thread
    guard.shutdown();
}
