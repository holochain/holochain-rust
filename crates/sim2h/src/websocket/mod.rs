/// Transport implementation that enables communication via websockets.
/// The interface and Ghost actor implementation in actor::GhostTransportWebsocket wraps
/// all internals, especially streams::StreamManager (former TransportWss) which implements
/// a connection pool.
///
/// The connection pool implemented abstractly based on any rust io Read/Write Stream.
/// Module tcp implements a concrete type based on std::net::TcpStream.
extern crate env_logger;
extern crate log;

pub mod streams;
mod tcp;
pub mod tls;
mod wss_info;

use lib3h::transport::error::TransportResult;
use lib3h_protocol::uri::Lib3hUri;
use wss_info::WssInfo;

static FAKE_PKCS12: &'static [u8] = include_bytes!("fake_key.p12");
static FAKE_PASS: &'static str = "hello";

// -- some internal types for readability -- //

type TlsConnectResult<T> = Result<TlsStream<T>, native_tls::HandshakeError<T>>;
type WsHandshakeError<T> =
    tungstenite::handshake::HandshakeError<tungstenite::handshake::client::ClientHandshake<T>>;
type WsConnectResult<T> =
    Result<(WsStream<T>, tungstenite::handshake::client::Response), WsHandshakeError<T>>;
type WsSrvHandshakeError<T> = tungstenite::handshake::HandshakeError<
    tungstenite::handshake::server::ServerHandshake<T, tungstenite::handshake::server::NoCallback>,
>;
type WsSrvAcceptResult<T> = Result<WsStream<T>, WsSrvHandshakeError<T>>;
type WssHandshakeError<T> = tungstenite::handshake::HandshakeError<
    tungstenite::handshake::client::ClientHandshake<TlsStream<T>>,
>;
type WssConnectResult<T> =
    Result<(WssStream<T>, tungstenite::handshake::client::Response), WssHandshakeError<T>>;
type WssSrvHandshakeError<T> = tungstenite::handshake::HandshakeError<
    tungstenite::handshake::server::ServerHandshake<
        TlsStream<T>,
        tungstenite::handshake::server::NoCallback,
    >,
>;
type WssSrvAcceptResult<T> = Result<WssStream<T>, WssSrvHandshakeError<T>>;
type TlsMidHandshake<T> = native_tls::MidHandshakeTlsStream<BaseStream<T>>;

type BaseStream<T> = T;
type TlsSrvMidHandshake<T> = native_tls::MidHandshakeTlsStream<BaseStream<T>>;
type TlsStream<T> = native_tls::TlsStream<BaseStream<T>>;
type WsMidHandshake<T> = tungstenite::handshake::MidHandshake<tungstenite::ClientHandshake<T>>;
type WsSrvMidHandshake<T> = tungstenite::handshake::MidHandshake<
    tungstenite::ServerHandshake<T, tungstenite::handshake::server::NoCallback>,
>;
type WssMidHandshake<T> =
    tungstenite::handshake::MidHandshake<tungstenite::ClientHandshake<TlsStream<T>>>;
type WssSrvMidHandshake<T> = tungstenite::handshake::MidHandshake<
    tungstenite::ServerHandshake<TlsStream<T>, tungstenite::handshake::server::NoCallback>,
>;
type WsStream<T> = tungstenite::protocol::WebSocket<T>;
type WssStream<T> = tungstenite::protocol::WebSocket<TlsStream<T>>;

type SocketMap<T> = std::collections::HashMap<Lib3hUri, WssInfo<T>>;
