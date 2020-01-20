extern crate lib3h_sodium;
extern crate structopt;

use lib3h_protocol::uri::Builder;
use lib3h_sodium::SodiumCryptoSystem;
use sim2h::{run_sim2h, DhtAlgorithm, Sim2h, MESSAGE_LOGGER};
use std::path::PathBuf;
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
    sim2h.set_dht_algorithm(DhtAlgorithm::NaiveSharding {
        redundant_count: args.sharding,
    });

    run_sim2h(sim2h)
        // just park the main thread indefinitely...
        .block_on(futures::future::pending::<()>());
}
