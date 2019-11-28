use crate::*;
use net2::TcpStreamExt;
use std::io::{Error, ErrorKind, Read, Result, Write};
use url2::prelude::*;

const SCHEME: &'static str = "tcp";

/// internal helper convert urls to socket addrs for binding / connection
fn tcp_url_to_socket_addr(url: &Url2) -> Result<std::net::SocketAddr> {
    if url.scheme() != SCHEME || url.host_str().is_none() || url.port().is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("got: '{}', expected: '{}://host:port'", SCHEME, url),
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

#[derive(Debug)]
/// configuration options for the listener bind call
pub struct TcpBindConfig {
    pub backlog: i32,
}

impl Default for TcpBindConfig {
    fn default() -> Self {
        Self { backlog: 1024 }
    }
}

impl InStreamConfig for TcpBindConfig {}

#[derive(Debug)]
pub struct InStreamListenerTcp(pub std::net::TcpListener);

impl InStreamListener<&mut [u8], &[u8]> for InStreamListenerTcp {
    type Stream = InStreamTcp;

    fn raw_bind<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self> {
        let config = TcpBindConfig::from_gen(config)?;
        let addr = tcp_url_to_socket_addr(url)?;
        let listener = net2::TcpBuilder::new_v4()?
            .bind(addr)?
            .listen(config.backlog)?;
        listener.set_nonblocking(true)?;
        Ok(Self(listener))
    }

    fn binding(&self) -> Url2 {
        let local = self.0.local_addr().unwrap();
        Url2::parse(&format!("{}://{}:{}", SCHEME, local.ip(), local.port()))
    }

    fn accept(&mut self) -> Result<<Self as InStreamListener<&mut [u8], &[u8]>>::Stream> {
        let (stream, _addr) = self.0.accept()?;
        stream.set_nonblocking(true)?;
        InStreamTcp::priv_new(stream, None)
    }
}

impl InStreamListenerStd for InStreamListenerTcp {
    type StreamStd = InStreamTcp;

    fn accept_std(&mut self) -> Result<<Self as InStreamListenerStd>::StreamStd> {
        self.accept()
    }
}

#[derive(Debug)]
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

impl InStreamConfig for TcpConnectConfig {}

#[derive(Debug)]
struct TcpConnectingData {
    addr: std::net::SocketAddr,
    connect_timeout: Option<std::time::Instant>,
}

#[derive(Shrinkwrap, Debug)]
#[shrinkwrap(mutable)]
pub struct InStreamTcp {
    #[shrinkwrap(main_field)]
    pub stream: std::net::TcpStream,
    connecting: Option<TcpConnectingData>,
    write_buf: Vec<u8>,
}

impl InStreamTcp {
    fn priv_new(
        stream: std::net::TcpStream,
        connecting: Option<TcpConnectingData>,
    ) -> Result<Self> {
        Ok(Self {
            stream,
            connecting,
            write_buf: Vec::new(),
        })
    }

    fn priv_process(&mut self) -> Result<()> {
        if let Some(cdata) = &mut self.connecting {
            if let Ok(_) = self.stream.connect(&cdata.addr) {
                self.connecting = None;
            } else {
                if let Some(timeout) = cdata.connect_timeout {
                    if std::time::Instant::now() >= timeout {
                        return Err(ErrorKind::TimedOut.into());
                    }
                }
            }
        }
        Ok(())
    }

    fn priv_write_pending(&mut self) -> Result<()> {
        if self.connecting.is_some() {
            return Ok(());
        }
        let written = self.stream.write(&self.write_buf)?;
        assert_eq!(written, self.write_buf.drain(..written).count());
        Ok(())
    }
}

impl InStream<&mut [u8], &[u8]> for InStreamTcp {
    /// tcp streams should use urls like tcp://
    const URL_SCHEME: &'static str = SCHEME;

    fn raw_connect<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self> {
        let config = TcpConnectConfig::from_gen(config)?;
        let addr = tcp_url_to_socket_addr(url)?;
        let stream = net2::TcpBuilder::new_v4()?.to_tcp_stream()?;
        stream.set_nonblocking(true)?;
        match stream.connect(addr) {
            Err(_) => Self::priv_new(
                stream,
                Some(TcpConnectingData {
                    addr,
                    connect_timeout: match config.connect_timeout_ms {
                        None => None,
                        Some(ms) => Some(
                            std::time::Instant::now()
                                .checked_add(std::time::Duration::from_millis(ms))
                                .unwrap(),
                        ),
                    },
                }),
            ),
            Ok(_) => Self::priv_new(stream, None),
        }
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        self.priv_process()?;
        if self.connecting.is_none() {
            self.stream.read(data)
        } else {
            Err(Error::with_would_block())
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.priv_process()?;
        if self.connecting.is_none() {
            if self.write_buf.is_empty() {
                // in the 99% case we can just write without buffering
                let written = self.stream.write(data)?;
                if written < data.len() {
                    self.write_buf.extend_from_slice(&data[written..]);
                }
                Ok(data.len())
            } else {
                // if we already have a buffer, append to it and
                // try to write the whole thing
                self.write_buf.extend_from_slice(data);
                self.priv_write_pending()?;
                Ok(data.len())
            }
        } else {
            self.write_buf.extend_from_slice(data);
            Ok(data.len())
        }
    }

    fn flush(&mut self) -> Result<()> {
        loop {
            self.priv_process()?;
            if self.connecting.is_none() {
                self.priv_write_pending()?;
                self.stream.flush()?;
            }
            if self.write_buf.is_empty() {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

impl InStreamStd for InStreamTcp {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tcp_works() {
        let (send_binding, recv_binding) = crossbeam_channel::unbounded();

        let server_thread = std::thread::spawn(move || {
            let mut listener = InStreamListenerTcp::raw_bind(
                &Url2::parse("tcp://127.0.0.1:0"),
                TcpBindConfig::default(),
            )
            .unwrap();
            println!("bound to: {}", listener.binding());
            send_binding.send(listener.binding()).unwrap();

            let mut srv = loop {
                match listener.accept() {
                    Ok(srv) => break srv,
                    Err(e) if e.would_block() => {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
            .into_std_stream();

            srv.write(b"hello from server").unwrap();
            srv.flush().unwrap();
            srv.shutdown(std::net::Shutdown::Write).unwrap();

            let mut res = String::new();
            loop {
                match srv.read_to_string(&mut res) {
                    Ok(_) => break,
                    Err(e) if e.would_block() => {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
            assert_eq!("hello from client", &res);
        });

        let client_thread = std::thread::spawn(move || {
            let binding = recv_binding.recv().unwrap();
            println!("connect to: {}", binding);

            let mut cli = InStreamTcp::raw_connect(&binding, TcpConnectConfig::default())
                .unwrap()
                .into_std_stream();

            cli.write(b"hello from client").unwrap();
            cli.flush().unwrap();
            cli.shutdown(std::net::Shutdown::Write).unwrap();

            let mut res = String::new();
            loop {
                match cli.read_to_string(&mut res) {
                    Ok(_) => break,
                    Err(e) if e.would_block() => {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
            assert_eq!("hello from server", &res);
        });

        server_thread.join().unwrap();
        client_thread.join().unwrap();

        println!("done");
    }
}
