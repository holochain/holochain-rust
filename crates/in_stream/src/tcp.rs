use crate::*;
use net2::TcpStreamExt;
use std::io::{Error, ErrorKind, Read, Result, Write};
use url2::prelude::*;

/// internal helper convert urls to socket addrs for binding / connection
fn tcp_url_to_socket_addr(url: &Url2) -> Result<std::net::SocketAddr> {
    if url.scheme() != "tcp" || url.host_str().is_none() || url.port().is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("got: '{}', expected: 'tcp://host:port'", url),
        ));
    }
    let rendered = format!("{}:{}", url.host_str().unwrap(), url.port().unwrap());
    match rendered.parse() {
        Ok(addr) => Ok(addr),
        Err(_) => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("could not parse '{}', as 'host:port'", rendered),
        )),
    }
}

/// a tcp listener allows you to receive tcp connections on a network interface
#[derive(Shrinkwrap, Debug)]
#[shrinkwrap(mutable)]
pub struct InStreamListenerTcp(pub std::net::TcpListener);

/// configuration options for the listener bind call
pub struct TcpBindConfig {
    pub backlog: i32,
}

impl Default for TcpBindConfig {
    fn default() -> Self {
        Self { backlog: 1024 }
    }
}

impl InStreamListener for InStreamListenerTcp {
    type Partial = InStreamPartialTcp;
    type BindConfig = TcpBindConfig;

    /// bind to a network interface
    fn bind(url: &Url2, config: Self::BindConfig) -> Result<Self> {
        let addr = tcp_url_to_socket_addr(url)?;
        let listener = net2::TcpBuilder::new_v4()?
            .bind(addr)?
            .listen(config.backlog)?;
        listener.set_nonblocking(true)?;
        Ok(Self(listener))
    }

    /// get the bound interface url
    fn binding(&self) -> Url2 {
        let local = self.0.local_addr().unwrap();
        Url2::parse(&format!("tcp://{}:{}", local.ip(), local.port()))
    }

    /// accept a connection from this listener
    fn accept(&mut self) -> Result<<Self as InStreamListener>::Partial> {
        let (stream, _addr) = self.0.accept()?;
        stream.set_nonblocking(true)?;
        Ok(InStreamPartialTcp::with_stream(InStreamTcp(stream))?)
    }
}

/// represents a partial tcp connection, there may still be handshaking to do
#[derive(Debug)]
pub struct InStreamPartialTcp {
    stream: Option<InStreamTcp>,
    addr: String,
    is_connecting: bool,
    connect_timeout: Option<std::time::Instant>,
}

/// configuration options for tcp connect
pub struct TcpConnectConfig {
    pub connect_timeout_ms: Option<u64>,
}

impl Default for TcpConnectConfig {
    fn default() -> Self {
        Self {
            connect_timeout_ms: Some(5000),
        }
    }
}

impl InStreamPartial for InStreamPartialTcp {
    type Stream = InStreamTcp;
    type ConnectConfig = TcpConnectConfig;

    /// tcp streams expect urls like tcp://
    const URL_SCHEME: &'static str = "tcp";

    /// convert a full stream back into a partial one
    fn with_stream(stream: Self::Stream) -> Result<Self> {
        Ok(Self {
            stream: Some(stream),
            addr: "".to_string(),
            is_connecting: false,
            connect_timeout: None,
        })
    }

    /// establish a tcp connection to a remote listener
    fn connect(url: &Url2, config: Self::ConnectConfig) -> Result<Self> {
        let addr = tcp_url_to_socket_addr(url)?;
        let stream = net2::TcpBuilder::new_v4()?.to_tcp_stream()?;
        stream.set_nonblocking(true)?;
        let is_connecting = match stream.connect(addr) {
            Err(_) => true,
            Ok(_) => false,
        };
        let connect_timeout = match config.connect_timeout_ms {
            None => None,
            Some(ms) => Some(
                std::time::Instant::now()
                    .checked_add(std::time::Duration::from_millis(ms))
                    .unwrap(),
            ),
        };
        Ok(Self {
            stream: Some(InStreamTcp(stream)),
            addr: addr.to_string(),
            is_connecting,
            connect_timeout,
        })
    }

    /// take a step attempting to finish any needed handshaking
    /// will return a full stream if ready
    fn process(&mut self) -> Result<Self::Stream> {
        match &mut self.stream {
            None => Err(Error::new(ErrorKind::NotFound, "raw stream is None")),
            Some(stream) => {
                if self.is_connecting {
                    if let Ok(_) = stream.0.connect(&self.addr) {
                        self.is_connecting = false;
                    };
                }

                if self.is_connecting {
                    if let Some(timeout) = self.connect_timeout {
                        if std::time::Instant::now() >= timeout {
                            return Err(ErrorKind::TimedOut.into());
                        }
                    }
                    Err(Error::with_would_block())
                } else {
                    Ok(std::mem::replace(&mut self.stream, None).unwrap())
                }
            }
        }
    }
}

/// a tcp connection to a remote node
#[derive(Shrinkwrap, Debug)]
#[shrinkwrap(mutable)]
pub struct InStreamTcp(pub std::net::TcpStream);

impl InStream for InStreamTcp {}

impl Read for InStreamTcp {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl Write for InStreamTcp {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tcp_works() {
        let mut l =
            InStreamListenerTcp::bind(&Url2::parse("tcp://127.0.0.1:0"), TcpBindConfig::default())
                .unwrap();
        println!("bound to: {}", l.binding());

        let mut c = InStreamPartialTcp::connect(&l.binding(), TcpConnectConfig::default()).unwrap();

        let mut srv = l.accept_blocking().unwrap().process_blocking().unwrap();

        let mut cli = c.process_blocking().unwrap();

        let mut buf = [0; 32];

        srv.write_all(b"hello from server").unwrap();
        cli.write_all(b"hello from client").unwrap();

        std::thread::sleep(std::time::Duration::from_millis(100));

        assert_eq!(17, srv.read(&mut buf).unwrap());
        assert_eq!("hello from client", &String::from_utf8_lossy(&buf[..17]));
        assert_eq!(17, cli.read(&mut buf).unwrap());
        assert_eq!("hello from server", &String::from_utf8_lossy(&buf[..17]));

        println!("done");
    }
}
