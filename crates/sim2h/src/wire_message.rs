//! encapsulates lib3h ghostmessage for sim2h including security challenge
use crate::error::Sim2hError;

use holochain_tracing as ht;
use holochain_tracing_macros::newrelic_autotrace;
use lib3h_protocol::{data_types::Opaque, protocol::*};
use std::convert::TryFrom;

pub type WireMessageVersion = u32;
pub const WIRE_VERSION: WireMessageVersion = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WireError {
    MessageWhileInLimbo,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusData {
    pub spaces: usize,
    pub connections: usize,
    pub joined_connections: usize,
    pub redundant_count: u64,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HelloData {
    pub redundant_count: u64,
    pub version: u32,
    pub extra: Option<String>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WireMessage {
    ClientToLib3h(ht::EncodedSpanWrap<ClientToLib3h>),
    ClientToLib3hResponse(ht::EncodedSpanWrap<ClientToLib3hResponse>),
    Lib3hToClient(ht::EncodedSpanWrap<Lib3hToClient>),
    Lib3hToClientResponse(ht::EncodedSpanWrap<Lib3hToClientResponse>),
    MultiSend(Vec<ht::EncodedSpanWrap<Lib3hToClient>>),
    Err(WireError),
    Ping,
    Pong,
    Hello(WireMessageVersion),
    HelloResponse(HelloData),
    Status,
    StatusResponse(StatusData),
    Ack(u64),
    TraceFilter(String),
    TraceFilterResponse(String),
}

#[newrelic_autotrace(SIM2H)]
impl WireMessage {
    pub fn message_type(&self) -> String {
        String::from(match self {
            WireMessage::Ping => "Ping",
            WireMessage::Pong => "Pong",
            WireMessage::Status => "Status",
            WireMessage::StatusResponse(_) => "StatusResponse",
            WireMessage::Hello(_) => "Hello",
            WireMessage::HelloResponse(_) => "HelloResponse",
            WireMessage::TraceFilter(_) => "TraceFilter",
            WireMessage::TraceFilterResponse(_) => "TraceFilterResponse",
            WireMessage::ClientToLib3h(span_wrap) => match span_wrap.data {
                ClientToLib3h::Bootstrap(_) => "[C>L]Bootstrap",
                ClientToLib3h::FetchEntry(_) => "[C>L]FetchEntry",
                ClientToLib3h::JoinSpace(_) => "[C>L]JoinSpace",
                ClientToLib3h::LeaveSpace(_) => "[C>L]LeaveSpace",
                ClientToLib3h::PublishEntry(_) => "[C>L]PublishEntry",
                ClientToLib3h::QueryEntry(_) => "[C>L]QueryEntry",
                ClientToLib3h::SendDirectMessage(_) => "[C>L]SendDirectmessage",
            },
            WireMessage::ClientToLib3hResponse(span_wrap) => match span_wrap.data {
                ClientToLib3hResponse::BootstrapSuccess => "[C<L]BootsrapSuccess",
                ClientToLib3hResponse::FetchEntryResult(_) => "[C<L]FetchEntryResult",
                // TODO this needs to respond because core will never know it has joined.
                ClientToLib3hResponse::JoinSpaceResult => "[C<L]JoinSpaceResult",
                ClientToLib3hResponse::LeaveSpaceResult => "[C<L]LeaveSpaceResult",
                ClientToLib3hResponse::QueryEntryResult(_) => "[C<L]QueryEntryResult",
                ClientToLib3hResponse::SendDirectMessageResult(_) => "[C<L]SendDirectMessageResult",
            },
            WireMessage::Lib3hToClient(span_wrap) => match span_wrap.data {
                Lib3hToClient::Connected(_) => "[L>C]Connected",
                Lib3hToClient::HandleDropEntry(_) => "[L>C]HandleDropEntry",
                Lib3hToClient::HandleFetchEntry(_) => "[L>C]HandleFetchEntry",
                Lib3hToClient::HandleGetAuthoringEntryList(_) => "[L>C]HandleGetAuthoringList",
                Lib3hToClient::HandleGetGossipingEntryList(_) => "[L>C]HandleGetGossipingEntryList",
                Lib3hToClient::HandleQueryEntry(_) => "[L>C]HandleQueryEntry",
                Lib3hToClient::HandleSendDirectMessage(_) => "[L>C]HandleSendDirectMessage",
                Lib3hToClient::HandleStoreEntryAspect(_) => "[L>C]HandleStoreEntryAspect",
                Lib3hToClient::SendDirectMessageResult(_) => "[L>C]SendDirectMessageResult",
                Lib3hToClient::Unbound(_) => "[L>C]Unbound",
            },
            WireMessage::Lib3hToClientResponse(span_wrap) => match span_wrap.data {
                Lib3hToClientResponse::HandleDropEntryResult => "[L<C]HandleDropEntryResult",
                Lib3hToClientResponse::HandleFetchEntryResult(_) => "[L<C]HandleFetchEntryResult",
                Lib3hToClientResponse::HandleGetAuthoringEntryListResult(_) => {
                    "[L<C]HandleGetAuthoringEntryListResult"
                }
                Lib3hToClientResponse::HandleGetGossipingEntryListResult(_) => {
                    "[L<C]HandleGetGossipingEntryListResult"
                }
                Lib3hToClientResponse::HandleQueryEntryResult(_) => "[L<C]HandleQueryEntryResult",
                Lib3hToClientResponse::HandleSendDirectMessageResult(_) => {
                    "[L<C]HandleSendDirectMessageResult"
                }
                Lib3hToClientResponse::HandleStoreEntryAspectResult => {
                    "[L<C]HandleStoreEntryAspectResult"
                }
            },
            WireMessage::MultiSend(m) => {
                let messages: Vec<&Lib3hToClient> = m.iter().map(|w| &w.data).collect();
                get_multi_type(messages)
            }
            WireMessage::Err(_) => "[Error] {:?}",
            WireMessage::Ack(_) => "[Ack] {:?}",
        })
    }
    pub fn try_get_span(&self) -> Option<Vec<&ht::EncodedSpanContext>> {
        match self {
            WireMessage::ClientToLib3h(s) => s.span_context.as_ref().map(|s| vec![s]),
            WireMessage::ClientToLib3hResponse(s) => s.span_context.as_ref().map(|s| vec![s]),
            WireMessage::Lib3hToClient(s) => s.span_context.as_ref().map(|s| vec![s]),
            WireMessage::Lib3hToClientResponse(s) => s.span_context.as_ref().map(|s| vec![s]),
            WireMessage::MultiSend(m) => m.iter().map(|s| s.span_context.as_ref()).collect(),
            _ => None,
        }
    }
}

fn get_multi_type(list: Vec<&Lib3hToClient>) -> &str {
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
    use holochain_tracing::SpanWrap;
    use lib3h_protocol::{
        data_types::SpaceData,
        types::{AgentPubKey, SpaceHash},
    };

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

    fn wire_message_join_space() -> WireMessage {
        let c2l = ClientToLib3h::JoinSpace(SpaceData {
            request_id: String::from("0123"),
            space_address: SpaceHash::from("QmABCDEF"),
            agent_id: AgentPubKey::from("Hc345345"),
        });
        WireMessage::ClientToLib3h(SpanWrap::new(c2l, None).into())
    }

    #[test]
    pub fn test_wire_message_client_to_lib3h() {
        let msg = wire_message_join_space();
        let opaque_msg: Opaque = msg.clone().into();

        let roundtrip_msg = WireMessage::try_from(opaque_msg).expect("deserialize should work");
        assert_eq!(roundtrip_msg, msg);

        let roundtrip_string: String = roundtrip_msg.into();
        let msg_string: String = msg.into();
        assert_eq!(roundtrip_string, msg_string);
    }

    // This WireMessage is a real-world example that serializes non-deterministically
    #[allow(dead_code)]
    fn wire_message_app_spec_fixture() -> WireMessage {
        let raw = r#"{"Lib3hToClientResponse":{"data":{"HandleGetAuthoringEntryListResult":{"space_address":"QmQ7guHG2Y3fbtNaLoV1kFex66AqepoCTqQ9XtYYQKAFZK","provider_agent_id":"HcScJxNnN6Bi5d5tda7OHWGKNBgjq9oieP9GQXsmO5Svp8fa3gTK5DJQFwgditr","request_id":"","address_map":{"Qmey39PmjYAJ5r5bCKtWe4nMxVcTmTwLE3YN142kVe5CJE":["QmT3mV6mKsh4aEQoJ5J8feUruNGYTTd4FPuPicMxmZf8DY"],"HcScJxNnN6Bi5d5tda7OHWGKNBgjq9oieP9GQXsmO5Svp8fa3gTK5DJQFwgditr":["QmVr1H6B6P6iydnzCF7fh7abDz1yznrjecMwCSMmtGA4EN"],"QmW22euyQLF7wK8yYhnCZHZq64G7ryQ2TqQ5D3L7vhszg2":["QmWAU3DTuuwdNFPpX3gqEsG7bAttePbJQZjirrh39MfGxR"],"Qmavdnym3BKrKJxuNoSxLnoPwUBWtqsVhSnQmdmm4FFnyK":["QmPybN5GGibjAno6hmKCWJgM8RRkeo1vga57ZA3QbevrrL"]}}},"span_context":[149,217,162,104,57,50,215,185,128,95,199,101,105,81,143,213,10,14,105,185,134,247,194,247,0,0,0,0,0,0,0,0,1,0,0,0,0]}}"#;
        let opaque: Opaque = raw.into();
        WireMessage::try_from(opaque).unwrap()
    }

    // This test is broken because of the non-deterministic serialization issue of WireMessages.
    // TODO: make serialization of WireMessages deterministic by replacing the HashMap in
    // Lib3hProtocol with a BTreeMap or add a custom serialization routine.
    // Then this test should be activated to ensure that WireMessages' into::<String>()
    // always returns the same result if the WireMessage is the same.
    #[cfg(feature = "broken-tests")]
    #[test]
    pub fn test_wire_message_app_spec_fixture_deterministic_serialization() {
        let msg = wire_message_app_spec_fixture();
        let opaque_msg: Opaque = msg.clone().into();

        let roundtrip_msg = WireMessage::try_from(opaque_msg).expect("deserialize should work");
        assert_eq!(roundtrip_msg, msg);

        let roundtrip_string: String = roundtrip_msg.into();
        let msg_string: String = msg.into();
        assert_eq!(roundtrip_string, msg_string);
    }
}
