use crate::*;
use std::io::{Error, ErrorKind, Result};
use url2::prelude::*;

mod frame_type;
pub use frame_type::*;

const SCHEME: &'static str = "wss";

/// internal helper, make sure we're dealing with wss urls
fn validate_url_scheme(url: &Url2) -> Result<()> {
    if url.scheme() != SCHEME {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("got: '{}', expected: '{}://...'", SCHEME, url),
        ));
    }
    Ok(())
}

/// bind to a network interface to listen for websocket connections
#[derive(Debug)]
pub struct InStreamListenerWss<Sub: InStreamListener> {
    sub: Sub,
}

/// websocket specific bind configuration
pub struct WssBindConfig<SubConfig: Default> {
    pub sub_bind_config: SubConfig,
}

impl<SubConfig: Default> WssBindConfig<SubConfig> {
    pub fn sub_bind_config(mut self, sub_bind_config: SubConfig) -> Self {
        self.sub_bind_config = sub_bind_config;
        self
    }
}

impl<SubConfig: Default> Default for WssBindConfig<SubConfig> {
    fn default() -> Self {
        Self {
            sub_bind_config: Default::default(),
        }
    }
}

impl<Sub: InStreamListener> InStreamFramedListener for InStreamListenerWss<Sub> {
    type Partial = InStreamPartialWss<Sub::Partial>;
    type BindConfig = WssBindConfig<Sub::BindConfig>;

    fn bind(url: &Url2, config: Self::BindConfig) -> Result<Self> {
        validate_url_scheme(url)?;
        let mut url = url.clone();
        url.set_scheme(Sub::Partial::URL_SCHEME).unwrap();
        let sub = Sub::bind(&url, config.sub_bind_config)?;
        Ok(Self { sub })
    }

    fn binding(&self) -> Url2 {
        let mut url = self.sub.binding();
        url.set_scheme(SCHEME).unwrap();
        url
    }

    fn accept(&mut self) -> Result<<Self as InStreamFramedListener>::Partial> {
        let stream = self.sub.accept()?;

        Ok(InStreamPartialWss {
            state: Some(PartialWssState::PartialSub(stream)),
            is_server: true,
            connect_url: Url2::default(),
        })
    }
}

#[derive(Debug)]
enum PartialWssState<Sub: InStreamPartial> {
    PartialSub(Sub),
    MidCliHandshake(
        tungstenite::handshake::MidHandshake<tungstenite::ClientHandshake<Sub::Stream>>,
    ),
    MidSrvHandshake(
        tungstenite::handshake::MidHandshake<
            tungstenite::ServerHandshake<Sub::Stream, tungstenite::handshake::server::NoCallback>,
        >,
    ),
    Ready(tungstenite::WebSocket<Sub::Stream>),
}

/// a partly connected websocket stream - may still need to handshake
#[derive(Debug)]
pub struct InStreamPartialWss<Sub: InStreamPartial> {
    state: Option<PartialWssState<Sub>>,
    is_server: bool,
    connect_url: Url2,
}

type TungsteniteCliHandshakeResult<S> = std::result::Result<
    (
        tungstenite::WebSocket<S>,
        tungstenite::handshake::client::Response,
    ),
    tungstenite::handshake::HandshakeError<tungstenite::handshake::client::ClientHandshake<S>>,
>;

type TungsteniteSrvHandshakeResult<S> = std::result::Result<
    tungstenite::WebSocket<S>,
    tungstenite::handshake::HandshakeError<
        tungstenite::handshake::server::ServerHandshake<
            S,
            tungstenite::handshake::server::NoCallback,
        >,
    >,
>;

impl<Sub: InStreamPartial> InStreamPartialWss<Sub> {
    fn priv_proc_wss_cli_result(
        &mut self,
        result: TungsteniteCliHandshakeResult<Sub::Stream>,
    ) -> Result<<Self as InStreamFramedPartial>::Stream> {
        match result {
            Ok((stream, _response)) => Ok(InStreamWebSocket(stream)),
            Err(tungstenite::HandshakeError::Interrupted(mid)) => {
                self.state = Some(PartialWssState::MidCliHandshake(mid));
                Err(Error::with_would_block())
            }
            Err(e) => Err(Error::new(ErrorKind::ConnectionRefused, format!("{:?}", e))),
        }
    }

    fn priv_proc_wss_srv_result(
        &mut self,
        result: TungsteniteSrvHandshakeResult<Sub::Stream>,
    ) -> Result<<Self as InStreamFramedPartial>::Stream> {
        match result {
            Ok(stream) => Ok(InStreamWebSocket(stream)),
            Err(tungstenite::HandshakeError::Interrupted(mid)) => {
                self.state = Some(PartialWssState::MidSrvHandshake(mid));
                Err(Error::with_would_block())
            }
            Err(e) => Err(Error::new(ErrorKind::ConnectionRefused, format!("{:?}", e))),
        }
    }

    fn priv_sub(&mut self, mut sub: Sub) -> Result<<Self as InStreamFramedPartial>::Stream> {
        let stream = match sub.process() {
            Ok(stream) => stream,
            Err(e) => {
                self.state = Some(PartialWssState::PartialSub(sub));
                return Err(e);
            }
        };
        if self.is_server {
            self.priv_proc_wss_srv_result(tungstenite::accept(stream))
        } else {
            self.priv_proc_wss_cli_result(tungstenite::client(
                tungstenite::handshake::client::Request {
                    url: self.connect_url.clone().into(),
                    extra_headers: None,
                },
                stream,
            ))
        }
    }
}

/// websocket specific connect config
pub struct WssConnectConfig<SubConfig: Default> {
    pub sub_connect_config: SubConfig,
}

impl<SubConfig: Default> Default for WssConnectConfig<SubConfig> {
    fn default() -> Self {
        Self {
            sub_connect_config: Default::default(),
        }
    }
}

impl<Sub: InStreamPartial> InStreamFramedPartial for InStreamPartialWss<Sub> {
    type Stream = InStreamWebSocket<Sub>;
    type ConnectConfig = WssConnectConfig<Sub::ConnectConfig>;

    const URL_SCHEME: &'static str = SCHEME;

    fn with_stream(stream: Self::Stream) -> Result<Self> {
        Ok(Self {
            state: Some(PartialWssState::Ready(stream.0)),
            is_server: false, // we don't actually know, but that's cool
            connect_url: Url2::default(),
        })
    }

    fn connect(url: &Url2, config: Self::ConnectConfig) -> Result<Self> {
        validate_url_scheme(url)?;
        let connect_url = url.clone();
        let mut url = url.clone();
        url.set_scheme(Sub::URL_SCHEME).unwrap();
        let sub = Sub::connect(&url, config.sub_connect_config)?;
        Ok(Self {
            state: Some(PartialWssState::PartialSub(sub)),
            is_server: false,
            connect_url,
        })
    }

    fn process(&mut self) -> Result<Self::Stream> {
        match self.state.take() {
            None => Err(Error::new(ErrorKind::NotFound, "raw stream is None")),
            Some(state) => match state {
                PartialWssState::PartialSub(sub) => self.priv_sub(sub),
                PartialWssState::MidCliHandshake(mid) => {
                    self.priv_proc_wss_cli_result(mid.handshake())
                }
                PartialWssState::MidSrvHandshake(mid) => {
                    self.priv_proc_wss_srv_result(mid.handshake())
                }
                PartialWssState::Ready(stream) => Ok(InStreamWebSocket(stream)),
            },
        }
    }
}

/// a websocket connection to a remote node
#[derive(Debug)]
pub struct InStreamWebSocket<Sub: InStreamPartial>(pub tungstenite::WebSocket<Sub::Stream>);

impl<Sub: InStreamPartial> std::ops::Deref for InStreamWebSocket<Sub> {
    type Target = tungstenite::WebSocket<Sub::Stream>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Sub: InStreamPartial> std::ops::DerefMut for InStreamWebSocket<Sub> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Sub: InStreamPartial> std::convert::AsRef<tungstenite::WebSocket<Sub::Stream>>
    for InStreamWebSocket<Sub>
{
    fn as_ref(&self) -> &tungstenite::WebSocket<Sub::Stream> {
        &self.0
    }
}

impl<Sub: InStreamPartial> std::convert::AsMut<tungstenite::WebSocket<Sub::Stream>>
    for InStreamWebSocket<Sub>
{
    fn as_mut(&mut self) -> &mut tungstenite::WebSocket<Sub::Stream> {
        &mut self.0
    }
}

impl<Sub: InStreamPartial> InStreamFramed for InStreamWebSocket<Sub> {
    type FrameType = WsFrame;

    fn read_frame<T: From<Self::FrameType>>(&mut self) -> Result<T> {
        match self.0.read_message() {
            Ok(msg) => Ok(match msg {
                tungstenite::Message::Text(s) => WsFrame::from(s),
                tungstenite::Message::Binary(b) => WsFrame::new(b, WsFrameType::Binary),
                tungstenite::Message::Ping(b) => WsFrame::new(b, WsFrameType::Ping),
                tungstenite::Message::Pong(b) => WsFrame::new(b, WsFrameType::Pong),
                tungstenite::Message::Close(c) => match c {
                    Some(c) => WsFrame::new(
                        c.reason.to_string().into_bytes(),
                        WsFrameType::Close {
                            code: c.code.into(),
                        },
                    ),
                    None => WsFrame::new(vec![], WsFrameType::Close { code: 1000 }),
                },
            }
            .into()),
            Err(tungstenite::error::Error::Io(e)) => Err(e),
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("tungstenite error: {:?}", e),
            )),
        }
    }

    fn write_frame<T: Into<Self::FrameType>>(&mut self, data: T) -> Result<()> {
        let frame: WsFrame = data.into();
        let frame = match frame.frame_type().clone() {
            WsFrameType::Text => tungstenite::Message::Text(frame.into()),
            WsFrameType::Binary => tungstenite::Message::Binary(frame.into()),
            WsFrameType::Ping => tungstenite::Message::Ping(frame.into()),
            WsFrameType::Pong => tungstenite::Message::Pong(frame.into()),
            WsFrameType::Close { code } => tungstenite::Message::Close(Some(
                tungstenite::protocol::CloseFrame {
                    code: code.into(),
                    reason: frame.as_str(),
                }
                .into_owned(),
            )),
        };
        let res = self.0.write_message(frame);
        match &res {
            Ok(()) => Ok(()),
            Err(tungstenite::error::Error::Io(e)) if e.would_block() => {
                // ignore would-block errors on write
                // tungstenite queues them in pending, they'll get sent
                Ok(())
            }
            Err(tungstenite::error::Error::Io(_)) => {
                if let Err(tungstenite::error::Error::Io(e)) = res {
                    Err(e)
                } else {
                    unreachable!();
                }
            }
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("tungstenite error: {:?}", e),
            )),
        }
    }
}

/// typedef for `ListenerWss<Tls<Tcp>>`
pub type InStreamListenerWssType = InStreamListenerWss<InStreamListenerTls<InStreamListenerTcp>>;

/// typedef for `PartialWss<Tls<Tcp>>`
pub type InStreamPartialWssType = InStreamPartialWss<InStreamPartialTls<InStreamPartialTcp>>;

/// typedef for `WebSocket<Tls<Tcp>>`
pub type InStreamWebSocketType = InStreamWebSocket<InStreamPartialTls<InStreamPartialTcp>>;

#[cfg(test)]
mod tests {
    use super::*;

    fn suite<L: InStreamFramedListener>(mut l: L)
        where
            <<<L as InStreamFramedListener>::Partial as InStreamFramedPartial>::Stream as InStreamFramed>::FrameType: From<Vec<u8>>,
            Vec<u8>: From<<<<L as InStreamFramedListener>::Partial as InStreamFramedPartial>::Stream as InStreamFramed>::FrameType>

    {
        println!("bound to: {}", l.binding());

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

        let mut srv: <<L as InStreamFramedListener>::Partial as InStreamFramedPartial>::Stream =
            srv.unwrap();
        let mut cli = cli.unwrap();

        srv.write_frame(b"hello from server".to_vec()).unwrap();
        cli.write_frame(b"hello from client".to_vec()).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(100));

        let cli_read: Vec<u8> = cli.read_frame().unwrap();
        assert_eq!("hello from server", &String::from_utf8_lossy(&cli_read));
        let srv_read: Vec<u8> = srv.read_frame().unwrap();
        assert_eq!("hello from client", &String::from_utf8_lossy(&srv_read));

        println!("done");
    }

    #[test]
    fn wss_works_mem() {
        let mut url = in_stream_mem::random_url("test");
        url.set_scheme(SCHEME).unwrap();
        let l: InStreamListenerWss<InStreamListenerTls<InStreamListenerMem>> =
            InStreamListenerWss::bind(&url, Default::default()).unwrap();
        suite(l);
    }

    #[test]
    fn wss_works_tcp() {
        let l: InStreamListenerWss<InStreamListenerTls<InStreamListenerTcp>> =
            InStreamListenerWss::bind(
                &url2!("{}://127.0.0.1:0", SCHEME),
                WssBindConfig::default().sub_bind_config(TlsBindConfig::with_fake_certificate()),
            )
            .unwrap();
        suite(l);
    }
}
