use crate::*;
use std::io::{Error, ErrorKind, Read, Result, Write};
use url2::prelude::*;

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
pub struct InStreamListenerTls<Sub: InStreamListenerStd>
{
    sub: Sub,
    acceptor: native_tls::TlsAcceptor,
}

impl<Sub: InStreamListenerStd>
std::fmt::Debug for InStreamListenerTls<Sub>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InStreamListenerTls")
            .field("sub", &self.sub)
            .finish()
    }
}

impl<Sub: InStreamListenerStd>
InStreamListener<&mut [u8], &[u8]> for InStreamListenerTls<Sub>
{
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
        match &mut self.state {
            None => Err(ErrorKind::NotConnected.into()),
            Some(state) => {
                if let TlsState::Ready(tls) = state {
                    let written = tls.write(&self.write_buf)?;
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
        match &mut self.state {
            None => Err(ErrorKind::NotConnected.into()),
            Some(state) => {
                match state {
                    TlsState::MidHandshake(_) => Err(Error::with_would_block()),
                    TlsState::Ready(tls) => {
                        tls.read(data)
                    }
                }
            }
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.priv_process()?;
        match &mut self.state {
            None => Err(ErrorKind::NotConnected.into()),
            Some(state) => {
                match state {
                    TlsState::MidHandshake(_) => {
                        self.write_buf.extend_from_slice(data);
                        Ok(data.len())
                    }
                    TlsState::Ready(tls) => {
                        if self.write_buf.is_empty() {
                            let written = tls.write(data)?;
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
                }
            }
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
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

impl<Sub: InStreamStd> InStreamStd for InStreamTls<Sub> {}

/*
/// bind to a network interface to listen for TLS connections
pub struct InStreamListenerTls<Sub: InStreamListener> {
    sub: Sub,
    acceptor: native_tls::TlsAcceptor,
}

impl<Sub: InStreamListener> std::fmt::Debug for InStreamListenerTls<Sub> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InStreamListenerTls")
            .field("sub", &self.sub)
            .finish()
    }
}

/// tls specific bind config
pub struct TlsBindConfig<SubConfig: Default> {
    pub tls_certificate: TlsCertificate,
    pub sub_bind_config: SubConfig,
}

impl<SubConfig: Default> TlsBindConfig<SubConfig> {
    pub fn with_fake_certificate() -> Self {
        Self {
            tls_certificate: TlsCertificate::with_fake_certificate(),
            sub_bind_config: Default::default(),
        }
    }

    pub fn sub_bind_config(mut self, sub_bind_config: SubConfig) -> Self {
        self.sub_bind_config = sub_bind_config;
        self
    }
}

impl<SubConfig: Default> Default for TlsBindConfig<SubConfig> {
    fn default() -> Self {
        Self {
            tls_certificate: TlsCertificate::generate_dev(),
            sub_bind_config: Default::default(),
        }
    }
}

impl<Sub: InStreamListener> InStreamListener for InStreamListenerTls<Sub> {
    type Partial = InStreamPartialTls<Sub::Partial>;
    type BindConfig = TlsBindConfig<Sub::BindConfig>;

    /// bind to the network interface && start listening for tls connections
    fn bind(url: &Url2, config: Self::BindConfig) -> Result<Self> {
        validate_url_scheme(url)?;
        let id = native_tls::Identity::from_pkcs12(
            &config.tls_certificate.pkcs12_data,
            &config.tls_certificate.passphrase,
        )
        .unwrap();
        let acceptor = native_tls::TlsAcceptor::new(id).unwrap();
        let mut url = url.clone();
        url.set_scheme(Sub::Partial::URL_SCHEME).unwrap();
        let sub = Sub::bind(&url, config.sub_bind_config)?;
        Ok(Self { sub, acceptor })
    }

    /// get our bound address
    fn binding(&self) -> Url2 {
        let mut url = self.sub.binding();
        url.set_scheme(SCHEME).unwrap();
        url
    }

    /// accept an incoming connection
    fn accept(&mut self) -> Result<<Self as InStreamListener>::Partial> {
        // get e.g. an InStreamTcpPartial
        let stream = self.sub.accept()?;

        // wrap it with our own partial
        Ok(InStreamPartialTls {
            state: Some(PartialTlsState::PartialSub(stream)),
            server_acceptor: Some(self.acceptor.clone()),
        })
    }
}
*/

/*
#[derive(Debug)]
enum PartialTlsState<Sub: InStreamPartial> {
    PartialSub(Sub),
    MidHandshake(native_tls::MidHandshakeTlsStream<Sub::Stream>),
    Ready(native_tls::TlsStream<Sub::Stream>),
}

/// a partial tls connection stream - may still need to tls handshake
pub struct InStreamPartialTls<Sub: InStreamPartial> {
    state: Option<PartialTlsState<Sub>>,
    // `None` if this is a client, or `Ready` stream
    server_acceptor: Option<native_tls::TlsAcceptor>,
}

impl<Sub: InStreamPartial> InStreamPartialTls<Sub> {
    fn priv_proc_tls_result(
        &mut self,
        result: std::result::Result<
            native_tls::TlsStream<Sub::Stream>,
            native_tls::HandshakeError<Sub::Stream>,
        >,
    ) -> Result<<Self as InStreamPartial>::Stream> {
        match result {
            Ok(tls) => Ok(InStreamTls(tls)),
            Err(e) => match e {
                native_tls::HandshakeError::WouldBlock(mid) => {
                    self.state = Some(PartialTlsState::MidHandshake(mid));
                    Err(Error::with_would_block())
                }
                native_tls::HandshakeError::Failure(e) => {
                    Err(Error::new(ErrorKind::ConnectionRefused, format!("{:?}", e)))
                }
            },
        }
    }

    fn priv_sub(&mut self, mut sub: Sub) -> Result<<Self as InStreamPartial>::Stream> {
        // first, process our substream... if it's not ready: WouldBlock
        let stream = match sub.process() {
            Ok(stream) => stream,
            Err(e) => {
                self.state = Some(PartialTlsState::PartialSub(sub));
                return Err(e);
            }
        };
        // now, wrap the sub-stream in a TlsStream
        match self.server_acceptor.take() {
            Some(acceptor) => {
                let res = acceptor.accept(stream);
                self.priv_proc_tls_result(res)
            }
            None => self.priv_proc_tls_result(
                native_tls::TlsConnector::builder()
                    .danger_accept_invalid_certs(true)
                    .danger_accept_invalid_hostnames(true)
                    .build()
                    .unwrap()
                    .connect("none:", stream),
            ),
        }
    }

    fn priv_mid(
        &mut self,
        mid: native_tls::MidHandshakeTlsStream<Sub::Stream>,
    ) -> Result<<Self as InStreamPartial>::Stream> {
        self.priv_proc_tls_result(mid.handshake())
    }
}

impl<Sub: InStreamPartial> std::fmt::Debug for InStreamPartialTls<Sub> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InStreamPartialTls")
            .field("state", &self.state)
            .field(
                "server_acceptor",
                if self.server_acceptor.is_some() {
                    &"some"
                } else {
                    &"none"
                },
            )
            .finish()
    }
}


impl<Sub: InStreamPartial> InStreamPartial for InStreamPartialTls<Sub> {
    type Stream = InStreamTls<Sub>;
    type ConnectConfig = TlsConnectConfig<Sub::ConnectConfig>;

    const URL_SCHEME: &'static str = SCHEME;

    fn with_stream(stream: Self::Stream) -> Result<Self> {
        Ok(Self {
            state: Some(PartialTlsState::Ready(stream.0)),
            server_acceptor: None,
        })
    }

    fn connect(url: &Url2, config: Self::ConnectConfig) -> Result<Self> {
        validate_url_scheme(url)?;
        let mut url = url.clone();
        url.set_scheme(Sub::URL_SCHEME).unwrap();
        let sub = Sub::connect(&url, config.sub_connect_config)?;
        Ok(Self {
            state: Some(PartialTlsState::PartialSub(sub)),
            server_acceptor: None,
        })
    }

    fn process(&mut self) -> Result<Self::Stream> {
        match self.state.take() {
            None => Err(Error::new(ErrorKind::NotFound, "raw stream is None")),
            Some(state) => match state {
                PartialTlsState::PartialSub(sub) => self.priv_sub(sub),
                PartialTlsState::MidHandshake(mid) => self.priv_mid(mid),
                PartialTlsState::Ready(tls) => Ok(InStreamTls(tls)),
            },
        }
    }
}

/// a TLS connection to a remote node
// alas shrinkwrap fails due to conflicting borrow blanket
#[derive(Debug)]
pub struct InStreamTls<Sub: InStreamPartial>(pub native_tls::TlsStream<Sub::Stream>);

impl<Sub: InStreamPartial> std::ops::Deref for InStreamTls<Sub> {
    type Target = native_tls::TlsStream<Sub::Stream>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Sub: InStreamPartial> std::ops::DerefMut for InStreamTls<Sub> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Sub: InStreamPartial> std::convert::AsRef<native_tls::TlsStream<Sub::Stream>>
    for InStreamTls<Sub>
{
    fn as_ref(&self) -> &native_tls::TlsStream<Sub::Stream> {
        &self.0
    }
}

impl<Sub: InStreamPartial> std::convert::AsMut<native_tls::TlsStream<Sub::Stream>>
    for InStreamTls<Sub>
{
    fn as_mut(&mut self) -> &mut native_tls::TlsStream<Sub::Stream> {
        &mut self.0
    }
}

impl<Sub: InStreamPartial> InStream for InStreamTls<Sub> {}

impl<Sub: InStreamPartial> Read for InStreamTls<Sub> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl<Sub: InStreamPartial> Write for InStreamTls<Sub> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.flush()
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    fn suite<L: 'static + InStreamListenerStd, C: InStreamConfig>(mut listener: L, c: C) {
        let (send_binding, recv_binding) = crossbeam_channel::unbounded();

        let server_thread = std::thread::spawn(move || {
            println!("bound to: {}", listener.binding());
            send_binding.send(listener.binding()).unwrap();

            let mut srv = loop {
                match listener.accept_std() {
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
            //srv.shutdown(std::net::Shutdown::Write).unwrap();

            std::thread::sleep(std::time::Duration::from_millis(100));
        });

        std::thread::sleep(std::time::Duration::from_millis(100));

        let client_thread = std::thread::spawn(move || {
            let binding = recv_binding.recv().unwrap();
            println!("connect to: {}", binding);

            let mut cli = L::StreamStd::raw_connect(
                &binding,
                TlsConnectConfig::new(c),
            )
                .unwrap()
                .into_std_stream();

            cli.write(b"hello from client").unwrap();
            cli.flush().unwrap();
            //cli.shutdown(std::net::Shutdown::Write).unwrap();

            std::thread::sleep(std::time::Duration::from_millis(100));
        });

        server_thread.join().unwrap();
        client_thread.join().unwrap();

        println!("done");
        /*
        let mut c: L::Partial = L::Partial::connect(&l.binding(), Default::default()).unwrap();

        let mut s = l.accept_blocking().unwrap();

        let mut srv = None;
        let mut cli = None;
        loop {
            if let None = cli {
                if let Ok(c) = c.process() {
                    cli = Some(c);
                }
            }

            if let None = srv {
                if let Ok(s) = s.process() {
                    srv = Some(s);
                }
            }

            if srv.is_some() && cli.is_some() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        let mut srv = srv.unwrap();
        let mut cli = cli.unwrap();

        let mut buf = [0; 32];

        srv.write_all(b"hello from server").unwrap();
        cli.write_all(b"hello from client").unwrap();

        std::thread::sleep(std::time::Duration::from_millis(100));

        assert_eq!(17, srv.read(&mut buf).unwrap());
        assert_eq!("hello from client", &String::from_utf8_lossy(&buf[..17]));
        assert_eq!(17, cli.read(&mut buf).unwrap());
        assert_eq!("hello from server", &String::from_utf8_lossy(&buf[..17]));

        println!("done");
        */
    }

    #[test]
    fn tls_works_mem() {
        let mut url = in_stream_mem::random_url("test");
        url.set_scheme(SCHEME).unwrap();
        let config = TlsBindConfig::new(()).fake_certificate();
        let l: InStreamListenerTls<InStreamListenerMem> =
            InStreamListenerTls::raw_bind(&url, config).unwrap();
        suite(l, ());
    }

    #[test]
    fn tls_works_tcp() {
        let config = TlsBindConfig::new(TcpBindConfig::default()).fake_certificate();
        let l: InStreamListenerTls<InStreamListenerTcp> = InStreamListenerTls::raw_bind(
            &url2!("{}://127.0.0.1:0", SCHEME),
            config,
        )
        .unwrap();
        suite(l, TcpConnectConfig::default());
    }
}
