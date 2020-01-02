use lib3h::transport::transport_test_harness::get_available_port;
use lib3h_protocol::uri::Builder;
use lib3h_sodium::SodiumCryptoSystem;
use sim2h::Sim2h;

#[test]
fn queue_size_is_reasonable() {
    let port = get_available_port(1001).expect("Couldn't get an available port");
    let host = "ws://0.0.0.0/";
    let uri = Builder::with_raw_url(host)
        .unwrap_or_else(|e| panic!("with_raw_url: {:?}", e))
        .with_port(port)
        .build();
    let mut _sim2h = Sim2h::new(Box::new(SodiumCryptoSystem::new()), uri);
}
