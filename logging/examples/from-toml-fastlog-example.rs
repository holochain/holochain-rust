use logging::prelude::*;

fn main() {
    let toml = r#"
    [logger]
    level = "debug"

        [[logger.rules]]
        pattern = "info"
        color = "blue"

        [[logger.rules]]
        pattern = "twice"
        exclude = true
        color = "blue"

    "#;

    FastLoggerBuilder::from_toml(toml)
        .expect("Fail to instantiate the logger from toml.")
        .build()
        .expect("Fail to build logger from toml.");

    trace!("Track me if you can.");
    debug!("What's bugging you today?");
    info!("Some interesting info here");
    warn!("You've been warned Sir!");
    // This next one will not be logged according to our rule defined in the toml
    warn!("Let's not warn twice about the same stuff.");
    // And this one will be printed in red
    error!("Abort the mission!!");

    // Let's give some time to the working thread to finish logging...
    std::thread::sleep(std::time::Duration::from_millis(10));
}
