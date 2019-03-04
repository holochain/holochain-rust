//! abstraction for working with Websocket connections
//! based on any rust io Read/Write Stream

mod tcp;

use std::io::{Read, Write};

use crate::ipc::transport::{
    DidWork, Transport, TransportError, TransportEvent, TransportId, TransportIdRef,
    TransportResult,
};

// -- some internal types for readability -- //

type TlsConnectResult<T> = Result<TlsStream<T>, native_tls::HandshakeError<T>>;
type WssHandshakeError<T> = tungstenite::handshake::HandshakeError<
    tungstenite::handshake::client::ClientHandshake<TlsStream<T>>,
>;
type WssConnectResult<T> =
    Result<(WssStream<T>, tungstenite::handshake::client::Response), WssHandshakeError<T>>;

type BaseStream<T> = T;
type TlsMidHandshake<T> = native_tls::MidHandshakeTlsStream<BaseStream<T>>;
type TlsStream<T> = native_tls::TlsStream<BaseStream<T>>;
type WssMidHandshake<T> =
    tungstenite::handshake::MidHandshake<tungstenite::ClientHandshake<TlsStream<T>>>;
type WssStream<T> = tungstenite::protocol::WebSocket<TlsStream<T>>;

type SocketMap<T> = std::collections::HashMap<String, TransportInfo<T>>;

// an internal state sequence for stream building
#[derive(Debug)]
enum WssStreamState<T: Read + Write + std::fmt::Debug> {
    None,
    Connecting(BaseStream<T>),
    TlsMidHandshake(TlsMidHandshake<T>),
    TlsReady(TlsStream<T>),
    WssMidHandshake(WssMidHandshake<T>),
    Ready(WssStream<T>),
}

/// how often should we send a heartbeat if we have not received msgs
pub const DEFAULT_HEARTBEAT_MS: usize = 2000;

/// when should we close a connection due to not receiving remote msgs
pub const DEFAULT_HEARTBEAT_WAIT_MS: usize = 5000;

/// Represents an individual connection
#[derive(Debug)]
struct TransportInfo<T: Read + Write + std::fmt::Debug> {
    id: TransportId,
    url: url::Url,
    last_msg: std::time::Instant,
    send_queue: Vec<Vec<u8>>,
    stateful_socket: WssStreamState<T>,
}

impl<T: Read + Write + std::fmt::Debug> TransportInfo<T> {
    pub fn close(&mut self) -> TransportResult<()> {
        println!("TransportInfo::close() for {:?}", self);
        if let WssStreamState::Ready(socket) = &mut self.stateful_socket {
            socket.close(None)?;
            socket.write_pending()?;
        }
        self.stateful_socket = WssStreamState::None;
        Ok(())
    }
}


/// a factory callback for generating base streams of type T
pub type StreamFactory<T> = fn(uri: &str) -> TransportResult<T>;

/// A "Transport" implementation based off the websocket protocol
/// any rust io Read/Write stream should be able to serve as the base
pub struct TransportWss<T: Read + Write + std::fmt::Debug> {
    stream_factory: StreamFactory<T>,
    stream_sockets: SocketMap<T>,
    event_queue: Vec<TransportEvent>,
    n_id: u64,
}

impl<T: Read + Write + std::fmt::Debug> Transport for TransportWss<T> {
    /// connect to a remote websocket service
    fn connect(&mut self, uri: &str) -> TransportResult<TransportId> {
        let uri = url::Url::parse(uri)?;
        let host_port = format!(
            "{}:{}",
            uri.host_str()
                .ok_or(TransportError("bad connect host".into()))?,
            uri.port()
                .ok_or(TransportError("bad connect port".into()))?,
        );
        let socket = (self.stream_factory)(&host_port)?;
        let id = self.priv_next_id();
        let info = TransportInfo {
            id: id.clone(),
            url: uri,
            last_msg: std::time::Instant::now(),
            send_queue: Vec::new(),
            stateful_socket: WssStreamState::Connecting(socket),
        };
        {
            //println!("Connect: socket = {:?}", socket);
            println!("Connect: info = {:?}", info);
        }
        self.stream_sockets.insert(id.clone(), info);
        Ok(id)
    }

    /// close a currently tracked connection
    fn close(&mut self, id: TransportId) -> TransportResult<()> {
        if let Some(mut info) = self.stream_sockets.remove(&id) {
            info.close()?;
        }
        Ok(())
    }

    /// close all currently tracked connections
    fn close_all(&mut self) -> TransportResult<()> {
        let mut errors: Vec<TransportError> = Vec::new();

        while !self.stream_sockets.is_empty() {
            let key = self
                .stream_sockets
                .keys()
                .next()
                .expect("should not be None")
                .to_string();
            if let Some(mut info) = self.stream_sockets.remove(&key) {
                if let Err(e) = info.close() {
                    errors.push(e.into());
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.into())
        }
    }

    /// get a list of all open transport ids
    fn transport_id_list(&self) -> TransportResult<Vec<TransportId>> {
        Ok(self.stream_sockets.keys().map(|k| k.to_string()).collect())
    }

    /// this should be called frequently on the event loop
    /// looks for incoming messages or processes ping/pong/close events etc
    fn poll(&mut self) -> TransportResult<(DidWork, Vec<TransportEvent>)> {
        let did_work = self.priv_process_stream_sockets()?;

        Ok((did_work, self.event_queue.drain(..).collect()))
    }

    /// send a message to one or more remote connected nodes
    fn send(&mut self, id_list: &[&TransportIdRef], payload: &[u8]) -> TransportResult<()> {
        for id in id_list {
            if let Some(info) = self.stream_sockets.get_mut(&id.to_string()) {
                info.send_queue.push(payload.to_vec());
            }
        }

        Ok(())
    }

    /// send a message to all remote nodes
    fn send_all(&mut self, payload: &[u8]) -> TransportResult<()> {
        for info in self.stream_sockets.values_mut() {
            info.send_queue.push(payload.to_vec());
        }
        Ok(())
    }
}

impl<T: Read + Write + std::fmt::Debug> TransportWss<T> {
    /// create a new websocket "Transport" instance of type T
    pub fn new(stream_factory: StreamFactory<T>) -> Self {
        TransportWss {
            stream_factory,
            stream_sockets: std::collections::HashMap::new(),
            event_queue: Vec::new(),
            n_id: 1,
        }
    }

    // -- private -- //

    // generate a unique id for
    fn priv_next_id(&mut self) -> String {
        let out = format!("ws{}", self.n_id);
        self.n_id += 1;
        return out;
    }

    // see if any work needs to be done on our stream sockets
    fn priv_process_stream_sockets(&mut self) -> TransportResult<bool> {
        let mut did_work = false;

        // take sockets out, so we can mut ref into self and it at same time
        let sockets: Vec<(String, TransportInfo<T>)> = self.stream_sockets.drain().collect();

        for (id, mut info) in sockets {
            if let Err(e) = self.priv_process_socket(&mut did_work, &mut info) {
                self.event_queue
                    .push(TransportEvent::TransportError(info.id.clone(), e));
            }
            if let WssStreamState::None = info.stateful_socket {
                self.event_queue.push(TransportEvent::Close(info.id));
                continue;
            }
            if info.last_msg.elapsed().as_millis() as usize > DEFAULT_HEARTBEAT_MS {
                if let WssStreamState::Ready(socket) = &mut info.stateful_socket {
                    socket.write_message(tungstenite::Message::Ping(vec![]))?;
                }
            } else if info.last_msg.elapsed().as_millis() as usize > DEFAULT_HEARTBEAT_WAIT_MS {
                self.event_queue.push(TransportEvent::Close(info.id));
                info.stateful_socket = WssStreamState::None;
                continue;
            }
            self.stream_sockets.insert(id, info);
        }

        Ok(did_work)
    }

    // process the state machine of an individual socket stream
    fn priv_process_socket(
        &mut self,
        did_work: &mut bool,
        info: &mut TransportInfo<T>,
    ) -> TransportResult<()> {
        // move the socket out, to be replaced
        let socket = std::mem::replace(&mut info.stateful_socket, WssStreamState::None);

        match socket {
            WssStreamState::None => {
                // stream must have closed, do nothing
                return Ok(());
            }
            WssStreamState::Connecting(socket) => {
                info.last_msg = std::time::Instant::now();
                *did_work = true;
                let connector = native_tls::TlsConnector::builder()
                    .danger_accept_invalid_certs(true)
                    .danger_accept_invalid_hostnames(true)
                    .build()
                    .expect("failed to build TlsConnector");
                info.stateful_socket =
                    self.priv_tls_handshake(connector.connect(info.url.as_str(), socket))?;
                return Ok(());
            }
            WssStreamState::TlsMidHandshake(socket) => {
                info.stateful_socket = self.priv_tls_handshake(socket.handshake())?;
                return Ok(());
            }
            WssStreamState::TlsReady(socket) => {
                info.last_msg = std::time::Instant::now();
                *did_work = true;
                info.stateful_socket = self
                    .priv_ws_handshake(&info.id, tungstenite::client(info.url.clone(), socket))?;
                return Ok(());
            }
            WssStreamState::WssMidHandshake(socket) => {
                info.stateful_socket = self.priv_ws_handshake(&info.id, socket.handshake())?;
                return Ok(());
            }
            WssStreamState::Ready(mut socket) => {
                let msgs: Vec<Vec<u8>> = info.send_queue.drain(..).collect();
                for msg in msgs {
                    if let Err(e) = socket.write_message(tungstenite::Message::Binary(msg)) {
                        return Err(e.into());
                    }
                }
                match socket.read_message() {
                    Err(tungstenite::error::Error::Io(e)) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            info.stateful_socket = WssStreamState::Ready(socket);
                            return Ok(());
                        }
                        return Err(e.into());
                    }
                    Err(tungstenite::error::Error::ConnectionClosed(_)) => {
                        // close event will be published
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                    Ok(msg) => {
                        info.last_msg = std::time::Instant::now();
                        *did_work = true;
                        let mut qmsg = None;
                        match msg {
                            tungstenite::Message::Text(s) => qmsg = Some(s.into_bytes()),
                            tungstenite::Message::Binary(b) => qmsg = Some(b),
                            _ => (),
                        }
                        if let Some(msg) = qmsg {
                            self.event_queue
                                .push(TransportEvent::Message(info.id.clone(), msg));
                        }
                        info.stateful_socket = WssStreamState::Ready(socket);
                        return Ok(());
                    }
                }
            }
        }
    }

    // process tls handshaking
    fn priv_tls_handshake(
        &mut self,
        res: TlsConnectResult<T>,
    ) -> TransportResult<WssStreamState<T>> {
        match res {
            Err(native_tls::HandshakeError::WouldBlock(socket)) => {
                Ok(WssStreamState::TlsMidHandshake(socket))
            }
            Err(e) => Err(e.into()),
            Ok(socket) => Ok(WssStreamState::TlsReady(socket)),
        }
    }

    // process websocket handshaking
    fn priv_ws_handshake(
        &mut self,
        id: &TransportId,
        res: WssConnectResult<T>,
    ) -> TransportResult<WssStreamState<T>> {
        match res {
            Err(tungstenite::HandshakeError::Interrupted(socket)) => {
                Ok(WssStreamState::WssMidHandshake(socket))
            }
            Err(e) => Err(e.into()),
            Ok((socket, _response)) => {
                self.event_queue.push(TransportEvent::Connect(id.clone()));
                Ok(WssStreamState::Ready(socket))
            }
        }
    }
}
