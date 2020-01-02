use crossbeam_channel::unbounded;
use lib3h::transport::transport_test_harness::get_available_port;
use lib3h_protocol::{data_types::*, protocol::*, uri::Builder};
use lib3h_sodium::SodiumCryptoSystem;
use sim2h::{wire_message::WireMessage, Sim2h};
use sim2h_client::Sim2hClient;
use std::{thread, time::Duration};
use url::Url;
use url2::Url2;

/// TODO: write test of some debug data once:
/// - Sim2hClient refactor is merged
/// - Walkman playback functionality is merged
#[test]
fn queue_size_is_reasonable() {
    let port = get_available_port(2000).expect("Couldn't get an available port");
    let host = "ws://0.0.0.0/";
    let uri = Builder::with_raw_url(host)
        .unwrap_or_else(|e| panic!("with_raw_url: {:?}", e))
        .with_port(port)
        .build();
    let sim2h_uri = uri.clone();
    let (tx_kill, rx_kill) = unbounded::<()>();
    let sim2h_thread = thread::spawn(move || {
        let mut dumps = Vec::new();
        println!("Starting sim2h at {}", sim2h_uri);
        let mut sim2h = Sim2h::new(Box::new(SodiumCryptoSystem::new()), sim2h_uri, false);
        let mut t = 0;
        while let Err(_) = rx_kill.try_recv() {
            sim2h.process().unwrap();
            t += 10;
            if t > 1000 {
                t -= 1000;
                println!("dump...");
                dumps.push(sim2h.get_debug_data());
            }
            thread::sleep(Duration::from_millis(10));
        }
        dumps
    });

    thread::sleep(Duration::from_millis(500));

    let clients = (0..100).map(|_| Sim2hClient::new(&Url2::from(Url::from(uri.clone()))).unwrap());
    clients.for_each(|mut client: Sim2hClient| {
        println!("joining...");
        client.send_wire(WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(
            SpaceData {
                request_id: "".into(),
                space_address: "space".into(),
                agent_id: client.agent_pubkey(),
            },
        )));
    });

    thread::sleep(Duration::from_secs(10));
    tx_kill.send(()).unwrap();
    let dumps = sim2h_thread.join().unwrap();
    assert_eq!(dumps.len(), 10);
    // TODO: check something real about the debug data
}
