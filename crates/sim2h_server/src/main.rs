extern crate lib3h_sodium;
extern crate num_cpus;
extern crate structopt;

use lib3h_protocol::uri::Builder;
use lib3h_sodium::SodiumCryptoSystem;
use log::error;
use sim2h::{DhtAlgorithm, Sim2hFactory, Spaces, MESSAGE_LOGGER};
use std::{path::PathBuf, process::exit};
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
const NUM_PROCESSING_THREADS_PER_CORE: usize = 1;

fn main() {
    env_logger::init();

    let args = Cli::from_args();

    let host = "ws://0.0.0.0/";
    if let Some(message_log_file) = args.message_log_file {
        MESSAGE_LOGGER.lock().set_logfile(message_log_file);
        MESSAGE_LOGGER.lock().start();
    }
    let uri = Builder::with_raw_url(host)
        .unwrap_or_else(|e| panic!("with_raw_url: {:?}", e))
        .with_port(args.port)
        .build();

    let mut sim2h_factory = Sim2hFactory::new(Box::new(SodiumCryptoSystem::new()), uri);
    if args.sharding > 0 {
        sim2h_factory.set_dht_algorithm(DhtAlgorithm::NaiveSharding {
            redundant_count: args.sharding,
        });
    }

    let (read, write) = evmap::new();
    let write = std::sync::Arc::new(holochain_locksmith::Mutex::new(write));
    let sim2h_factory = std::sync::Arc::new(sim2h_factory);
    let mut threads = Vec::new();
    for cpu_index in 0..(NUM_PROCESSING_THREADS_PER_CORE * num_cpus::get()) {
        let sim2h_factory = sim2h_factory.clone();
        let read = read.clone();
        let write = write.clone();
        let result = std::thread::Builder::new()
            .name(format!("sim2h-processor-thread-{}", cpu_index))
            .spawn(move || {
                let sim2h = sim2h_factory.create_sim2h(Spaces { read, write });
                loop {
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
                }
            });
        threads.push(result);
    }

    for t in threads {
        let _ = t.map(|t| t.join());
    }
}
