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

#[derive(StructOpt, Debug)]
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

struct Job {}

impl Job {
    pub fn new() -> Self {
        Self {}
    }
}

impl StressJob for Job {
    fn tick(&mut self, _logger: &mut StressJobMetricLogger) -> StressJobTickResult {
        StressJobTickResult::default()
    }
}

struct Suite {
    sim2h: Arc<Mutex<Sim2h>>,
}

impl Suite {
    pub fn new(port: u16) -> Self {
        let tls_config = TlsConfig::build_from_entropy();
        let stream_manager = StreamManager::with_std_tcp_stream(tls_config);
        let url = Url2::parse(&format!("wss://0.0.0.0:{}", port));
        let sim2h = Sim2h::new(
            Box::new(SodiumCryptoSystem::new()),
            stream_manager,
            Lib3hUri(url.into()),
        );
        Self {
            sim2h: Arc::new(Mutex::new(sim2h)),
        }
    }
}

impl StressSuite for Suite {
    fn start(&mut self) {}

    fn tick(&mut self) {
        if let Err(e) = self.sim2h.lock().unwrap().process() {
            panic!("{:?}", e);
        }
    }

    fn progress(&mut self, stats: &StressStats) {
        println!("{:#?}", stats);
    }

    fn stop(&mut self, stats: StressStats) {
        println!("{:#?}", stats);
    }
}

pub fn main() {
    let opt = Opt::from_args();
    if opt.sim2h_message_log_file.is_some() {
        unimplemented!();
    }
    stress_run(
        opt.create_stress_run_config(Suite::new(opt.sim2h_port), Box::new(move || Job::new())),
    );
}
