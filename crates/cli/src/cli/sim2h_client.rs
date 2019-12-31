use dns_lookup::lookup_host;
use in_stream::*;
use sim2h::WireMessage;
use sim2h_client::Sim2hClient;
use url2::prelude::*;

pub fn sim2h_client(url_string: String, message_string: String) -> Result<(), String> {
    let url = match Url2::try_parse(url_string.clone()) {
        Err(e) => Err(format!(
            "unable to parse url:{} got error: {}",
            url_string, e
        ))?,
        Ok(url) => url,
    };
    let host = format!("{}", url.host().unwrap());
    let ip = if host == "localhost" {
        "127.0.0.1".to_string()
    } else {
        println!("looking up: {}", host);
        let ips: Vec<std::net::IpAddr> = lookup_host(&host).map_err(|e| format!("{}", e))?;
        println!("resolved to: {}", ips[0]);
        format!("{}", ips[0])
    };
    let maybe_port = url.port();
    if maybe_port.is_none() {
        return Err(format!("expecting port in url, got: {}", url));
    }
    let url = Url2::parse(format!("{}://{}:{}", url.scheme(), ip, maybe_port.unwrap()));

    println!("connecting to: {}", url);
    let mut client = Sim2hClient::new(&url)?;
    client.send_wire(match message_string.as_ref() {
        "ping" => WireMessage::Ping,
        "status" => WireMessage::Status,
        _ => {
            return Err(format!(
                "expecting 'ping' or 'status' for message, got: {}",
                message_string
            ))
        }
    });
    let timeout = std::time::Instant::now()
        .checked_add(std::time::Duration::from_millis(60000))
        .unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
        let mut frame = WsFrame::default();
        match client.connection().read(&mut frame) {
            Ok(_) => {
                if let WsFrame::Binary(b) = frame {
                    let msg: WireMessage = serde_json::from_slice(&b).unwrap();
                    println!("{:?}", msg);
                    break;
                } else {
                    Err(format!("unexpected {:?}", frame))?;
                }
            }
            Err(e) if e.would_block() => (),
            Err(e) => Err(format!("{}", e))?,
        }
        if std::time::Instant::now() >= timeout {
            Err(format!("timeout waiting for status response from {}", host))?;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}
