//! `cargo run --bin sim2h_max_connections`

use in_stream::*;
use lib3h_crypto_api::CryptoSystem;
use lib3h_protocol::{data_types::*, protocol::*, uri::Lib3hUri};
use lib3h_sodium::SodiumCryptoSystem;
use log::*;
use sim2h::{
    crypto::{Provenance, SignedWireMessage},
    run_sim2h, DhtAlgorithm, Sim2h, WireMessage,
};
use std::sync::{Arc, Mutex};
use url2::prelude::*;

//fn await_in_stream_connect(connect_uri: &Lib3hUri) -> InStreamWss<InStreamTls<InStreamTcp>> {
fn await_in_stream_connect(connect_uri: &Lib3hUri) -> InStreamWss<InStreamTcp> {
    let timeout = std::time::Instant::now()
        .checked_add(std::time::Duration::from_millis(20000))
        .unwrap();

    let mut read_frame = WsFrame::default();

    // keep trying to connect
    loop {
        //let config = WssConnectConfig::new(TlsConnectConfig::new(TcpConnectConfig::default()));
        let config = WssConnectConfig::new(TcpConnectConfig::default());
        info!("try new connection -- {}", connect_uri);
        let mut connection = InStreamWss::connect(&(**connect_uri).clone().into(), config).unwrap();
        connection.write(WsFrame::Ping(b"".to_vec())).unwrap();

        loop {
            let mut err = false;

            match connection.read(&mut read_frame) {
                Ok(_) => return connection,
                Err(e) if e.would_block() => (),
                Err(_) => {
                    err = true;
                }
            }

            if std::time::Instant::now() >= timeout {
                panic!("could not connect within timeout");
            }

            if err {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

thread_local! {
    pub static CRYPTO: Box<dyn CryptoSystem> = Box::new(SodiumCryptoSystem::new());
}

/// our job is a websocket connection to sim2h immitating a holochain-rust core
struct Job {
    agent_id: String,
    #[allow(dead_code)]
    pub_key: Arc<Mutex<Box<dyn lib3h_crypto_api::Buffer>>>,
    sec_key: Arc<Mutex<Box<dyn lib3h_crypto_api::Buffer>>>,
    connection: InStreamWss<InStreamTcp>,
    //connection: InStreamWss<InStreamTls<InStreamTcp>>,
    last_ping: std::time::Instant,
    last_pong: std::time::Instant,
}

impl Job {
    /// create a new job - connected to sim2h
    pub fn new(connect_uri: &Lib3hUri) -> Self {
        let (pub_key, sec_key) = CRYPTO.with(|crypto| {
            let mut pub_key = crypto.buf_new_insecure(crypto.sign_public_key_bytes());
            let mut sec_key = crypto.buf_new_secure(crypto.sign_secret_key_bytes());
            crypto.sign_keypair(&mut pub_key, &mut sec_key).unwrap();
            (pub_key, sec_key)
        });
        let enc = hcid::HcidEncoding::with_kind("hcs0").unwrap();
        let agent_id = enc.encode(&*pub_key).unwrap();
        info!("GENERATED AGENTID {}", agent_id);

        let connection = await_in_stream_connect(connect_uri);

        let now = std::time::Instant::now();
        let mut out = Self {
            agent_id,
            pub_key: Arc::new(Mutex::new(pub_key)),
            sec_key: Arc::new(Mutex::new(sec_key)),
            connection,
            last_ping: now,
            last_pong: now,
        };

        out.join_space();
        out.ping();

        out
    }

    /// sign a message and send it to sim2h
    pub fn send_wire(&mut self, message: WireMessage) {
        let payload: Opaque = message.into();
        let payload_buf: Box<dyn lib3h_crypto_api::Buffer> = Box::new(payload.clone().as_bytes());
        let sig = base64::encode(
            &*CRYPTO
                .with(|crypto| {
                    let mut sig = crypto.buf_new_insecure(crypto.sign_bytes());
                    crypto
                        .sign(&mut sig, &payload_buf, &*self.sec_key.lock().unwrap())
                        .unwrap();
                    sig
                })
                .read_lock(),
        );
        let signed_message = SignedWireMessage {
            provenance: Provenance::new(self.agent_id.clone().into(), sig.into()),
            payload,
        };
        let to_send: Opaque = signed_message.into();
        self.connection.write(to_send.as_bytes().into()).unwrap();
    }

    /// join the space "abcd" : )
    pub fn join_space(&mut self) {
        self.send_wire(WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(
            SpaceData {
                agent_id: self.agent_id.clone().into(),
                request_id: "".to_string(),
                space_address: "abcd".to_string().into(),
            },
        )));
    }

    /// send a ping message to sim2h
    pub fn ping(&mut self) {
        self.last_ping = std::time::Instant::now();
        self.send_wire(WireMessage::Ping);
    }

    fn priv_handle_msg(&mut self, msg: WireMessage) {
        match msg {
            WireMessage::Pong => {
                self.last_pong = std::time::Instant::now();
            }
            WireMessage::Lib3hToClient(Lib3hToClient::HandleGetAuthoringEntryList(_))
            | WireMessage::Lib3hToClient(Lib3hToClient::HandleGetGossipingEntryList(_)) => {}
            e @ _ => panic!("unexpected: {:?}", e),
        }
    }

    /// check for any messages from sim2h and also send a ping
    fn tick(&mut self) {
        let mut frame = WsFrame::default();
        match self.connection.read(&mut frame) {
            Ok(_) => {
                if let WsFrame::Binary(b) = frame {
                    let msg: WireMessage = serde_json::from_slice(&b).unwrap();
                    self.priv_handle_msg(msg);
                } else {
                    panic!("unexpected {:?}", frame);
                }
            }
            Err(e) if e.would_block() => (),
            Err(e) => panic!(e),
        }

        if self.last_ping.elapsed().as_secs() > 1 {
            self.ping();
        }

        if self.last_pong.elapsed().as_secs() > 5 {
            panic!("connection hasn't gotten a pong in 5 seconds!!!");
        }
    }
}

#[allow(clippy::mutex_atomic)]
/// main function executes the stress suite given the cli arguments
pub fn main() {
    env_logger::init();

    // changed to ws until we reactive TLS
    let url = Url2::parse("ws://127.0.0.1:0");

    let sim2h = Sim2h::new(
        Box::new(SodiumCryptoSystem::new()),
        Lib3hUri(url.into()),
        DhtAlgorithm::FullSync,
    );

    let bound_uri = sim2h.bound_uri.as_ref().unwrap().clone();

    let mut rt = run_sim2h(sim2h);
    rt.block_on(async move {
        std::thread::spawn(|| {
            loop {
                warn!("1 second tick - hardware");
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });

        tokio::task::spawn(async move {
            loop {
                warn!("1 second tick - tokio");
                tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
            }
        });

        info!("bound sim2h to {}", bound_uri);

        // wait 'till server is accepting connections.
        await_in_stream_connect(&bound_uri);

        //let mut job = Job::new(&bound_uri);
        //job.tick();

        info!("CONNECTED, starting max_connection test");

        let mut con_count = 0;

        loop {
            // add a new job
            con_count += 1;

            let cbound_uri = bound_uri.clone();
            tokio::task::spawn(async move {
                let mut job = tokio::task::spawn_blocking(move || {
                    Job::new(&cbound_uri)
                }).await.unwrap();
                loop {
                    job.tick();
                    tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
                }
            });

            info!("CONNECTION COUNT: {}", con_count);

            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
        }
    });
}
