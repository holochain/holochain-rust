//! This module contains message serialization structures.

use serde_bytes;

/// Client wishes to `send` a message to another node.
/// This message is an array of 3 `&[u8]` slices.
/// - index 0 : local message identifier
/// - index 1 : destination node address
/// - index 2 : message data
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MsgCliSend<'a>(
    #[serde(with = "serde_bytes")] pub &'a [u8],
    #[serde(with = "serde_bytes")] pub &'a [u8],
    #[serde(with = "serde_bytes")] pub &'a [u8],
);

/// Client wishes to send a `call` message to another node.
/// This message is an array of 4 `&[u8]` slices.
/// - index 0 : local message identifier
/// - index 1 : remote message identifier (can be the same as index 0)
/// - index 2 : destination node address
/// - index 3 : message data
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MsgCliCall<'a>(
    #[serde(with = "serde_bytes")] pub &'a [u8],
    #[serde(with = "serde_bytes")] pub &'a [u8],
    #[serde(with = "serde_bytes")] pub &'a [u8],
    #[serde(with = "serde_bytes")] pub &'a [u8],
);

/// Client wishes to respond to a `call` message another node sent.
/// This message is an array of 4 `&[u8]` slices.
/// - index 0 : local message identifier
/// - index 1 : remote message identifier
/// - index 2 : destination node address
/// - index 3 : message data
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MsgCliCallResp<'a>(
    #[serde(with = "serde_bytes")] pub &'a [u8],
    #[serde(with = "serde_bytes")] pub &'a [u8],
    #[serde(with = "serde_bytes")] pub &'a [u8],
    #[serde(with = "serde_bytes")] pub &'a [u8],
);

/// A server response to a client-intiated `ping` message.
/// This message is an array of 2 `f64` millisecond epoch timestamp values.
/// - index 0 : the echoed initiation time of the originating `ping` message
/// - index 1 : the timestamp at which the server received / responded to the originating `ping` message
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgSrvPong(pub f64, pub f64);

/// A server response `success` to a `send` or `call` message.
/// This message contains a single `Vec<u8>` value representing the outgoing local message id from the `send` or `call message.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgSrvRespOk(#[serde(with = "serde_bytes")] pub Vec<u8>);

/// A server response `failure` to a `send` or `call` message.
/// This message is a tuple of 3 values.
/// - index 0 (`Vec<u8>`): the outgoing local message id from the `send` or `call message.
/// - index 1 (`i64`): an error code integer
/// - index 2 (`String`): error message text
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgSrvRespFail(
    #[serde(with = "serde_bytes")] pub Vec<u8>,
    pub i64,
    pub String,
);

/// This message represents this client receiving a `send` message from another node.
/// This message is an array of 2 `Vec<u8>` values.
/// - index 0 : the address of the originating node
/// - index 1 : message data
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgSrvRecvSend(
    #[serde(with = "serde_bytes")] pub Vec<u8>,
    #[serde(with = "serde_bytes")] pub Vec<u8>,
);

/// This message represents this client receiving a `call` message from another node.
/// This message is an array of 3 `Vec<u8>` values.
/// - index 0 : remote message identifier (should be echoed in the sent RecvCallResp).
/// - index 1 : the address of the originating node
/// - index 2 : message data
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgSrvRecvCall(
    #[serde(with = "serde_bytes")] pub Vec<u8>,
    #[serde(with = "serde_bytes")] pub Vec<u8>,
    #[serde(with = "serde_bytes")] pub Vec<u8>,
);

/// This message represents this client receiving a `call_resp` message from another node to a `call` message we had previously sent.
/// This message is an array of 3 `Vec<u8>` values.
/// - index 0 : remote message identifier (that we had previously sent with the `call` message).
/// - index 1 : the address of the originating node
/// - index 2 : message data
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgSrvRecvCallResp(
    #[serde(with = "serde_bytes")] pub Vec<u8>,
    #[serde(with = "serde_bytes")] pub Vec<u8>,
    #[serde(with = "serde_bytes")] pub Vec<u8>,
);

/// This enum is an amalgomation of all the server-sent message types to be used as a return type when receiving messages.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    SrvPong(MsgSrvPong),
    SrvRespOk(MsgSrvRespOk),
    SrvRespFail(MsgSrvRespFail),
    SrvRecvSend(MsgSrvRecvSend),
    SrvRecvCall(MsgSrvRecvCall),
    SrvRecvCallResp(MsgSrvRecvCallResp),
}
