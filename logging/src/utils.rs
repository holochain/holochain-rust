use log::{
    error, warn, info, debug, trace,
};

pub fn simulate_messages() {
    trace!("Track me this if you can.");
    debug!("What's bugging you today?");
    info!("This message is realy informative :)");
    warn!("You've been warned Sir!");
    error!("Abort the mission!!");
}
