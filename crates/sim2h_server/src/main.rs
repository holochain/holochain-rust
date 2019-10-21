extern crate structopt;

use lib3h::transport::websocket::{
    streams::*,
    tls::{TlsCertificate, TlsConfig},
};
use lib3h_protocol::{
    types::{NetworkHash, NodePubKey},
    uri::Builder,
};
use log::error;
use sim2h::{Sim2h, MESSAGE_LOGGER};
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
        help = "CSV file to log all incoming and outgoing messages to"
    )]
    message_log_file: Option<PathBuf>,
}

fn create_stream_manager() -> StreamManager<std::net::TcpStream> {
    let tls_config = TlsConfig::SuppliedCertificate(TlsCertificate::build_from_entropy());
    StreamManager::with_std_tcp_stream(tls_config)
}

fn main() {
    env_logger::init();
    let transport = create_stream_manager();

    let args = Cli::from_args();

    let host = "wss://0.0.0.0/";
    let uri = Builder::with_raw_url(host)
        .unwrap_or_else(|e| panic!("with_raw_url: {:?}", e))
        .with_port(args.port)
        .build();
    if let Some(message_log_file) = args.message_log_file {
        MESSAGE_LOGGER.lock().set_logfile(message_log_file);
        MESSAGE_LOGGER.lock().start();
    }

    let mut sim2h = Sim2h::new(transport, uri);

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
}
