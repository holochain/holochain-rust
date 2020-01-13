//! abstraction for working with Websocket connections
//! TcpStream specific functions
use lib3h::transport::error::{ErrorKind, TransportError, TransportResult};

use crate::websocket::{
    streams::{Acceptor, Bind, StreamManager},
    tls::TlsConfig,
    wss_info::WssInfo,
};
use log::*;
use url2::prelude::*;

use std::net::{TcpListener, TcpStream};

#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
impl StreamManager<std::net::TcpStream> {
    /// convenience constructor for creating a websocket "Transport"
    /// instance that is based of the rust std TcpStream
    pub fn with_std_tcp_stream(tls_config: TlsConfig) -> Self {
        let bind: Bind<TcpStream> = Box::new(|url| Self::tcp_bind(url));
        StreamManager::new(
            |uri| {
                let socket = std::net::TcpStream::connect(uri)?;
                socket.set_nonblocking(true)?;
                Ok(socket)
            },
            bind,
            tls_config,
        )
    }

    fn tcp_bind(url: &url::Url) -> TransportResult<(Url2, Acceptor<TcpStream>)> {
        // TODO return transport result rather than expect()
        let host = url.host_str().expect("host name must be supplied");
        let port = url.port().unwrap_or(80); // TODO default or error here?
        let formatted_url = format!("{}:{}", host, port);
        trace!("websocket tcp_bind with url: {}", formatted_url);
        TcpListener::bind(formatted_url)
            .map_err(|err| err.into())
            .and_then(move |listener: TcpListener| {
                let new_url = listener.local_addr()?;
                let new_url = Url2::parse(&format!(
                    "{}://{}:{}",
                    url.scheme(),
                    new_url.ip(),
                    new_url.port(),
                ));
                listener
                    .set_nonblocking(true)
                    .map_err(|err| {
                        error!("transport_wss::tcp listener error: {:?}", err);
                        err.into()
                    })
                    .map(|()| {
                        let acceptor: Acceptor<TcpStream> = Box::new(move || {
                            listener
                                .accept()
                                .map_err(|err| match err.kind() {
                                    std::io::ErrorKind::WouldBlock => {
                                        TransportError::new_kind(ErrorKind::Ignore(err.to_string()))
                                    }
                                    _ => {
                                        error!("transport_wss::tcp accept error: {:?}", err);
                                        err.into()
                                    }
                                })
                                .and_then(|(tcp_stream, socket_address)| {
                                    tcp_stream.set_nonblocking(true)?;
                                    let v4_socket_address = format!(
                                        "ws://{}:{}",
                                        socket_address.ip(),
                                        socket_address.port()
                                    );

                                    trace!(
                                        "transport_wss::tcp v4 socket_address: {}",
                                        v4_socket_address
                                    );
                                    url::Url::parse(v4_socket_address.as_str())
                                        .map(|url| {
                                            trace!(
                                                "transport_wss::tcp accepted for url {}",
                                                url.clone()
                                            );
                                            WssInfo::server(url, tcp_stream)
                                        })
                                        .map_err(|err| {
                                            error!("transport_wss::tcp url error: {:?}", err);
                                            err.into()
                                        })
                                })
                        });
                        (new_url, acceptor)
                    })
            })
    }
}
