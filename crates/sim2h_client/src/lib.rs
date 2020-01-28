#[macro_use]
extern crate log;

use holochain_persistence_api::hash::HashString;
use in_stream::*;
use lib3h_crypto_api::CryptoSystem;
use lib3h_protocol::{data_types::*, types::AgentPubKey};
use lib3h_sodium::SodiumCryptoSystem;
use sim2h::{
    crypto::{Provenance, SignedWireMessage},
    WireMessage,
};
use std::sync::{Arc, Mutex};
use url2::prelude::*;

thread_local! {
    pub static CRYPTO: Box<dyn CryptoSystem> = Box::new(SodiumCryptoSystem::new());
}

type Connection = InStreamWss<InStreamTcp>;

pub struct Sim2hClient {
    agent_pubkey: HashString,
    #[allow(dead_code)]
    pub_key: Arc<Mutex<Box<dyn lib3h_crypto_api::Buffer>>>,
    sec_key: Arc<Mutex<Box<dyn lib3h_crypto_api::Buffer>>>,
    connection: Connection,
    //    wss_connection: InStreamWss<InStreamTls<InStreamTcp>>,
}

impl Sim2hClient {
    #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CLI)]
    pub fn new(connect_uri: &Url2) -> Result<Self, String> {
        let (pub_key, sec_key) = CRYPTO.with(|crypto| {
            let mut pub_key = crypto.buf_new_insecure(crypto.sign_public_key_bytes());
            let mut sec_key = crypto.buf_new_secure(crypto.sign_secret_key_bytes());
            crypto.sign_keypair(&mut pub_key, &mut sec_key).unwrap();
            (pub_key, sec_key)
        });
        let enc = hcid::HcidEncoding::with_kind("hcs0").map_err(|e| format!("{}", e))?;
        let agent_pubkey = HashString::from(enc.encode(&*pub_key).unwrap());
        info!("Generated agent id: {}", agent_pubkey);
        let connection = await_in_stream_connect(connect_uri)
            .map_err(|e| format!("Error awaiting connection: {}", e))?;

        let out = Self {
            agent_pubkey,
            pub_key: Arc::new(Mutex::new(pub_key)),
            sec_key: Arc::new(Mutex::new(sec_key)),
            connection,
        };

        Ok(out)
    }

    pub fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }

    pub fn agent_pubkey(&self) -> AgentPubKey {
        AgentPubKey::from(self.agent_pubkey.clone())
    }

    pub fn await_msg<F>(&mut self, predicate: F) -> Result<WireMessage, String>
    where
        F: Fn(&WireMessage) -> bool,
    {
        let timeout = std::time::Instant::now()
            .checked_add(std::time::Duration::from_millis(10000))
            .unwrap();

        loop {
            if let Some(msg) = pull_message_from_stream(&mut self.connection) {
                if predicate(&msg) {
                    return Ok(msg);
                } else {
                    println!("await_msg skipping message: {:?}", msg);
                }
            }

            if std::time::Instant::now() >= timeout {
                Err("could not connect within timeout".to_string())?
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
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
            provenance: Provenance::new(self.agent_pubkey.clone().into(), sig.into()),
            payload,
        };
        let to_send: Opaque = signed_message.into();
        self.connection.write(to_send.as_bytes().into()).unwrap();
    }
}

fn pull_message_from_stream(connection: &mut Connection) -> Option<WireMessage> {
    let mut frame = WsFrame::default();
    match connection.read(&mut frame) {
        Ok(_) => {
            if let WsFrame::Binary(b) = frame {
                let msg: WireMessage = serde_json::from_slice(&b).unwrap();
                Some(msg)
            } else {
                panic!("unexpected {:?}", frame);
            }
        }
        Err(e) if e.would_block() => None,
        Err(e) => panic!(e),
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CLI)]
fn await_in_stream_connect(connect_uri: &Url2) -> Result<InStreamWss<InStreamTcp>, String> {
    let timeout = std::time::Instant::now()
        .checked_add(std::time::Duration::from_millis(60000))
        .unwrap();

    let mut read_frame = WsFrame::default();

    // keep trying to connect
    loop {
        //        let config = WssConnectConfig::new(TlsConnectConfig::new(TcpConnectConfig::default()));
        let config = WssConnectConfig::new(TcpConnectConfig::default());
        let mut connection =
            InStreamWss::connect(connect_uri, config).map_err(|e| format!("{}", e))?;
        connection.write(WsFrame::Ping(b"".to_vec())).unwrap();

        loop {
            let mut err = false;
            let frame = connection.read(&mut read_frame);
            println!("read: {:?}", frame);
            match frame {
                Ok(_) => return Ok(connection),
                Err(e) if e.would_block() => (),
                Err(_) => {
                    err = true;
                }
            }

            if std::time::Instant::now() >= timeout {
                Err("could not connect within timeout".to_string())?
            }

            if err {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
