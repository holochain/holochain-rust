//! encapsulates lib3h ghostmessage for sim2h including security challenge
use crate::error::Sim2hError;
use lib3h_protocol::{data_types::Opaque, protocol::*};
use std::convert::TryFrom;

pub type WireMessageVersion = u32;
pub const WIRE_VERSION: WireMessageVersion = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WireError {
    MessageWhileInLimbo,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusData {
    pub spaces: usize,
    pub connections: usize,
    pub redundant_count: u64,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HelloData {
    pub redundant_count: u64,
    pub version: u32,
    pub extra: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WireMessage {
    ClientToLib3h(ClientToLib3h),
    ClientToLib3hResponse(ClientToLib3hResponse),
    Lib3hToClient(Lib3hToClient),
    Lib3hToClientResponse(Lib3hToClientResponse),
    MultiSend(Vec<Lib3hToClient>),
    Err(WireError),
    Ping,
    Pong,
    Hello(WireMessageVersion),
    HelloResponse(HelloData),
    Status,
    StatusResponse(StatusData),
}

impl WireMessage {
    pub fn message_type(&self) -> String {
        String::from(match self {
            WireMessage::Ping => "Ping",
            WireMessage::Pong => "Pong",
            WireMessage::Status => "Status",
            WireMessage::StatusResponse(_) => "StatusResponse",
            WireMessage::Hello(_) => "Hello",
            WireMessage::HelloResponse(_) => "HelloResponse",
            WireMessage::ClientToLib3h(ClientToLib3h::Bootstrap(_)) => "[C>L]Bootstrap",
            WireMessage::ClientToLib3h(ClientToLib3h::FetchEntry(_)) => "[C>L]FetchEntry",
            WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(_)) => "[C>L]JoinSpace",
            WireMessage::ClientToLib3h(ClientToLib3h::LeaveSpace(_)) => "[C>L]LeaveSpace",
            WireMessage::ClientToLib3h(ClientToLib3h::PublishEntry(_)) => "[C>L]PublishEntry",
            WireMessage::ClientToLib3h(ClientToLib3h::QueryEntry(_)) => "[C>L]QueryEntry",
            WireMessage::ClientToLib3h(ClientToLib3h::SendDirectMessage(_)) => {
                "[C>L]SendDirectmessage"
            }
            WireMessage::ClientToLib3hResponse(ClientToLib3hResponse::BootstrapSuccess) => {
                "[C<L]BootsrapSuccess"
            }
            WireMessage::ClientToLib3hResponse(ClientToLib3hResponse::FetchEntryResult(_)) => {
                "[C<L]FetchEntryResult"
            }
            WireMessage::ClientToLib3hResponse(ClientToLib3hResponse::JoinSpaceResult) => {
                "[C<L]JoinSpaceResult"
            }
            WireMessage::ClientToLib3hResponse(ClientToLib3hResponse::LeaveSpaceResult) => {
                "[C<L]LeaveSpaceResult"
            }
            WireMessage::ClientToLib3hResponse(ClientToLib3hResponse::QueryEntryResult(_)) => {
                "[C<L]QueryEntryResult"
            }
            WireMessage::ClientToLib3hResponse(ClientToLib3hResponse::SendDirectMessageResult(
                _,
            )) => "[C<L]SendDirectMessageResult",
            WireMessage::Lib3hToClient(Lib3hToClient::Connected(_)) => "[L>C]Connected",
            WireMessage::Lib3hToClient(Lib3hToClient::HandleDropEntry(_)) => "[L>C]HandleDropEntry",
            WireMessage::Lib3hToClient(Lib3hToClient::HandleFetchEntry(_)) => {
                "[L>C]HandleFetchEntry"
            }
            WireMessage::Lib3hToClient(Lib3hToClient::HandleGetAuthoringEntryList(_)) => {
                "[L>C]HandleGetAuthoringList"
            }
            WireMessage::Lib3hToClient(Lib3hToClient::HandleGetGossipingEntryList(_)) => {
                "[L>C]HandleGetGossipingEntryList"
            }
            WireMessage::Lib3hToClient(Lib3hToClient::HandleQueryEntry(_)) => {
                "[L>C]HandleQueryEntry"
            }
            WireMessage::Lib3hToClient(Lib3hToClient::HandleSendDirectMessage(_)) => {
                "[L>C]HandleSendDirectMessage"
            }
            WireMessage::Lib3hToClient(Lib3hToClient::HandleStoreEntryAspect(_)) => {
                "[L>C]HandleStoreEntryAspect"
            }
            WireMessage::Lib3hToClient(Lib3hToClient::SendDirectMessageResult(_)) => {
                "[L>C]SendDirectMessageResult"
            }
            WireMessage::Lib3hToClient(Lib3hToClient::Unbound(_)) => "[L>C]Unbound",
            WireMessage::Lib3hToClientResponse(Lib3hToClientResponse::HandleDropEntryResult) => {
                "[L<C]HandleDropEntryResult"
            }
            WireMessage::Lib3hToClientResponse(Lib3hToClientResponse::HandleFetchEntryResult(
                _,
            )) => "[L<C]HandleFetchEntryResult",
            WireMessage::Lib3hToClientResponse(
                Lib3hToClientResponse::HandleGetAuthoringEntryListResult(_),
            ) => "[L<C]HandleGetAuthoringEntryListResult",
            WireMessage::Lib3hToClientResponse(
                Lib3hToClientResponse::HandleGetGossipingEntryListResult(_),
            ) => "[L<C]HandleGetGossipingEntryListResult",
            WireMessage::Lib3hToClientResponse(Lib3hToClientResponse::HandleQueryEntryResult(
                _,
            )) => "[L<C]HandleQueryEntryResult",
            WireMessage::Lib3hToClientResponse(
                Lib3hToClientResponse::HandleSendDirectMessageResult(_),
            ) => "[L<C]HandleSendDirectMessageResult",
            WireMessage::Lib3hToClientResponse(
                Lib3hToClientResponse::HandleStoreEntryAspectResult,
            ) => "[L<C]HandleStoreEntryAspectResult",
            WireMessage::MultiSend(m) => get_multi_type(&m),
            WireMessage::Err(_) => "[Error] {:?}",
        })
    }
}

fn get_multi_type(list: &Vec<Lib3hToClient>) -> &str {
    if list.len() > 0 {
        match list.get(0).unwrap() {
            Lib3hToClient::HandleFetchEntry(_) => "[L>C]MultiSend::HandleFetchEntry",
            Lib3hToClient::HandleStoreEntryAspect(_) => "[L>C]MultiSend::HandleStoreEntryAspect",
            _ => "[L>C]MultiSend::UNEXPECTED_VARIANT",
        }
    } else {
        "[L>C]MultiSend::EMPTY_SEND"
    }
}

impl From<WireMessage> for Opaque {
    fn from(message: WireMessage) -> Opaque {
        serde_json::to_string(&message)
            .expect("wiremessage should serialize")
            .into()
    }
}

impl From<WireMessage> for String {
    fn from(message: WireMessage) -> String {
        serde_json::to_string(&message).expect("wiremessage should serialize")
    }
}

impl TryFrom<Opaque> for WireMessage {
    type Error = WireError;
    fn try_from(message: Opaque) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&String::from_utf8_lossy(&message))
            .map_err(|e| format!("{:?}", e))?)
    }
}

impl TryFrom<&Opaque> for WireMessage {
    type Error = WireError;
    fn try_from(message: &Opaque) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&String::from_utf8_lossy(message))
            .map_err(|e| format!("{:?}", e))?)
    }
}

impl From<&str> for WireError {
    fn from(err: &str) -> Self {
        WireError::Other(format!("{:?}", err))
    }
}

impl From<String> for WireError {
    fn from(err: String) -> Self {
        WireError::Other(err)
    }
}

impl From<WireError> for Sim2hError {
    fn from(err: WireError) -> Sim2hError {
        format!("{:?}", err).into()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_wire_message() {
        let msg = WireMessage::Err("fake_error".into());

        let opaque_msg: Opaque = msg.clone().into();
        assert_eq!(
            "\"{\\\"Err\\\":{\\\"Other\\\":\\\"\\\\\\\"fake_error\\\\\\\"\\\"}}\"",
            format!("{}", opaque_msg)
        );
        let roundtrip_msg = WireMessage::try_from(opaque_msg).expect("deserialize should work");
        assert_eq!(roundtrip_msg, msg);
    }
    #[test]
    pub fn test_wire_message_version() {
        let msg = WireMessage::Hello(1);
        let opaque_msg: Opaque = msg.clone().into();
        assert_eq!("\"{\\\"Hello\\\":1}\"", format!("{}", opaque_msg));
        let roundtrip_msg = WireMessage::try_from(opaque_msg).expect("deserialize should work");
        assert_eq!(roundtrip_msg, msg);
    }
    #[test]
    pub fn test_wire_message_ping() {
        let msg = WireMessage::Ping;
        let opaque_msg: Opaque = msg.clone().into();
        assert_eq!("\"\\\"Ping\\\"\"", format!("{}", opaque_msg));
        let roundtrip_msg = WireMessage::try_from(opaque_msg).expect("deserialize should work");
        assert_eq!(roundtrip_msg, msg);
    }
}
