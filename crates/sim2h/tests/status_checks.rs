use lib3h::transport::transport_test_harness::get_available_port;
use lib3h_protocol::uri::Builder;
use lib3h_sodium::SodiumCryptoSystem;
use sim2h::Sim2h;
use std::{thread, time::Duration};

/// TODO: write test of some debug data once:
/// - Sim2hClient refactor is merged
/// - Walkman playback functionality is merged
#[test]
fn queue_size_is_reasonable() {
    let port = get_available_port(1001).expect("Couldn't get an available port");
    let host = "ws://0.0.0.0/";
    let uri = Builder::with_raw_url(host)
        .unwrap_or_else(|e| panic!("with_raw_url: {:?}", e))
        .with_port(port)
        .build();
    thread::spawn(move || {
        let mut _sim2h = Sim2h::new(Box::new(SodiumCryptoSystem::new()), uri, false);
        thread::sleep(Duration::from_secs(5))
    });
    thread::sleep(Duration::from_secs(3));
    println!("Do a debug dump check here");
}
