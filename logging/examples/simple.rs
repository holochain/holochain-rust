#[macro_use]
extern crate log;
use doodle_log::{FastLoggerBuilder, tag::TagFilter};


fn main() {
    let _logger = FastLoggerBuilder::new()
        .set_level_from_str("Trace")
        .add_tag_filter(TagFilter::new("Abort", false, "Red"))
        .build()
        .expect("Fail to instanciate the logging factory.");

    // logger.add_tag_filter(TagFilter::new("^Abort", true, "Red"));

    trace!("Track me if you can.");
    debug!("What's bugging you today?");
    info!("Some interesting info here");
    warn!("You've been warned Sir!");
    error!("Abort the mission!!");

    // Let's give some time to the working thread to finish logging...
    std::thread::sleep(std::time::Duration::from_millis(10));
}
