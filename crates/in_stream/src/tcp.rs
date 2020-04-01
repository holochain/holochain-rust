use crate::*;
use net2::TcpStreamExt;
use std::{
    io::{Error, ErrorKind, Read, Result, Write},
    net::{SocketAddr, ToSocketAddrs},
};
use url2::prelude::*;

const SCHEME: &'static str = "tcp";

/// internal helper convert urls to socket addrs for binding / connection
fn tcp_url_to_socket_addr(url: &Url2) -> Result<SocketAddr> {
    if url.scheme() != SCHEME || url.host_str().is_none() || url.port().is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("got: '{}', expected: '{}://host:port'", SCHEME, url),
        ));
    }

    let rendered = format!("{}:{}", url.host_str().unwrap(), url.port().unwrap());

    if let Ok(mut iter) = rendered.to_socket_addrs() {
        let mut tmp = iter.next();
        let mut fallback = None;
        loop {
            if tmp.is_none() {
                break;
            }

            if tmp.as_ref().unwrap().is_ipv4() {
                return Ok(tmp.unwrap());
            }

            fallback = tmp;
            tmp = iter.next();
        }
        if let Some(addr) = fallback {
            return Ok(addr);
        }
    }

    Err(Error::new(
        ErrorKind::InvalidInput,
        format!("could not parse '{}', as 'host:port'", rendered),
    ))
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

/// basic tcp socket server/listener
#[derive(Debug)]
pub struct InStreamListenerTcp(pub std::net::TcpListener);

impl InStreamListenerTcp {
    pub fn bind(url: &Url2, config: TcpBindConfig) -> Result<Self> {
        InStreamListenerTcp::raw_bind(url, config)
    }
}

impl InStreamListener<&mut [u8], &[u8]> for InStreamListenerTcp {
    type Stream = InStreamTcp;

    fn raw_bind<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self> {
        let config = TcpBindConfig::from_gen(config)?;
        let addr = tcp_url_to_socket_addr(url)?;
        let listener = match &addr {
            SocketAddr::V4(_) => net2::TcpBuilder::new_v4()?,
            SocketAddr::V6(_) => net2::TcpBuilder::new_v6()?,
        }
        .reuse_address(true)?
        .bind(addr)?
        .listen(config.backlog)?;
        listener.set_nonblocking(true)?;
        Ok(Self(listener))
    }

    fn binding(&self) -> Url2 {
        let local = self
            .0
            .local_addr()
            .expect("Couldn't unwrap local_addr() of TcpListener when trying to get binding URL");
        url2!("{}://{}", SCHEME, local)
    }

    fn accept(&mut self) -> Result<<Self as InStreamListener<&mut [u8], &[u8]>>::Stream> {
        let (stream, addr) = self.0.accept()?;
        stream.set_nonblocking(true)?;
        let remote_url = url2!("{}://{}", SCHEME, addr);
        log::debug!("tcp: accepted from {}", remote_url);
        InStreamTcp::priv_new(stream, remote_url, None)
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
            connect_timeout_ms: Some(20000),
        }
    }
}

impl InStreamConfig for TcpConnectConfig {}

#[derive(Debug)]
struct TcpConnectingData {
    addr: std::net::SocketAddr,
    connect_timeout: Option<std::time::Instant>,
}

/// basic tcp socket stream
#[derive(Debug)]
pub struct InStreamTcp {
    pub stream: std::net::TcpStream,
    url: Url2,
    connecting: Option<TcpConnectingData>,
    write_buf: Vec<u8>,
}

impl InStreamTcp {
    pub fn connect(url: &Url2, config: TcpConnectConfig) -> Result<Self> {
        InStreamTcp::raw_connect(url, config)
    }

    fn priv_new(
        stream: std::net::TcpStream,
        url: Url2,
        connecting: Option<TcpConnectingData>,
    ) -> Result<Self> {
        Ok(Self {
            stream,
            url,
            connecting,
            write_buf: Vec::new(),
        })
    }

    fn priv_process(&mut self) -> Result<()> {
        if let Some(cdata) = &mut self.connecting {
            match self.stream.connect(&cdata.addr) {
                Err(e) => {
                    if let Some(code) = e.raw_os_error() {
                        // `Socket is already connected` : )
                        if code == 56 {
                            self.connecting = None;
                        }
                    }
                }
                Ok(_) => {
                    self.connecting = None;
                }
            }
        }
        if let Some(cdata) = &mut self.connecting {
            if let Some(timeout) = cdata.connect_timeout {
                if std::time::Instant::now() >= timeout {
                    return Err(ErrorKind::TimedOut.into());
                }
            }
        }
        Ok(())
    }

    fn priv_write_pending(&mut self) -> Result<()> {
        if self.write_buf.is_empty() {
            return Ok(());
        }
        if self.connecting.is_some() {
            return Ok(());
        }
        let written = match self.stream.write(&self.write_buf) {
            Ok(written) => written,
            Err(e) if e.would_block() => return Ok(()),
            Err(e) => return Err(e),
        };
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
        let stream = match &addr {
            SocketAddr::V4(_) => net2::TcpBuilder::new_v4()?,
            SocketAddr::V6(_) => net2::TcpBuilder::new_v6()?,
        }
        .to_tcp_stream()?;
        stream.set_nonblocking(true)?;
        match stream.connect(addr) {
            Err(_) => Self::priv_new(
                stream,
                url.clone(),
                Some(TcpConnectingData {
                    addr,
                    connect_timeout: config.connect_timeout_ms.map(|ms| {
                        std::time::Instant::now()
                            .checked_add(std::time::Duration::from_millis(ms))
                            .unwrap()
                    }),
                }),
            ),
            Ok(_) => Self::priv_new(stream, url.clone(), None),
        }
    }

    fn check_ready(&mut self) -> Result<bool> {
        self.priv_process()?;
        Ok(self.connecting.is_none())
    }

    fn remote_url(&self) -> Url2 {
        self.url.clone()
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        self.priv_process()?;
        self.priv_write_pending()?;
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
                let written = match self.stream.write(data) {
                    Ok(written) => written,
                    Err(e) if e.would_block() => {
                        self.write_buf.extend_from_slice(data);
                        return Ok(data.len());
                    }
                    Err(e) => return Err(e),
                };
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
        // TODO - flush config with potential timeout?
        loop {
            self.priv_process()?;
            if self.connecting.is_none() {
                self.priv_write_pending()?;
                self.stream.flush()?;
            }
            if self.write_buf.is_empty() {
                return Ok(());
            }
            std::thread::yield_now();
        }
    }
}

impl InStreamStd for InStreamTcp {}

impl Drop for InStreamTcp {
    fn drop(&mut self) {
        log::warn!("dropping tcp stream {:?}", &self.url)
    }
}

impl std::ops::Deref for InStreamTcp {
    type Target = std::net::TcpStream;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl std::ops::DerefMut for InStreamTcp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suite(bind: &str, con: Option<&str>) {
        let bind = bind.to_string();
        let con = con.map(|c| c.to_string());

        let (send_binding, recv_binding) = crossbeam_channel::bounded(CHANNEL_SIZE);

        let server_thread = std::thread::spawn(move || {
            let mut listener =
                InStreamListenerTcp::bind(&Url2::parse(bind), TcpBindConfig::default()).unwrap();
            println!("bound to: {}", listener.binding());
            let binding = match con {
                None => listener.binding(),
                Some(c) => {
                    let mut url = url2!("{}", c);
                    let port = listener.binding().port();
                    url.set_port(port).unwrap();
                    url
                }
            };
            send_binding.send(binding).unwrap();

            let mut srv = loop {
                match listener.accept() {
                    Ok(srv) => break srv,
                    Err(e) if e.would_block() => {
                        std::thread::yield_now();
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
            .into_std_stream();

            let rurl = srv.remote_url();
            assert_ne!(listener.binding(), rurl);
            assert_eq!(SCHEME, rurl.scheme());

            srv.write(b"hello from server").unwrap();
            srv.flush().unwrap();
            srv.shutdown(std::net::Shutdown::Write).unwrap();

            let mut res = String::new();
            loop {
                match srv.read_to_string(&mut res) {
                    Ok(_) => break,
                    Err(e) if e.would_block() => {
                        std::thread::yield_now();
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
            assert_eq!("hello from client", &res);
        });

        let client_thread = std::thread::spawn(move || {
            let binding = recv_binding.recv().unwrap();
            println!("connect to: {}", binding);

            let mut cli = InStreamTcp::connect(&binding, TcpConnectConfig::default())
                .unwrap()
                .into_std_stream();

            assert_eq!(binding.as_str(), cli.remote_url().as_str());

            cli.write(b"hello from client").unwrap();
            cli.flush().unwrap();
            cli.shutdown(std::net::Shutdown::Write).unwrap();

            let mut res = String::new();
            loop {
                match cli.read_to_string(&mut res) {
                    Ok(_) => break,
                    Err(e) if e.would_block() => {
                        std::thread::yield_now();
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

    #[test]
    fn tcp_v4_works() {
        suite("tcp://127.0.0.1:0", None);
    }

    #[test]
    #[ignore] // our CI doesn't support "localhost" dns resolution : (
    fn tcp_v4_local_works() {
        suite("tcp://127.0.0.1:0", Some("tcp://localhost:0"));
    }

    #[test]
    #[ignore] // our CI doesn't support v6 loopback : (
    fn tcp_v6_works() {
        suite("tcp://[::1]:0", None);
    }
}
