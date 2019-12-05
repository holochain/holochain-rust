use crate::*;
use std::io::{Error, ErrorKind, Read, Result, Write};
use url2::prelude::*;

mod certificate;
pub use certificate::*;

const SCHEME: &'static str = "tls";

/// internal helper make sure we're dealing with tls:// urls
fn validate_url_scheme(url: &Url2) -> Result<()> {
    if url.scheme() != SCHEME {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("got: '{}', expected: '{}://...'", SCHEME, url),
        ));
    }
    Ok(())
}

#[derive(Debug)]
/// tls specific bind config
pub struct TlsBindConfig {
    pub tls_certificate: Option<TlsCertificate>,
    pub sub_bind_config: InStreamConfigAny,
}

impl TlsBindConfig {
    pub fn new<Sub: InStreamConfig>(sub_config: Sub) -> Self {
        Self {
            tls_certificate: None,
            sub_bind_config: sub_config.to_any(),
        }
    }

    pub fn fake_certificate(mut self) -> Self {
        self.tls_certificate = Some(TlsCertificate::with_fake_certificate());
        self
    }
}

impl InStreamConfig for TlsBindConfig {}

/// bind to a network interface to listen for TLS connections
pub struct InStreamListenerTls<Sub: InStreamListenerStd> {
    sub: Sub,
    acceptor: native_tls::TlsAcceptor,
}

impl<Sub: InStreamListenerStd> InStreamListenerTls<Sub> {
    pub fn bind(url: &Url2, config: TlsBindConfig) -> Result<Self> {
        InStreamListenerTls::raw_bind(url, config)
    }
}

impl<Sub: InStreamListenerStd> std::fmt::Debug for InStreamListenerTls<Sub> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InStreamListenerTls")
            .field("sub", &self.sub)
            .finish()
    }
}

impl<Sub: InStreamListenerStd> InStreamListener<&mut [u8], &[u8]> for InStreamListenerTls<Sub> {
    type Stream = InStreamTls<Sub::StreamStd>;

    fn raw_bind<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self> {
        let config = TlsBindConfig::from_gen(config)?;
        validate_url_scheme(url)?;
        let id = native_tls::Identity::from_pkcs12(
            &config.tls_certificate.as_ref().unwrap().pkcs12_data,
            &config.tls_certificate.as_ref().unwrap().passphrase,
        )
        .unwrap();
        let acceptor = native_tls::TlsAcceptor::new(id).unwrap();
        let mut url = url.clone();
        url.set_scheme(Sub::StreamStd::URL_SCHEME).unwrap();
        let sub = Sub::raw_bind(&url, config.sub_bind_config)?;
        Ok(Self { sub, acceptor })
    }

    /// get our bound address
    fn binding(&self) -> Url2 {
        let mut url = self.sub.binding();
        url.set_scheme(SCHEME).unwrap();
        url
    }

    /// accept an incoming connection
    fn accept(&mut self) -> Result<<Self as InStreamListener<&mut [u8], &[u8]>>::Stream> {
        // get e.g. an InStreamTcp
        let stream: Sub::StreamStd = self.sub.accept_std()?;

        let res = self.acceptor.accept(stream.into_std_stream());
        let mut out = InStreamTls::priv_new();
        match out.priv_proc_tls_result(res) {
            Ok(_) => Ok(out),
            Err(e) if e.would_block() => Ok(out),
            Err(e) => Err(e),
        }
    }
}

impl<Sub: InStreamListenerStd> InStreamListenerStd for InStreamListenerTls<Sub> {
    type StreamStd = InStreamTls<Sub::StreamStd>;

    fn accept_std(&mut self) -> Result<<Self as InStreamListenerStd>::StreamStd> {
        self.accept()
    }
}

#[derive(Debug)]
/// tls specific connection config
pub struct TlsConnectConfig {
    pub sub_connect_config: InStreamConfigAny,
}

impl TlsConnectConfig {
    pub fn new<Sub: InStreamConfig>(sub_config: Sub) -> Self {
        Self {
            sub_connect_config: sub_config.to_any(),
        }
    }
}

impl InStreamConfig for TlsConnectConfig {}

#[derive(Debug)]
enum TlsState<Sub: InStreamStd> {
    MidHandshake(native_tls::MidHandshakeTlsStream<StdStreamAdapter<Sub>>),
    Ready(native_tls::TlsStream<StdStreamAdapter<Sub>>),
}

/// basic tls wrapper stream
pub struct InStreamTls<Sub: InStreamStd> {
    state: Option<TlsState<Sub>>,
    write_buf: Vec<u8>,
}

impl<Sub: InStreamStd> std::fmt::Debug for InStreamTls<Sub> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InStreamTls")
            .field("state", &self.state)
            .field("write_buf", &format!("{} bytes", self.write_buf.len()))
            .finish()
    }
}

impl<Sub: InStreamStd> InStreamTls<Sub> {
    pub fn connect(url: &Url2, config: TlsConnectConfig) -> Result<Self> {
        InStreamTls::raw_connect(url, config)
    }

    fn priv_new() -> Self {
        Self {
            state: None,
            write_buf: Vec::new(),
        }
    }

    fn priv_proc_tls_result(
        &mut self,
        result: std::result::Result<
            native_tls::TlsStream<StdStreamAdapter<Sub>>,
            native_tls::HandshakeError<StdStreamAdapter<Sub>>,
        >,
    ) -> Result<()> {
        match result {
            Ok(tls) => {
                self.state = Some(TlsState::Ready(tls));
                Ok(())
            }
            Err(e) => match e {
                native_tls::HandshakeError::WouldBlock(mid) => {
                    self.state = Some(TlsState::MidHandshake(mid));
                    Err(Error::with_would_block())
                }
                native_tls::HandshakeError::Failure(e) => {
                    Err(Error::new(ErrorKind::ConnectionRefused, format!("{:?}", e)))
                }
            },
        }
    }

    fn priv_process(&mut self) -> Result<()> {
        if self.state.is_none() {
            return Ok(());
        }

        if let TlsState::Ready(_) = self.state.as_mut().unwrap() {
            return Ok(());
        }

        let mid = match self.state.take().unwrap() {
            TlsState::MidHandshake(mid) => mid,
            _ => unreachable!(),
        };

        match self.priv_proc_tls_result(mid.handshake()) {
            Ok(_) => Ok(()),
            Err(e) if e.would_block() => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn priv_write_pending(&mut self) -> Result<()> {
        if self.write_buf.is_empty() {
            return Ok(());
        }
        match &mut self.state {
            None => Err(ErrorKind::NotConnected.into()),
            Some(state) => {
                if let TlsState::Ready(tls) = state {
                    let written = match tls.write(&self.write_buf) {
                        Ok(written) => written,
                        Err(e) if e.would_block() => return Ok(()),
                        Err(e) => return Err(e),
                    };
                    assert_eq!(written, self.write_buf.drain(..written).count());
                }
                Ok(())
            }
        }
    }
}

impl<Sub: InStreamStd> InStream<&mut [u8], &[u8]> for InStreamTls<Sub> {
    const URL_SCHEME: &'static str = SCHEME;

    fn raw_connect<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self> {
        let config = TlsConnectConfig::from_gen(config)?;
        validate_url_scheme(url)?;
        let mut url = url.clone();
        url.set_scheme(Sub::URL_SCHEME).unwrap();
        let sub = Sub::raw_connect(&url, config.sub_connect_config)?;
        let mut out = Self::priv_new();
        match out.priv_proc_tls_result(
            native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()
                .unwrap()
                .connect("none:", sub.into_std_stream()),
        ) {
            Ok(_) => Ok(out),
            Err(e) if e.would_block() => Ok(out),
            Err(e) => Err(e),
        }
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        self.priv_process()?;
        self.priv_write_pending()?;
        match &mut self.state {
            None => Err(ErrorKind::NotConnected.into()),
            Some(state) => match state {
                TlsState::MidHandshake(_) => Err(Error::with_would_block()),
                TlsState::Ready(tls) => tls.read(data),
            },
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.priv_process()?;
        match &mut self.state {
            None => Err(ErrorKind::NotConnected.into()),
            Some(state) => match state {
                TlsState::MidHandshake(_) => {
                    self.write_buf.extend_from_slice(data);
                    Ok(data.len())
                }
                TlsState::Ready(tls) => {
                    if self.write_buf.is_empty() {
                        let written = match tls.write(data) {
                            Ok(written) => written,
                            Err(e) if e.would_block() => {
                                self.write_buf.extend_from_slice(data);
                                return Ok(data.len());
                            }
                            Err(e) => return Err(e),
                        };
                        if written < data.len() {
                            self.write_buf.extend_from_slice(&data[..written]);
                        }
                        Ok(data.len())
                    } else {
                        self.write_buf.extend_from_slice(data);
                        self.priv_write_pending()?;
                        Ok(data.len())
                    }
                }
            },
        }
    }

    fn flush(&mut self) -> Result<()> {
        loop {
            self.priv_process()?;
            self.priv_write_pending()?;
            if let Some(TlsState::Ready(tls)) = &mut self.state {
                tls.flush()?;
            }
            if self.write_buf.is_empty() {
                return Ok(());
            }
            std::thread::yield_now();
        }
    }
}

impl<Sub: InStreamStd> InStreamStd for InStreamTls<Sub> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_count<S: 'static + InStreamStd>(s: &mut StdStreamAdapter<S>, c: usize) -> String {
        let mut out: Vec<u8> = vec![];
        let mut buf: [u8; 32] = [0; 32];

        loop {
            match s.read(&mut buf) {
                Ok(read) => out.extend_from_slice(&buf[..read]),
                Err(e) if e.would_block() => std::thread::yield_now(),
                Err(e) => panic!("{:?}", e),
            }
            if out.len() >= c {
                return String::from_utf8_lossy(&out).to_string();
            }
        }
    }

    fn suite<SubL: 'static + InStreamListenerStd, C: InStreamConfig>(mut listener: InStreamListenerTls<SubL>, c: C) {
        let (send_binding, recv_binding) = crossbeam_channel::unbounded();

        let server_thread = std::thread::spawn(move || {
            println!("bound to: {}", listener.binding());
            send_binding.send(listener.binding()).unwrap();

            let mut srv = loop {
                match listener.accept_std() {
                    Ok(srv) => break srv,
                    Err(e) if e.would_block() => std::thread::yield_now(),
                    Err(e) => panic!("{:?}", e),
                }
            }
            .into_std_stream();

            srv.write(b"hello from server").unwrap();
            srv.flush().unwrap();

            let res = read_count(&mut srv, 17);
            assert_eq!("hello from client", &res);
        });

        let client_thread = std::thread::spawn(move || {
            let binding = recv_binding.recv().unwrap();
            println!("connect to: {}", binding);

            let mut cli: StdStreamAdapter<InStreamTls<SubL::StreamStd>> =
                InStreamTls::connect(&binding, TlsConnectConfig::new(c))
                .unwrap()
                .into_std_stream();

            cli.write(b"hello from client").unwrap();
            cli.flush().unwrap();

            let res = read_count(&mut cli, 17);
            assert_eq!("hello from server", &res);
        });

        server_thread.join().unwrap();
        client_thread.join().unwrap();

        println!("done");
    }

    #[test]
    fn tls_works_mem() {
        let mut url = in_stream_mem::random_url("test");
        url.set_scheme(SCHEME).unwrap();
        let config = TlsBindConfig::new(MemBindConfig::default()).fake_certificate();
        let l: InStreamListenerTls<InStreamListenerMem> =
            InStreamListenerTls::bind(&url, config).unwrap();
        suite(l, MemConnectConfig::default());
    }

    #[test]
    fn tls_works_tcp() {
        let config = TlsBindConfig::new(TcpBindConfig::default()).fake_certificate();
        let l: InStreamListenerTls<InStreamListenerTcp> =
            InStreamListenerTls::bind(&url2!("{}://127.0.0.1:0", SCHEME), config).unwrap();
        suite(l, TcpConnectConfig::default());
    }
}
