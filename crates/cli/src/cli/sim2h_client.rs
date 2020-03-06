use dns_lookup::lookup_host;
use in_stream::*;
use lib3h_crypto_api::CryptoSystem;
use lib3h_protocol::data_types::*;
use lib3h_sodium::SodiumCryptoSystem;
use sim2h::{
    crypto::{Provenance, SignedWireMessage},
    WireMessage, WIRE_VERSION,
};
use std::{
    fs::File,
    io::prelude::*,
    sync::{Arc, Mutex},
};
use url2::prelude::*;

//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CLI)]
pub fn sim2h_client(url_string: String, message_string: String) -> Result<(), String> {
    let url = match Url2::try_parse(url_string.clone()) {
        Err(e) => Err(format!(
            "unable to parse url:{} got error: {}",
            url_string, e
        ))?,
        Ok(url) => url,
    };
    let host = format!("{}", url.host().unwrap());
    let ip = if host == "localhost" {
        "127.0.0.1".to_string()
    } else {
        println!("looking up: {}", host);
        let ips: Vec<std::net::IpAddr> = lookup_host(&host).map_err(|e| format!("{}", e))?;
        println!("resolved to: {}", ips[0]);
        format!("{}", ips[0])
    };
    let maybe_port = url.port();
    if maybe_port.is_none() {
        return Err(format!("expecting port in url, got: {}", url));
    }
    let url = Url2::parse(format!("{}://{}:{}", url.scheme(), ip, maybe_port.unwrap()));

    println!("connecting to: {}", url);
    let mut job = Job::new(&url)?;
    job.send_wire(match message_string.as_ref() {
        "ping" => WireMessage::Ping,
        "hello" => WireMessage::Hello(WIRE_VERSION),
        "status" => WireMessage::Status,
        "debug" => WireMessage::Debug,
        _ => {
            return Err(format!(
                "expecting 'ping', 'status' or 'debug' for message, got: {}",
                message_string
            ))
        }
    });
    let timeout = std::time::Instant::now()
        .checked_add(std::time::Duration::from_millis(60000))
        .unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
        let mut frame = WsFrame::default();
        match job.connection.read(&mut frame) {
            Ok(_) => {
                if let WsFrame::Binary(b) = frame {
                    let msg: WireMessage = serde_json::from_slice(&b).unwrap();
                    match msg {
                        WireMessage::Pong
                        | WireMessage::HelloResponse(_)
                        | WireMessage::StatusResponse(_) => {
                            println!("Got response => {:?}", msg);
                            break;
                        }
                        WireMessage::DebugResponse(debug_response_map) => {
                            println!("Got DebugResponse for {} spaces.", debug_response_map.len());
                            for (space, json) in debug_response_map {
                                let filename = format!("{}.json", space);
                                println!(
                                    "Writing Sim2h state dump for space {} to file: {}",
                                    space, filename
                                );

                                File::create(filename.clone())
                                    .unwrap_or_else(|_| {
                                        panic!("Could not create file {}!", filename)
                                    })
                                    .write_all(json.into_bytes().as_slice())
                                    .expect("Could not write to file!");
                            }
                            break;
                        }
                        _ => println!("{:?}", msg),
                    }
                } else {
                    Err(format!("unexpected {:?}", frame))?;
                }
            }
            Err(e) if e.would_block() => (),
            Err(e) => Err(format!("{}", e))?,
        }
        if std::time::Instant::now() >= timeout {
            Err(format!("timeout waiting for status response from {}", host))?;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}

thread_local! {
    pub static CRYPTO: Box<dyn CryptoSystem> = Box::new(SodiumCryptoSystem::new());
}
struct Job {
    agent_id: String,
    #[allow(dead_code)]
    pub_key: Arc<Mutex<Box<dyn lib3h_crypto_api::Buffer>>>,
    sec_key: Arc<Mutex<Box<dyn lib3h_crypto_api::Buffer>>>,
    connection: InStreamWss<InStreamTcp>,
    //    wss_connection: InStreamWss<InStreamTls<InStreamTcp>>,
}

//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CLI)]
impl Job {
    pub fn new(connect_uri: &Url2) -> Result<Self, String> {
        let (pub_key, sec_key) = CRYPTO.with(|crypto| {
            let mut pub_key = crypto.buf_new_insecure(crypto.sign_public_key_bytes());
            let mut sec_key = crypto.buf_new_secure(crypto.sign_secret_key_bytes());
            crypto.sign_keypair(&mut pub_key, &mut sec_key).unwrap();
            (pub_key, sec_key)
        });
        let enc = hcid::HcidEncoding::with_kind("hcs0").map_err(|e| format!("{}", e))?;
        let agent_id = enc.encode(&*pub_key).unwrap();
        println!("Generated agent id: {}", agent_id);
        let connection = await_in_stream_connect(connect_uri)
            .map_err(|e| format!("Error awaiting connection: {}", e))?;
        println!("Await successfull");
        let out = Self {
            agent_id,
            pub_key: Arc::new(Mutex::new(pub_key)),
            sec_key: Arc::new(Mutex::new(sec_key)),
            connection,
        };

        Ok(out)
    }

    /// sign a message and send it to sim2h
    pub fn send_wire(&mut self, message: WireMessage) {
        println!("Sending wire message to sim2h: {:?}", message);
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
}

//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CLI)]
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
            match connection.read(&mut read_frame) {
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
