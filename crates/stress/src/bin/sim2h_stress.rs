//! `cargo run --bin sim2h_stress -- --help`

extern crate base64;
extern crate env_logger;
extern crate hcid;
extern crate holochain_stress;
extern crate lib3h_crypto_api;
extern crate lib3h_protocol;
extern crate lib3h_sodium;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate sim2h;
extern crate structopt;
extern crate url2;

use holochain_stress::*;
use lib3h_crypto_api::CryptoSystem;
use lib3h_protocol::{
    data_types::{Opaque, SpaceData},
    protocol::*,
    uri::Lib3hUri,
};
use lib3h_sodium::SodiumCryptoSystem;
use sim2h::{
    crypto::{Provenance, SignedWireMessage},
    websocket::{streams::*, tls::TlsConfig},
    Sim2h, WireMessage,
};
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use url2::prelude::*;

/// give us some cli command line options
#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "sim2h_stress")]
struct Opt {
    #[structopt(short, long, default_value = "10")]
    /// how many threads to spin up in the job executor pool
    thread_count: usize,

    #[structopt(short, long, default_value = "100")]
    /// how many parallel jobs to execute
    job_count: usize,

    #[structopt(short, long, default_value = "10000")]
    /// total runtime for the test
    run_time_ms: u64,

    #[structopt(short, long, default_value = "5000")]
    /// how often to output in-progress statistics
    progress_interval_ms: u64,

    #[structopt(long, default_value = "33221")]
    /// port on which to spin up the sim2h server
    sim2h_port: u16,

    #[structopt(long)]
    /// optional sim2h log file path
    sim2h_message_log_file: Option<std::path::PathBuf>,
}

impl Opt {
    /// private convert our cli options into a stress job config
    fn create_stress_run_config<S: StressSuite, J: StressJob>(
        &self,
        suite: S,
        job_factory: JobFactory<J>,
    ) -> StressRunConfig<S, J> {
        StressRunConfig {
            thread_pool_size: self.thread_count,
            job_count: self.job_count,
            run_time_ms: self.run_time_ms,
            progress_interval_ms: self.progress_interval_ms,
            suite,
            job_factory,
        }
    }
}

/// private wait for a websocket connection to connect && return it
fn await_connection(port: u16) -> (Url2, StreamManager<std::net::TcpStream>) {
    let timeout = std::time::Instant::now()
        .checked_add(std::time::Duration::from_millis(1000))
        .unwrap();

    // keep trying to connect
    loop {
        // StreamManager is dual sided, but we're only using the client side
        // this tls config is for the not used server side, it can be fake
        let tls_config = TlsConfig::FakeServer;
        let mut stream_manager = StreamManager::with_std_tcp_stream(tls_config);

        // TODO - wtf, we don't want a listening socket : (
        //        but the logs are way too complainy
        stream_manager
            .bind(&Url2::parse("wss://127.0.0.1:0").into())
            .unwrap();

        let url = Url2::parse(&format!("wss://127.0.0.1:{}", port));

        // the actual connect request
        if let Err(e) = stream_manager.connect(&url) {
            error!("e1 {:?}", e);

            if std::time::Instant::now() >= timeout {
                panic!("could not connect within timeout");
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        // now loop to see if we can communicate
        loop {
            let (_, evs) = match stream_manager.process() {
                Err(e) => {
                    error!("e2 {:?}", e);
                    break;
                }
                Ok(s) => s,
            };

            let mut did_err = false;
            for ev in evs {
                match ev {
                    StreamEvent::ConnectResult(_, _) => return (url, stream_manager),
                    StreamEvent::ErrorOccured(_, e) => {
                        error!("e3 {:?}", e);
                        did_err = true;
                        break;
                    }
                    _ => (),
                }
            }

            if did_err {
                break;
            }
        }

        if std::time::Instant::now() >= timeout {
            panic!("could not connect within timeout");
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
    remote_url: Url2,
    stream_manager: StreamManager<std::net::TcpStream>,
}

impl Job {
    /// create a new job - connected to sim2h
    pub fn new(port: u16) -> Self {
        let (pub_key, sec_key) = CRYPTO.with(|crypto| {
            let mut pub_key = crypto.buf_new_insecure(crypto.sign_public_key_bytes());
            let mut sec_key = crypto.buf_new_secure(crypto.sign_secret_key_bytes());
            crypto.sign_keypair(&mut pub_key, &mut sec_key).unwrap();
            (pub_key, sec_key)
        });
        let enc = hcid::HcidEncoding::with_kind("hcs0").unwrap();
        let agent_id = enc.encode(&*pub_key).unwrap();
        info!("GENERATED AGENTID {}", agent_id);
        let (remote_url, stream_manager) = await_connection(port);
        let mut out = Self {
            agent_id,
            pub_key: Arc::new(Mutex::new(pub_key)),
            sec_key: Arc::new(Mutex::new(sec_key)),
            remote_url,
            stream_manager,
        };

        out.join_space();

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
        self.stream_manager
            .send(
                &self.remote_url.clone().into(),
                to_send.as_bytes().as_slice(),
            )
            .unwrap();
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
        self.send_wire(WireMessage::Ping);
    }
}

impl StressJob for Job {
    /// check for any messages from sim2h and also send a ping
    fn tick(&mut self, logger: &mut StressJobMetricLogger) -> StressJobTickResult {
        let (_, evs) = self.stream_manager.process().unwrap();
        for ev in evs {
            match ev {
                StreamEvent::ErrorOccured(_, e) => panic!("{:?}", e),
                StreamEvent::ConnectResult(_, _) => panic!("got ConnectResult"),
                StreamEvent::IncomingConnectionEstablished(_) => unimplemented!(),
                StreamEvent::ReceivedData(_, data) => {
                    let data = String::from_utf8_lossy(&data);
                    if &data == "\"Pong\"" {
                        logger.log("received_pong_count", 1.0);
                    }
                }
                StreamEvent::ConnectionClosed(_) => panic!("connection cloned"),
            }
        }

        // stress test is currently just sending ping every tick
        self.ping();

        StressJobTickResult::default()
    }
}

/// our suite creates a thread for sim2h and gives the code processor time
struct Suite {
    sim2h_cont: Arc<Mutex<bool>>,
    sim2h_join: Option<std::thread::JoinHandle<()>>,
}

impl Suite {
    /// create a new sim2h server instance on given port
    pub fn new(port: u16) -> Self {
        let sim2h_cont = Arc::new(Mutex::new(true));
        let sim2h_cont_clone = sim2h_cont.clone();
        let sim2h_join = Some(std::thread::spawn(move || {
            let tls_config = TlsConfig::build_from_entropy();
            let stream_manager = StreamManager::with_std_tcp_stream(tls_config);
            let url = Url2::parse(&format!("wss://127.0.0.1:{}", port));

            let mut sim2h = Sim2h::new(
                Box::new(SodiumCryptoSystem::new()),
                stream_manager,
                Lib3hUri(url.into()),
            );

            while *sim2h_cont_clone.lock().unwrap() {
                std::thread::sleep(std::time::Duration::from_millis(1));
                if let Err(e) = sim2h.process() {
                    panic!("{:?}", e);
                }
            }
        }));

        info!("sim2h started, attempt test self connection");

        // wait 'till server is accepting connections.
        // let this one get dropped
        await_connection(port);

        Self {
            sim2h_cont,
            sim2h_join,
        }
    }
}

impl StressSuite for Suite {
    fn start(&mut self) {}

    fn progress(&mut self, stats: &StressStats) {
        println!("PROGRESS: {:#?}", stats);
    }

    fn stop(&mut self, stats: StressStats) {
        *self.sim2h_cont.lock().unwrap() = false;
        std::mem::replace(&mut self.sim2h_join, None)
            .unwrap()
            .join()
            .unwrap();
        println!("FINAL STATS: {:#?}", stats);
    }
}

/// main function executes the stress suite given the cli arguments
pub fn main() {
    env_logger::init();
    let opt = Opt::from_args();
    if opt.sim2h_message_log_file.is_some() {
        unimplemented!();
    }
    let config = opt.clone().create_stress_run_config(
        Suite::new(opt.sim2h_port),
        Box::new(move || Job::new(opt.sim2h_port)),
    );
    println!("RUNNING WITH CONFIG: {:#?}", config);
    stress_run(config);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_start_sim2h_and_connect() {
        env_logger::init();
        let suite = Suite::new(44332);
        let mut job = Some(Job::new(44332));
        std::thread::sleep(std::time::Duration::from_millis(500));
        stress_run(StressRunConfig {
            thread_pool_size: 1,
            job_count: 1,
            run_time_ms: 1000,
            progress_interval_ms: 2000,
            suite,
            job_factory: Box::new(move || std::mem::replace(&mut job, None).unwrap()),
        });
    }
}
