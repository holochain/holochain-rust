extern crate holochain_tracing as ht;
extern crate lib3h_sodium;
extern crate log;
extern crate newrelic;
extern crate structopt;
#[macro_use(new_relic_setup)]
extern crate holochain_common;

use lib3h_protocol::uri::Builder;
use lib3h_sodium::SodiumCryptoSystem;
use log::*;
use newrelic::{LogLevel, LogOutput, NewRelicConfig};
use sim2h::{run_sim2h, DhtAlgorithm, MESSAGE_LOGGER};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
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
        help = "The service name to use for Jaeger tracing spans. No tracing is done if not specified."
    )]
    tracing_name: Option<String>,

    #[structopt(long, help = "Outputs structured json from logging", default_value = "Json")]
    structured: ht::structured::Output,
}

new_relic_setup!("NEW_RELIC_LICENSE_KEY");
#[holochain_tracing_macros::newrelic_autotrace(SIM2H_SERVER)]
fn main() {
    //this set up new relic needs
    NewRelicConfig::default()
        .logging(LogLevel::Error, LogOutput::StdErr)
        .init()
        .unwrap_or_else(|_| warn!("Could not configure new relic daemon"));
    //env_logger::init();
    let args = Cli::from_args();

    /*
    let tracer = if let Some(service_name) = args.tracing_name {
        let (span_tx, span_rx) = crossbeam_channel::unbounded();
        let _ = std::thread::Builder::new()
            .name("tracer_loop".to_string())
            .spawn(move || {
                info!("Tracer loop started.");
                // TODO: killswitch
                let reporter = ht::reporter::JaegerBinaryReporter::new(&service_name).unwrap();
                for span in span_rx {
                    reporter.report(&[span]).expect("could not report span");
                }
            });
        Some(ht::Tracer::with_sender(ht::AllSampler, span_tx))
    } else {
        None
    };
    */
    let tracer = if let Some(service_name) = args.tracing_name {
        ht::tracing::init(service_name).expect("Failed to start tracing");
        None
    } else {
        ht::structured::init_fmt(args.structured).expect("Failed to start structed tracing");
        tracing_log::LogTracer::init().expect("Failed to init tracing log");
        None
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

    let (mut rt, _) = run_sim2h(
        Box::new(SodiumCryptoSystem::new()),
        uri,
        DhtAlgorithm::NaiveSharding {
            redundant_count: args.sharding,
        },
        tracer,
    );

    // just park the main thread indefinitely...
    rt.block_on(futures::future::pending::<()>());
}
