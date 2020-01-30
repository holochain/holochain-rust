extern crate lib3h_sodium;
extern crate newrelic;
extern crate structopt;

use lib3h_protocol::uri::Builder;
use lib3h_sodium::SodiumCryptoSystem;
use log::*;
use newrelic::{LogLevel, LogOutput, NewRelicConfig};
use sim2h::{run_sim2h, DhtAlgorithm, MESSAGE_LOGGER};
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

#[holochain_tracing_macros::newrelic_autotrace(SIM2H_SERVER)]
fn main() {
    NewRelicConfig::default()
        .logging(LogLevel::Error, LogOutput::StdErr)
        .init()
        .unwrap_or_else(|_| warn!("Could not configure new relic daemon"));
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

    let (mut rt, _) = run_sim2h(
        Box::new(SodiumCryptoSystem::new()),
        uri,
        DhtAlgorithm::NaiveSharding {
            redundant_count: args.sharding,
        },
    );

    // just park the main thread indefinitely...
    rt.block_on(futures::future::pending::<()>());
}
