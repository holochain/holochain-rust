//! `cargo run --bin sim2h_admin -- --help`

/*use lib3h_crypto_api::CryptoSystem;
use sim2h::{
    crypto::{Provenance, SignedWireMessage},
    Sim2h, WireMessage,
};*/
use structopt::StructOpt;
use url2::prelude::*;
//use lib3h_protocol::{    /*data_types::*, protocol::*,*/ uri::Lib3hUri};
use in_stream::*;
use dns_lookup::{lookup_host};

#[derive(StructOpt)]
#[structopt(name = "sim2h_admin")]
struct Opt {
    #[structopt(long)]
    /// sim2h_server url to connect to
    url: String
}



fn main() {
    let args = Opt::from_args();
    let url = match Url2::try_parse(args.url.clone()) {
        Err(e) => {
            println!("unable to parse url:{} got error: {}", args.url, e);
            return
        }
        Ok(url) => url
    };
    //let uri = Lib3hUri(url.into());
    let host = format!("{}",url.host().unwrap());
    println!("looking up: {}", host);
    let ips: Vec<std::net::IpAddr> = lookup_host(&host).unwrap();
    if ips.len() < 1 {
        println!("unable to lookup host");
        return
    }
    println!("resolved to: {}", ips[0]);
    let url = Url2::parse(format!("{}://{}:{}",url.scheme(),ips[0],url.port().unwrap()));

    println!("connecting to: {}", url);
    let _connection = await_in_stream_connect(&url);

}

fn await_in_stream_connect(connect_uri: &Url2) -> InStreamWss<InStreamTls<InStreamTcp>> {
    let timeout = std::time::Instant::now()
        .checked_add(std::time::Duration::from_millis(10000))
        .unwrap();

    let mut read_frame = WsFrame::default();

    // keep trying to connect
    loop {
        let config = WssConnectConfig::new(TlsConnectConfig::new(TcpConnectConfig::default()));
        let mut connection = InStreamWss::connect(connect_uri, config).unwrap();
        connection.write(WsFrame::Ping(b"".to_vec())).unwrap();
gs
        loop {
            let mut err = false;

            match connection.read(&mut read_frame) {
                Ok(_) => return connection,
                Err(e) if e.would_block() => (),
                Err(_) => {
                    err = true;
                }
            }

            if std::time::Instant::now() >= timeout {
                panic!("could not connect within timeout");
            }

            if err {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
