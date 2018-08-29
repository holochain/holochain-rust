//! This module contains message serialization structures.

use serde_bytes;

/// Send a ping to the IPC server
/// This message is an array of 1 `f64` millisecond epoch timestamp value.
/// - index 0 : the current system timestamp of this system
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MsgPingSend(pub f64);

/// A server response to a client-intiated `ping` message.
/// This message is an array of 2 `f64` millisecond epoch timestamp values.
/// - index 0 : the echoed initiation time of the originating `ping` message
/// - index 1 : the timestamp at which the server received / responded to the originating `ping` message
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgPongRecv(pub f64, pub f64);

/// Client wishes to send a `call` message to another node.
/// This message is an array of 2 `&[u8]` slices.
/// - index 0 : message identifier
/// - index 1 : message data
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MsgCallSend<'a>(
    #[serde(with = "serde_bytes")] pub &'a [u8],
    #[serde(with = "serde_bytes")] pub &'a [u8],
);

/// This message represents this client receiving a `call` message.
/// This message is an array of 2 `Vec<u8>` values.
/// - index 0 : message identifier
/// - index 1 : message data
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgCallRecv(
    #[serde(with = "serde_bytes")] pub Vec<u8>,
    #[serde(with = "serde_bytes")] pub Vec<u8>,
);

/// Client wishes to respond with success to a `call` message.
/// This message is an array of 2 `&[u8]` slices.
/// - index 0 : message identifier
/// - index 1 : message data
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MsgCallOkSend<'a>(
    #[serde(with = "serde_bytes")] pub &'a [u8],
    #[serde(with = "serde_bytes")] pub &'a [u8],
);

/// This message represents this client receiving a success response message.
/// This message is an array of 2 `Vec<u8>` values.
/// - index 0 : message identifier
/// - index 1 : message data
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgCallOkRecv(
    #[serde(with = "serde_bytes")] pub Vec<u8>,
    #[serde(with = "serde_bytes")] pub Vec<u8>,
);

/// Client wishes to respond with an error to a `call` message.
/// This message is an array of 2 `&[u8]` slices.
/// - index 0 : message identifier
/// - index 1 : message data
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MsgCallFailSend<'a>(
    #[serde(with = "serde_bytes")] pub &'a [u8],
    #[serde(with = "serde_bytes")] pub &'a [u8],
);

/// This message represents this client receiving an error response message.
/// This message is an array of 2 `Vec<u8>` values.
/// - index 0 : message identifier
/// - index 1 : message data
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgCallFailRecv(
    #[serde(with = "serde_bytes")] pub Vec<u8>,
    #[serde(with = "serde_bytes")] pub Vec<u8>,
);

/// This enum is an amalgomation of all the server-sent message types to be used as a return type when receiving messages.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Pong(MsgPongRecv),
    Call(MsgCallRecv),
    CallOk(MsgCallOkRecv),
    CallFail(MsgCallFailRecv),
}
