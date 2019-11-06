use crate::websocket::{
    tls::TlsConfig, wss_info::WssInfo, BaseStream, SocketMap, TlsConnectResult, TlsMidHandshake,
    TlsSrvMidHandshake, TlsStream, WsConnectResult, WsMidHandshake, WsSrvAcceptResult,
    WsSrvMidHandshake, WsStream, WssConnectResult, WssMidHandshake, WssSrvAcceptResult,
    WssSrvMidHandshake, WssStream,
};
use log::*;

use lib3h::transport::error::{TransportError, TransportResult};

use lib3h_zombie_actor::GhostMutex;
use lib3h_protocol::{uri::Lib3hUri, DidWork};
use std::{
    io::{Read, Write},
    sync::Arc,
};

use url::Url;

/// how often should we send a heartbeat if we have not received msgs
pub const DEFAULT_HEARTBEAT_MS: usize = 2000;

/// when should we close a connection due to not receiving remote msgs
pub const DEFAULT_HEARTBEAT_WAIT_MS: usize = 5000;

// an internal state sequence for stream building
#[derive(Debug)]
pub enum WebsocketStreamState<T: Read + Write + std::fmt::Debug> {
    None,
    Connecting(BaseStream<T>),
    #[allow(dead_code)]
    ConnectingSrv(BaseStream<T>),
    TlsMidHandshake(TlsMidHandshake<T>),
    TlsSrvMidHandshake(TlsSrvMidHandshake<T>),
    TlsReady(TlsStream<T>),
    TlsSrvReady(TlsStream<T>),
    WsMidHandshake(WsMidHandshake<T>),
    WsSrvMidHandshake(WsSrvMidHandshake<T>),
    WssMidHandshake(WssMidHandshake<T>),
    WssSrvMidHandshake(WssSrvMidHandshake<T>),
    ReadyWs(Box<WsStream<T>>),
    ReadyWss(Box<WssStream<T>>),
}

#[derive(PartialEq)]
pub enum ConnectionStatus {
    None,
    Initializing,
    Ready,
}

/// Events that can be generated during a `process()`
#[derive(Debug, PartialEq, Clone)]
pub enum StreamEvent {
    /// Notify that some TransportError occured
    ErrorOccured(Url, TransportError),
    /// an outgoing connection has been established
    ConnectResult(Url, String),
    /// we have received an incoming connection
    IncomingConnectionEstablished(Url),
    /// We have received data from a connection
    ReceivedData(Url, Vec<u8>),
    /// A connection closed for whatever reason
    ConnectionClosed(Url),
}

/// A factory callback for generating base streams of type T
pub type StreamFactory<T> = fn(uri: &str) -> TransportResult<T>;

lazy_static! {
    static ref TRANSPORT_COUNT: Arc<GhostMutex<u64>> = Arc::new(GhostMutex::new(0));
}

/// A function that produces accepted sockets of type R wrapped in a TransportInfo
pub type Acceptor<T> = Box<dyn FnMut() -> TransportResult<WssInfo<T>>>;

/// A function that binds to a url and produces sockt acceptors of type T
pub type Bind<T> = Box<dyn FnMut(&Url) -> TransportResult<Acceptor<T>>>;

/// A "Transport" implementation based off the websocket protocol
/// any rust io Read/Write stream should be able to serve as the base
pub struct StreamManager<T: Read + Write + std::fmt::Debug> {
    tls_config: TlsConfig,
    stream_factory: StreamFactory<T>,
    stream_sockets: SocketMap<T>,
    event_queue: Vec<StreamEvent>,
    bind: Bind<T>,
    acceptor: TransportResult<Acceptor<T>>,
}

impl<T: Read + Write + std::fmt::Debug> StreamManager<T> {
    pub fn new(stream_factory: StreamFactory<T>, bind: Bind<T>, tls_config: TlsConfig) -> Self {
        StreamManager {
            tls_config,
            stream_factory,
            stream_sockets: std::collections::HashMap::new(),
            event_queue: Vec::new(),
            bind,
            acceptor: Err(TransportError::new("acceptor not initialized".into())),
        }
    }

    /// connect to a remote websocket service
    pub fn connect(&mut self, uri: &Url) -> TransportResult<()> {
        let host_port = format!(
            "{}:{}",
            uri.host_str()
                .ok_or_else(|| TransportError::new("bad connect host".into()))?,
            uri.port()
                .ok_or_else(|| TransportError::new("bad connect port".into()))?,
        );
        let socket = (self.stream_factory)(&host_port)?;
        let info = WssInfo::client(uri.clone(), socket);
        self.stream_sockets.insert(uri.clone().into(), info);
        Ok(())
    }

    /// close a currently tracked connection
    #[allow(dead_code)]
    pub fn close(&mut self, uri: &Url) -> TransportResult<()> {
        if let Some(mut info) = self.stream_sockets.remove(uri) {
            info.close()?;
        }
        Ok(())
    }

    /// close all currently tracked connections
    #[allow(dead_code)]
    pub fn close_all(&mut self) -> TransportResult<()> {
        let mut errors: Vec<TransportError> = Vec::new();

        while !self.stream_sockets.is_empty() {
            let key = self
                .stream_sockets
                .keys()
                .next()
                .expect("should not be None")
                .clone();
            if let Some(mut info) = self.stream_sockets.remove(&key) {
                if let Err(e) = info.close() {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.into())
        }
    }

    /// this should be called frequently on the event loop
    /// looks for incoming messages or processes ping/pong/close events etc
    pub fn process(&mut self) -> TransportResult<(DidWork, Vec<StreamEvent>)> {
        let mut did_work = false;

        if self.priv_process_stream_sockets()? {
            did_work = true
        }

        Ok((did_work, self.event_queue.drain(..).collect()))
    }

    /// send a message to one or more remote connected nodes
    pub fn send(&mut self, url: &Url, payload: &[u8]) -> TransportResult<()> {
        //println!("send() 1 {:?}", url);
        let mut info = self
            .stream_sockets
            .get_mut(url)
            .ok_or_else(|| format!("No socket found for URL: {}", url.to_string()))?;

        //println!("send() 2 {:?}", url);
        let mut ws_stream =
            std::mem::replace(&mut info.stateful_socket, WebsocketStreamState::None);
        let send_result = match &mut ws_stream {
            WebsocketStreamState::ReadyWs(socket) => socket
                .write_message(tungstenite::Message::Binary(payload.to_vec()))
                .map_err(|error| format!("{}", error)),
            WebsocketStreamState::ReadyWss(socket) => socket
                .write_message(tungstenite::Message::Binary(payload.to_vec()))
                .map_err(|error| format!("{}", error)),
            _ => Err(String::from("Websocket not in Ready state")),
        };
        info.stateful_socket = ws_stream;
        //println!("send() 3 {:?}", send_result);
        send_result.map_err(|error_string| {
            //println!("Error in send(): {}", error_string);
            TransportError::from(error_string)
        })
    }

    pub fn bind(&mut self, url: &Url) -> TransportResult<Url> {
        let acceptor = (self.bind)(&url.clone());
        acceptor.map(|acceptor| {
            self.acceptor = Ok(acceptor);
            url.clone()
        })
    }

    pub fn connection_status(&self, url: &Url) -> ConnectionStatus {
        self.stream_sockets
            .get(url)
            .map(|info| match info.stateful_socket {
                WebsocketStreamState::ReadyWs(_) | WebsocketStreamState::ReadyWss(_) => {
                    ConnectionStatus::Ready
                }
                _ => ConnectionStatus::Initializing,
            })
            .unwrap_or(ConnectionStatus::None)
    }

    // -- private -- //

    fn priv_process_accept(&mut self) -> DidWork {
        match &mut self.acceptor {
            Err(err) => {
                warn!("acceptor in error state: {:?}", err);
                false
            }
            Ok(acceptor) => (acceptor)()
                .map(move |wss_info| {
                    let _insert_result = self
                        .stream_sockets
                        .insert(wss_info.url.clone().into(), wss_info);
                    true
                })
                .unwrap_or_else(|err| {
                    if !err.is_ignorable() {
                        // TODO: handle these actual errors, and probably this is where the unbinding
                        // would be detectable.
                        panic!("Error when attempting to accept connections: {:?}", err);
                    }
                    false
                }),
        }
    }

    // see if any work needs to be done on our stream sockets
    fn priv_process_stream_sockets(&mut self) -> TransportResult<DidWork> {
        let mut did_work = false;

        // accept some incoming connections
        did_work |= self.priv_process_accept();

        // take sockets out, so we can mut ref into self and it at same time
        let sockets: Vec<(Lib3hUri, WssInfo<T>)> = self.stream_sockets.drain().collect();

        for (id, mut info) in sockets {
            if let Err(e) = self.priv_process_socket(&mut did_work, &mut info) {
                self.event_queue
                    .push(StreamEvent::ErrorOccured(info.url.clone(), e));
            }
            if let WebsocketStreamState::None = info.stateful_socket {
                self.event_queue
                    .push(StreamEvent::ConnectionClosed(info.url));
                continue;
            }
            if info.last_msg.elapsed().as_millis() as usize > DEFAULT_HEARTBEAT_MS {
                if let WebsocketStreamState::ReadyWss(socket) = &mut info.stateful_socket {
                    if let Err(e) = socket.write_message(tungstenite::Message::Ping(vec![])) {
                        error!("Transport error trying to send ping over stream: {:?}. Dropping stream...", e);
                        continue;
                    }
                }
                if let WebsocketStreamState::ReadyWs(socket) = &mut info.stateful_socket {
                    if let Err(e) = socket.write_message(tungstenite::Message::Ping(vec![])) {
                        error!("Transport error trying to send ping over stream: {:?}. Dropping stream...", e);
                        continue;
                    }
                }
            } else if info.last_msg.elapsed().as_millis() as usize > DEFAULT_HEARTBEAT_WAIT_MS {
                self.event_queue
                    .push(StreamEvent::ConnectionClosed(info.url));
                info.stateful_socket = WebsocketStreamState::None;
                continue;
            }
            self.stream_sockets.insert(id, info);
            //match info.stateful_socket {
            //    WebsocketStreamState::None => {None},
            //    _ => self.stream_sockets.insert(id, info),
            //};
        }

        Ok(did_work)
    }

    // process the state machine of an individual socket stream
    fn priv_process_socket(
        &mut self,
        did_work: &mut bool,
        info: &mut WssInfo<T>,
    ) -> TransportResult<()> {
        // move the socket out, to be replaced
        let socket = std::mem::replace(&mut info.stateful_socket, WebsocketStreamState::None);

        trace!("transport_wss: socket={:?}", socket);
        // TODO remove?
        std::io::stdout().flush().ok().expect("flush stdout");
        match socket {
            WebsocketStreamState::None => {
                // stream must have closed, do nothing
                Ok(())
            }
            WebsocketStreamState::Connecting(socket) => {
                info.last_msg = std::time::Instant::now();
                *did_work = true;
                match &self.tls_config {
                    TlsConfig::Unencrypted => {
                        info.stateful_socket = self.priv_ws_handshake(
                            &info.url,
                            &info.request_id,
                            tungstenite::client(info.url.clone(), socket),
                        )?;
                    }
                    _ => {
                        let connector = native_tls::TlsConnector::builder()
                            .danger_accept_invalid_certs(true)
                            .danger_accept_invalid_hostnames(true)
                            .build()
                            .expect("failed to build TlsConnector");
                        info.stateful_socket =
                            self.priv_tls_handshake(connector.connect(info.url.as_str(), socket))?;
                    }
                }
                Ok(())
            }
            WebsocketStreamState::ConnectingSrv(socket) => {
                info.last_msg = std::time::Instant::now();
                *did_work = true;
                if let &TlsConfig::Unencrypted = &self.tls_config {
                    info.stateful_socket =
                        self.priv_ws_srv_handshake(&info.url, tungstenite::accept(socket))?;
                    return Ok(());
                }
                let ident = self.tls_config.get_identity()?;
                let acceptor = native_tls::TlsAcceptor::builder(ident)
                    .build()
                    .expect("failed to build TlsAcceptor");
                info.stateful_socket = self.priv_tls_srv_handshake(acceptor.accept(socket))?;
                Ok(())
            }
            WebsocketStreamState::TlsMidHandshake(socket) => {
                info.stateful_socket = self.priv_tls_handshake(socket.handshake())?;
                Ok(())
            }
            WebsocketStreamState::TlsSrvMidHandshake(socket) => {
                info.stateful_socket = self.priv_tls_srv_handshake(socket.handshake())?;
                Ok(())
            }
            WebsocketStreamState::TlsReady(socket) => {
                info.last_msg = std::time::Instant::now();
                *did_work = true;
                info.stateful_socket = self.priv_wss_handshake(
                    &info.url,
                    &info.request_id,
                    tungstenite::client(info.url.clone(), socket),
                )?;
                Ok(())
            }
            WebsocketStreamState::TlsSrvReady(socket) => {
                info.last_msg = std::time::Instant::now();
                *did_work = true;
                info.stateful_socket =
                    self.priv_wss_srv_handshake(&info.url, tungstenite::accept(socket))?;
                Ok(())
            }
            WebsocketStreamState::WsMidHandshake(socket) => {
                info.stateful_socket =
                    self.priv_ws_handshake(&info.url, &info.request_id, socket.handshake())?;
                Ok(())
            }
            WebsocketStreamState::WsSrvMidHandshake(socket) => {
                info.stateful_socket = self.priv_ws_srv_handshake(&info.url, socket.handshake())?;
                Ok(())
            }
            WebsocketStreamState::WssMidHandshake(socket) => {
                info.stateful_socket =
                    self.priv_wss_handshake(&info.url, &info.request_id, socket.handshake())?;
                Ok(())
            }
            WebsocketStreamState::WssSrvMidHandshake(socket) => {
                info.stateful_socket =
                    self.priv_wss_srv_handshake(&info.url, socket.handshake())?;
                Ok(())
            }
            WebsocketStreamState::ReadyWs(mut socket) => {
                match socket.read_message() {
                    Err(tungstenite::error::Error::Io(e)) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            info.stateful_socket = WebsocketStreamState::ReadyWs(socket);
                            return Ok(());
                        }
                        Err(e.into())
                    }
                    Err(tungstenite::error::Error::ConnectionClosed) => {
                        error!("Connection unexpectedly closed");
                        // close event will be published
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                    Ok(msg) => {
                        info.last_msg = std::time::Instant::now();
                        *did_work = true;
                        let qmsg = match msg {
                            tungstenite::Message::Text(s) => Some(s.into_bytes()),
                            tungstenite::Message::Binary(b) => Some(b),
                            _ => None,
                        };

                        if let Some(msg) = qmsg {
                            self.event_queue
                                .push(StreamEvent::ReceivedData(info.url.clone(), msg));
                        }
                        info.stateful_socket = WebsocketStreamState::ReadyWs(socket);
                        Ok(())
                    }
                }
            }
            WebsocketStreamState::ReadyWss(mut socket) => {
                match socket.read_message() {
                    Err(tungstenite::error::Error::Io(e)) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            info.stateful_socket = WebsocketStreamState::ReadyWss(socket);
                            return Ok(());
                        }
                        Err(e.into())
                    }
                    Err(tungstenite::error::Error::ConnectionClosed) => {
                        // close event will be published
                        error!("Connection unexpectedly closed");
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                    Ok(msg) => {
                        info.last_msg = std::time::Instant::now();
                        *did_work = true;
                        let qmsg = match msg {
                            tungstenite::Message::Text(s) => Some(s.into_bytes()),
                            tungstenite::Message::Binary(b) => Some(b),
                            _ => None,
                        };

                        if let Some(msg) = qmsg {
                            self.event_queue
                                .push(StreamEvent::ReceivedData(info.url.clone(), msg));
                        }
                        info.stateful_socket = WebsocketStreamState::ReadyWss(socket);
                        Ok(())
                    }
                }
            }
        }
    }

    // process tls handshaking
    fn priv_tls_handshake(
        &mut self,
        res: TlsConnectResult<T>,
    ) -> TransportResult<WebsocketStreamState<T>> {
        match res {
            Err(native_tls::HandshakeError::WouldBlock(socket)) => {
                Ok(WebsocketStreamState::TlsMidHandshake(socket))
            }
            Err(e) => Err(e.into()),
            Ok(socket) => Ok(WebsocketStreamState::TlsReady(socket)),
        }
    }

    // process tls handshaking
    fn priv_tls_srv_handshake(
        &mut self,
        res: TlsConnectResult<T>,
    ) -> TransportResult<WebsocketStreamState<T>> {
        trace!("[t] processing tls connect result: {:?}", res);
        match res {
            Err(native_tls::HandshakeError::WouldBlock(socket)) => {
                Ok(WebsocketStreamState::TlsSrvMidHandshake(socket))
            }
            Err(e) => Err(e.into()),
            Ok(socket) => Ok(WebsocketStreamState::TlsSrvReady(socket)),
        }
    }

    // process websocket handshaking
    fn priv_ws_handshake(
        &mut self,
        url: &Url,
        request_id: &str,
        res: WsConnectResult<T>,
    ) -> TransportResult<WebsocketStreamState<T>> {
        match res {
            Err(tungstenite::HandshakeError::Interrupted(socket)) => {
                Ok(WebsocketStreamState::WsMidHandshake(socket))
            }
            Err(e) => Err(e.into()),
            Ok((socket, _response)) => {
                self.event_queue.push(StreamEvent::ConnectResult(
                    url.clone(),
                    request_id.to_string(),
                ));
                Ok(WebsocketStreamState::ReadyWs(Box::new(socket)))
            }
        }
    }

    // process websocket handshaking
    fn priv_wss_handshake(
        &mut self,
        url: &Url,
        request_id: &str,
        res: WssConnectResult<T>,
    ) -> TransportResult<WebsocketStreamState<T>> {
        match res {
            Err(tungstenite::HandshakeError::Interrupted(socket)) => {
                Ok(WebsocketStreamState::WssMidHandshake(socket))
            }
            Err(e) => Err(e.into()),
            Ok((socket, _response)) => {
                self.event_queue.push(StreamEvent::ConnectResult(
                    url.clone(),
                    request_id.to_string(),
                ));
                Ok(WebsocketStreamState::ReadyWss(Box::new(socket)))
            }
        }
    }

    // process websocket srv handshaking
    fn priv_ws_srv_handshake(
        &mut self,
        url: &Url,
        res: WsSrvAcceptResult<T>,
    ) -> TransportResult<WebsocketStreamState<T>> {
        match res {
            Err(tungstenite::HandshakeError::Interrupted(socket)) => {
                Ok(WebsocketStreamState::WsSrvMidHandshake(socket))
            }
            Err(e) => Err(e.into()),
            Ok(socket) => {
                self.event_queue
                    .push(StreamEvent::IncomingConnectionEstablished(url.clone()));
                Ok(WebsocketStreamState::ReadyWs(Box::new(socket)))
            }
        }
    }

    // process websocket srv handshaking
    fn priv_wss_srv_handshake(
        &mut self,
        url: &Url,
        res: WssSrvAcceptResult<T>,
    ) -> TransportResult<WebsocketStreamState<T>> {
        match res {
            Err(tungstenite::HandshakeError::Interrupted(socket)) => {
                Ok(WebsocketStreamState::WssSrvMidHandshake(socket))
            }
            Err(e) => Err(e.into()),
            Ok(socket) => {
                self.event_queue
                    .push(StreamEvent::IncomingConnectionEstablished(url.clone()));
                Ok(WebsocketStreamState::ReadyWss(Box::new(socket)))
            }
        }
    }
}
