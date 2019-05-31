//! This module provides a higher level interface to p2p / network messaging
//! basically handles serialization / deserialization from / to the core
//! protocol message types (NamedBinary and Json).

#![allow(non_snake_case)]

use failure::Error;
use holochain_core_types::{cas::content::Address, error::HolochainError, json::JsonString};
use std::convert::TryFrom;

use super::protocol::Protocol;

/// Tuple holding all the info required for identifying an EntryAspect:
/// (entry_address, aspect_address)
pub type AspectKey = (Address, Address);

fn get_default_state_id() -> String {
    "undefined".to_string()
}

fn get_default_state_bindings() -> Vec<String> {
    Vec::new()
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
pub struct StateData {
    pub state: String,
    #[serde(default = "get_default_state_id")]
    pub id: String,
    #[serde(default = "get_default_state_bindings")]
    pub bindings: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
pub struct ConfigData {
    pub config: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
pub struct ConnectData {
    #[serde(rename = "address")]
    pub peer_address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
pub struct PeerData {
    #[serde(rename = "agentId")]
    pub agent_id: Address,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
#[serde(rename_all = "camelCase")]
pub struct MessageData {
    pub dna_address: Address,
    #[serde(rename = "_id")]
    pub request_id: String,
    pub to_agent_id: Address,
    pub from_agent_id: Address,
    #[serde(rename = "data")]
    pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
#[serde(rename_all = "camelCase")]
pub struct TrackDnaData {
    pub dna_address: Address,
    pub agent_id: Address,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
#[serde(rename_all = "camelCase")]
pub struct GenericResultData {
    pub dna_address: Address,
    #[serde(rename = "_id")]
    pub request_id: String,
    pub to_agent_id: Address,
    pub result_info: Vec<u8>,
}

//--------------------------------------------------------------------------------------------------
// Entry
//--------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
#[serde(rename_all = "camelCase")]
pub struct EntryAspectData {
    pub aspect_address: Address,
    pub type_hint: String,
    pub aspect: Vec<u8>,
    pub publish_ts: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
#[serde(rename_all = "camelCase")]
pub struct EntryData {
    pub entry_address: Address,
    pub aspect_list: Vec<EntryAspectData>,
}

//--------------------------------------------------------------------------------------------------
// Query
//--------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryEntryData {
    pub dna_address: Address,
    #[serde(rename = "address")]
    pub entry_address: Address,
    #[serde(rename = "_id")]
    pub request_id: String,
    pub requester_agent_id: Address,
    pub query: Vec<u8>, // opaque query struct
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryEntryResultData {
    pub dna_address: Address,
    #[serde(rename = "address")]
    pub entry_address: Address,
    #[serde(rename = "_id")]
    pub request_id: String,
    pub requester_agent_id: Address,
    pub responder_agent_id: Address,
    pub query_result: Vec<u8>, // opaque query-result struct
}

//--------------------------------------------------------------------------------------------------
// Publish & Store
//--------------------------------------------------------------------------------------------------

///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProvidedEntryData {
    pub dna_address: Address,
    pub provider_agent_id: Address,
    pub entry: EntryData,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
#[serde(rename_all = "camelCase")]
pub struct StoreEntryAspectData {
    #[serde(rename = "_id")]
    pub request_id: String,
    pub dna_address: Address,
    pub provider_agent_id: Address,
    #[serde(rename = "address")]
    pub entry_address: Address,
    pub entry_aspect: EntryAspectData,
}

//--------------------------------------------------------------------------------------------------
// Gossip
//--------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
#[serde(rename_all = "camelCase")]
pub struct FetchEntryData {
    pub dna_address: Address,
    pub provider_agent_id: Address,
    #[serde(rename = "_id")]
    pub request_id: String,
    #[serde(rename = "address")]
    pub entry_address: Address,
    pub aspect_address_list: Option<Vec<Address>>, // None -> Get all, otherwise get specified aspects
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
#[serde(rename_all = "camelCase")]
pub struct FetchEntryResultData {
    pub dna_address: Address,
    pub provider_agent_id: Address,
    #[serde(rename = "_id")]
    pub request_id: String,
    pub entry: EntryData,
}

//--------------------------------------------------------------------------------------------------
// Get Lists
//--------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
#[serde(rename_all = "camelCase")]
pub struct GetListData {
    pub dna_address: Address,
    #[serde(rename = "_id")]
    pub request_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
#[serde(rename_all = "camelCase")]
pub struct EntryListData {
    pub dna_address: Address,
    #[serde(rename = "_id")]
    pub request_id: String,
    pub address_map: std::collections::HashMap<Address, Vec<Address>>, // Aspect addresses per entry
}

//--------------------------------------------------------------------------------------------------
// Enum
//--------------------------------------------------------------------------------------------------

/// Enum holding all message types that serialize as json in the 'hc-core <-> P2P network module' protocol.
/// There are 4 categories of messages:
///  - Command: An order from the local node to the p2p module. Local node expects a reponse. Starts with a verb.
///  - Handle-command: An order from the p2p module to the local node. The p2p module expects a response. Start withs 'Handle' followed by a verb.
///  - Result: A response to a Command. Starts with the name of the Command it responds to and ends with 'Result'.
///  - Notification: Notify that something happened. Not expecting any response. Ends with verb in past form, i.e. '-ed'.
/// Fetch = Request between node and the network (other nodes)
/// Get   = Request within a node between p2p module and core
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
#[serde(rename_all = "camelCase", tag = "method")]
pub enum JsonProtocol {
    // -- Generic responses -- //
    /// Success response to a request (any message with an _id field.)
    SuccessResult(GenericResultData),
    /// Failure response to a request (any message with an _id field.)
    /// Can also be a response to a mal-formed request.
    FailureResult(GenericResultData),

    // -- Connection -- //
    /// Order the p2p module to be part of the network of the specified DNA.
    TrackDna(TrackDnaData),
    /// Order the p2p module to leave the network of the specified DNA.
    UntrackDna(TrackDnaData),

    /// Connect to the specified multiaddr
    Connect(ConnectData),
    /// Notification of a connection from another peer.
    PeerConnected(PeerData),

    // -- Config (deprecated?) -- //
    /// Request the current state from the p2p module
    #[serde(rename = "requestState")]
    GetState,
    /// p2p module's current state response.
    #[serde(rename = "state")]
    GetStateResult(StateData),
    /// Request the default config from the p2p module
    #[serde(rename = "requestDefaultConfig")]
    GetDefaultConfig,
    /// the p2p module's default config response
    #[serde(rename = "defaultConfig")]
    GetDefaultConfigResult(ConfigData),
    /// Set the p2p config
    SetConfig(ConfigData),

    // -- Direct Messaging -- //
    /// Send a message to another peer on the network
    SendMessage(MessageData),
    /// the response from a previous `SendMessage`
    SendMessageResult(MessageData),
    /// Request to handle a message another peer has sent us.
    HandleSendMessage(MessageData),
    /// Our response to a message from another peer.
    HandleSendMessageResult(MessageData),

    // -- Entry -- //
    /// Another node, or the network module itself is requesting data from us
    HandleFetchEntry(FetchEntryData),
    /// Successful data response for a `HandleFetchEntry` request
    HandleFetchEntryResult(FetchEntryResultData),

    /// Publish data to the dht.
    PublishEntry(ProvidedEntryData),
    /// Store data on a node's dht slice.
    HandleStoreEntryAspect(StoreEntryAspectData),

    // -- Query -- //
    /// Request some info / data from a Entry
    QueryEntry(QueryEntryData),
    QueryEntryResult(QueryEntryResultData),
    HandleQueryEntry(QueryEntryData),
    HandleQueryEntryResult(QueryEntryResultData),

    // -- Entry lists -- //
    HandleGetAuthoringEntryList(GetListData),
    HandleGetAuthoringEntryListResult(EntryListData),

    HandleGetGossipingEntryList(GetListData),
    HandleGetGossipingEntryListResult(EntryListData),
}

/// Conversions
impl<'a> TryFrom<&'a Protocol> for JsonProtocol {
    type Error = Error;
    fn try_from(p: &Protocol) -> Result<Self, Error> {
        if let Protocol::Json(json) = p {
            JsonProtocol::try_from(json).map_err(|e| format_err!("{:?}", e))
        } else {
            bail!("could not convert into JsonProtocol: {:?}", p);
        }
    }
}

impl TryFrom<Protocol> for JsonProtocol {
    type Error = Error;
    fn try_from(p: Protocol) -> Result<Self, Error> {
        JsonProtocol::try_from(&p)
    }
}

impl<'a> From<&'a JsonProtocol> for Protocol {
    fn from(w: &JsonProtocol) -> Self {
        Protocol::Json(JsonString::from(w))
    }
}

impl From<JsonProtocol> for Protocol {
    fn from(w: JsonProtocol) -> Self {
        Protocol::from(&w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_convert {
        ($e:expr) => {
            let orig = $e;
            let p = Protocol::from(orig.clone());
            let w = JsonProtocol::try_from(p).unwrap();
            assert_eq!(orig, w);
        };
    }

    fn test_aspect() -> EntryAspectData {
        EntryAspectData {
            aspect_address: "HkAspect".into(),
            type_hint: "test_aspect".into(),
            aspect: vec![1, 2, 3, 4],
            publish_ts: 42,
        }
    }

    fn test_entry() -> EntryData {
        EntryData {
            entry_address: "HkEntry".into(),
            aspect_list: vec![test_aspect()],
        }
    }

    #[test]
    fn it_can_convert_GetState() {
        test_convert!(JsonProtocol::GetState);
    }

    #[test]
    fn it_can_convert_GetStateResult() {
        test_convert!(JsonProtocol::GetStateResult(StateData {
            state: "test_state".to_string(),
            id: "test_id".to_string(),
            bindings: vec!["test_binding".to_string()],
        }));
    }

    #[test]
    fn it_can_convert_funky_state() {
        let w = JsonProtocol::try_from(JsonString::from_json(
            r#"{
            "method": "state",
            "state": "test_state"
        }"#,
        ))
        .unwrap();
        if let JsonProtocol::GetStateResult(s) = w {
            assert_eq!("undefined", &s.id);
            assert_eq!(0, s.bindings.len());
        } else {
            panic!("bad enum type");
        }
    }

    #[test]
    fn it_can_convert_GetDefaultConfig() {
        test_convert!(JsonProtocol::GetDefaultConfig);
    }

    #[test]
    fn it_can_convert_GetDefaultConfigResult() {
        test_convert!(JsonProtocol::GetDefaultConfigResult(ConfigData {
            config: "test".to_string(),
        }));
    }

    #[test]
    fn it_can_convert_SetConfig() {
        test_convert!(JsonProtocol::SetConfig(ConfigData {
            config: "test".to_string(),
        }));
    }

    #[test]
    fn it_can_convert_Connect() {
        test_convert!(JsonProtocol::Connect(ConnectData {
            peer_address: "test".into(),
        }));
    }

    #[test]
    fn it_can_convert_PeerConnected() {
        test_convert!(JsonProtocol::PeerConnected(PeerData {
            agent_id: "test_id".to_string(),
        }));
    }

    #[test]
    fn it_can_convert_SendMessage() {
        test_convert!(JsonProtocol::SendMessage(MessageData {
            dna_address: "test_dna".into(),
            request_id: "test_id".to_string(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            content: "hello".as_bytes(),
        }));
    }

    #[test]
    fn it_can_convert_SendMessageResult() {
        test_convert!(JsonProtocol::SendMessageResult(MessageData {
            dna_address: "test_dna".into(),
            request_id: "test_id".to_string(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            content: "hello".into(),
        }));
    }

    #[test]
    fn it_can_convert_HandleSendMessage() {
        test_convert!(JsonProtocol::HandleSendMessage(MessageData {
            dna_address: "test_dna".into(),
            request_id: "test_id".to_string(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            content: "hello".into(),
        }));
    }

    #[test]
    fn it_can_convert_HandleSendMessageResult() {
        test_convert!(JsonProtocol::HandleSendMessageResult(MessageData {
            dna_address: "test_dna".into(),
            request_id: "test_id".to_string(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            content: "hello".into(),
        }));
    }

    #[test]
    fn it_can_convert_FetchEntry() {
        test_convert!(JsonProtocol::FetchEntry(FetchEntryData {
            dna_address: "test_dna".into(),
            request_id: "test_id".to_string(),
            requester_agent_id: "test_to".to_string(),
            entry_address: "Hk42".into(),
            aspect_address_list: None,
        }));
    }
    #[test]
    fn it_can_convert_HandleFetchEntry() {
        test_convert!(JsonProtocol::HandleFetchEntry(FetchEntryData {
            dna_address: "test_dna".into(),
            request_id: "test_id".to_string(),
            requester_agent_id: "test_to".to_string(),
            entry_address: "Hk42".into(),
            aspect_address_list: None,
        }));
    }
    #[test]
    fn it_can_convert_HandleFetchEntryResult() {
        test_convert!(JsonProtocol::HandleFetchEntryResult(FetchEntryResultData {
            dna_address: "test_dna".into(),
            request_id: "test_id".to_string(),
            entry: test_entry(),
        }));
    }
    #[test]
    fn it_can_convert_PublishEntry() {
        test_convert!(JsonProtocol::PublishEntry(ProvidedEntryData {
            dna_address: "test_dna".into(),
            provider_agent_id: "test_from".to_string(),
            entry: test_entry(),
        }));
    }
    #[test]
    fn it_can_convert_HandleStoreEntry() {
        test_convert!(JsonProtocol::HandleStoreEntryAspect(StoreEntryAspectData {
            request_id: "req_id".to_string(),
            dna_address: "test_dna".into(),
            provider_agent_id: "test_from".to_string(),
            entry_address: "Hk42".into(),
            entry_aspect: test_aspect(),
        }));
    }

    // -- Query -- //

    #[test]
    fn it_can_convert_Query() {
        test_convert!(JsonProtocol::QueryEntry(QueryData {
            dna_address: "test_dna".into(),
            entry_address: "Hk42".into(),
            request_id: "test_id".to_string(),
            requester_agent_id: "test_from".to_string(),
            query: vec![4, 3, 2, 1],
        }));
    }
    #[test]
    fn it_can_convert_QueryResult() {
        test_convert!(JsonProtocol::QueryEntryResult(QueryResultData {
            dna_address: "test_dna".into(),
            entry_address: "Hk42".into(),
            request_id: "test_id".to_string(),
            requester_agent_id: "test_from".to_string(),
            requester_agent_id: "test_to".to_string(),
            query_result: vec![4, 3, 2, 1],
        }));
    }

    // -- Entry lists -- //

    #[test]
    fn it_can_convert_HandleGetAuthoringEntryList() {
        test_convert!(JsonProtocol::HandleGetAuthoringEntryList(GetListData {
            dna_address: "test_dna".into(),
            request_id: "test_id".to_string(),
        }));
    }
    #[test]
    fn it_can_convert_HandleGetAuthoringEntryListResult() {
        test_convert!(JsonProtocol::HandleGetAuthoringEntryListResult(
            EntryListData {
                dna_address: "test_dna".into(),
                request_id: "test_id".to_string(),
                entry_address_list: vec!["data1".into(), "data2".into()],
            }
        ));
    }
    #[test]
    fn it_can_convert_HandleGetGossipingEntryList() {
        test_convert!(JsonProtocol::HandleGetGossipingEntryList(GetListData {
            dna_address: "test_dna".into(),
            request_id: "test_id".to_string(),
        }));
    }
    #[test]
    fn it_can_convert_HandleGetGossipingEntryListResult() {
        test_convert!(JsonProtocol::HandleGetGossipingEntryListResult(
            EntryListData {
                dna_address: "test_dna".into(),
                request_id: "test_id".to_string(),
                entry_address_list: vec!["data1".into(), "data2".into()],
            }
        ));
    }
}
