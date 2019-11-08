extern crate holochain_stress;
extern crate lib3h_protocol;
extern crate lib3h_sodium;
extern crate sim2h;
extern crate structopt;
extern crate url2;

use holochain_stress::*;
use lib3h_protocol::uri::Lib3hUri;
use lib3h_sodium::SodiumCryptoSystem;
use sim2h::{
    websocket::{streams::*, tls::TlsConfig},
    Sim2h,
};
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use url2::prelude::*;

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

fn await_connection(port: u16) -> StreamManager<std::net::TcpStream> {
    let timeout = std::time::Instant::now()
        .checked_add(std::time::Duration::from_millis(5000))
        .unwrap();

    loop {
        // StreamManager is dual sided, but we're only using the client side
        // this tls config is for the not used server side, it can be unenc
        let tls_config = TlsConfig::Unencrypted;
        let mut stream_manager = StreamManager::with_std_tcp_stream(tls_config);

        if let Err(e) = stream_manager.connect(
            &Url2::parse(&format!("wss://127.0.0.1:{}", port))
        ) {
            println!("e1 {:?}", e);

            if std::time::Instant::now() >= timeout {
                panic!("could not connect within timeout");
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }

        loop {
            let (_, evs) = match stream_manager.process() {
                Err(e) => {
                    println!("e2 {:?}", e);
                    break;
                }
                Ok(s) => s,
            };

            let mut did_err = false;
            for ev in evs {
                match ev {
                    StreamEvent::ConnectResult(_, _) => return stream_manager,
                    StreamEvent::ErrorOccured(_, e) => {
                        println!("e3 {:?}", e);
                        did_err = true;
                        break;
                    }
                    _ => (),
                }
            }

            if did_err {
                break
            }
        }

        if std::time::Instant::now() >= timeout {
            panic!("could not connect within timeout");
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

struct Job {
    stream_manager: StreamManager<std::net::TcpStream>,
}

impl Job {
    pub fn new(port: u16) -> Self {
        Self {
            stream_manager: await_connection(port),
        }
    }
}

impl StressJob for Job {
    fn tick(&mut self, _logger: &mut StressJobMetricLogger) -> StressJobTickResult {
        let (_, evs) = self.stream_manager.process().unwrap();
        for ev in evs {
            match ev {
                StreamEvent::ErrorOccured(_, e) => panic!("{:?}", e),
                StreamEvent::ConnectResult(_, _) => println!("got connect"),
                StreamEvent::IncomingConnectionEstablished(_) => unimplemented!(),
                StreamEvent::ReceivedData(_, data) => println!("got data: {}", String::from_utf8_lossy(&data)),
                StreamEvent::ConnectionClosed(_) => panic!("connection cloned"),
            }
        }
        StressJobTickResult::default()
    }
}

struct Suite {
    sim2h_cont: Arc<Mutex<bool>>,
    sim2h_join: Option<std::thread::JoinHandle<()>>,
}

impl Suite {
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

        println!("sim2h started, attempt test self connection");

        // wait 'till server is accepting connections.
        // let this one get dropped
        await_connection(port);

        println!("HHHHHHHHHHHHHHHHHHHH");

        Self {
            sim2h_cont,
            sim2h_join,
        }
    }
}

impl StressSuite for Suite {
    fn start(&mut self) {}

    fn progress(&mut self, stats: &StressStats) {
        println!("{:#?}", stats);
    }

    fn stop(&mut self, stats: StressStats) {
        *self.sim2h_cont.lock().unwrap() = false;
        std::mem::replace(&mut self.sim2h_join, None).unwrap().join().unwrap();
        println!("{:#?}", stats);
    }
}

pub fn main() {
    let opt = Opt::from_args();
    if opt.sim2h_message_log_file.is_some() {
        unimplemented!();
    }
    stress_run(
        opt.clone().create_stress_run_config(Suite::new(opt.sim2h_port), Box::new(move || Job::new(opt.sim2h_port))),
    );
}
