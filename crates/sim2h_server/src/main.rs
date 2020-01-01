extern crate lib3h_sodium;
extern crate num_cpus;
extern crate structopt;

use lib3h_protocol::uri::Builder;
use lib3h_sodium::SodiumCryptoSystem;
use log::error;
use sim2h::{DhtAlgorithm, Sim2h, MESSAGE_LOGGER};
use std::{path::PathBuf, process::exit, sync::Arc};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(
        long,
        short,
        help = "The port to run the websocket server at",
        default_value = "9000"
    )]
    port: u16,
    #[structopt(
        long,
        short,
        help = "Sharding redundancy count; use 0 for fullsync",
        default_value = "50"
    )]
    sharding: u64,
    #[structopt(
        long,
        short,
        help = "CSV file to log all incoming and outgoing messages to"
    )]
    message_log_file: Option<PathBuf>,
}

/// By default will scale to number of cores.
/// This forces it to *at most* this many threads.
const MAX_PROCESSING_THREADS: usize = 2;

fn main() {
    env_logger::init();

    let args = Cli::from_args();

    let host = "ws://0.0.0.0/";
    let uri = Builder::with_raw_url(host)
        .unwrap_or_else(|e| panic!("with_raw_url: {:?}", e))
        .with_port(args.port)
        .build();
    if let Some(message_log_file) = args.message_log_file {
        MESSAGE_LOGGER.lock().set_logfile(message_log_file);
        MESSAGE_LOGGER.lock().start();
    }

    let mut sim2h = Sim2h::new(Box::new(SodiumCryptoSystem::new()), uri);
    if args.sharding > 0 {
        sim2h.set_dht_algorithm(DhtAlgorithm::NaiveSharding {
            redundant_count: args.sharding,
        });
    }

    let mut threads = Vec::new();
    let sim2h = Arc::new(sim2h);
    for cpu_index in 0..std::cmp::min(MAX_PROCESSING_THREADS, num_cpus::get()) {
        let sim2h = sim2h.clone();
        let result = std::thread::Builder::new()
            .name(format!("sim2h-processor-thread-{}", cpu_index))
            .spawn(move || loop {
            let result = sim2h.process();
            if let Err(e) = result {
                if e.to_string().contains("Bind error:") {
                    println!("{:?}", e);
                    exit(1)
                } else {
                    error!("{}", e.to_string())
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        });
        threads.push(result);
    }

    for t in threads {
        let _ = t.map(|t| t.join());
    }
}
