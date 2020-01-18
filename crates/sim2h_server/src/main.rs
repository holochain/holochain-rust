extern crate crossbeam_channel;
extern crate lib3h_sodium;
extern crate structopt;
#[macro_use]
extern crate log;
extern crate holochain_tracing as ht;
// #[macro_use]
// extern crate holochain_tracing_macros;

use lib3h_protocol::uri::Builder;
use lib3h_sodium::SodiumCryptoSystem;
use log::error;
use sim2h::{DhtAlgorithm, Sim2h, MESSAGE_LOGGER};
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
    #[structopt(
        long,
        short,
        help = "URL of a Jaeger server to send tracing spans to. No tracing if not specified."
    )]
    _tracing_url: Option<PathBuf>,
}

fn main() {
    env_logger::init();

    let args = Cli::from_args();

    let tracer = {
        let (span_tx, span_rx) = crossbeam_channel::unbounded();
        let _ = std::thread::Builder::new()
            .name("tracer_loop".to_string())
            .spawn(move || {
                info!("Tracer loop started.");
                // TODO: killswitch
                let reporter = ht::Reporter::new("sim2h-server").unwrap();
                for span in span_rx {
                    reporter.report(&[span]).expect("could not report span");
                }
            });
        ht::Tracer::with_sender(ht::AllSampler, span_tx)
    };

    let host = "ws://0.0.0.0/";
    let uri = Builder::with_raw_url(host)
        .unwrap_or_else(|e| panic!("with_raw_url: {:?}", e))
        .with_port(args.port)
        .build();
    if let Some(message_log_file) = args.message_log_file {
        MESSAGE_LOGGER.lock().set_logfile(message_log_file);
        MESSAGE_LOGGER.lock().start();
    }

    let mut sim2h = Sim2h::new(Box::new(SodiumCryptoSystem::new()), uri, Some(tracer));
    if args.sharding > 0 {
        sim2h.set_dht_algorithm(DhtAlgorithm::NaiveSharding {
            redundant_count: args.sharding,
        });
    }

    loop {
        let result = sim2h.process();
        match result {
            Err(e) => {
                if e.to_string().contains("Bind error:") {
                    println!("{:?}", e);
                    exit(1)
                } else {
                    error!("{}", e.to_string())
                }
            }
            Ok(false) => {
                // if no work sleep a little
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            _ => (),
        }
    }
}
