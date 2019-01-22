//! This module provides a higher level interface to p2p / network messaging
//! basically handles serialization / deserialization from / to the core
//! protocol message types (NamedBinary and Json).

use serde_json;

use failure::Error;
use holochain_core_types::{cas::content::Address, error::HolochainError, json::JsonString};
use std::convert::TryFrom;

use super::protocol::Protocol;

fn get_default_state_id() -> String {
    "undefined".to_string()
}

fn get_default_state_bindings() -> Vec<String> {
    Vec::new()
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct StateData {
    pub state: String,
    #[serde(default = "get_default_state_id")]
    pub id: String,
    #[serde(default = "get_default_state_bindings")]
    pub bindings: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct ConfigData {
    pub config: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct ConnectData {
    #[serde(rename = "address")]
    pub peer_address: Address,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct PeerData {
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "agentId")]
    pub agent_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct MessageData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "toAgentId")]
    pub to_agent_id: String,

    #[serde(rename = "fromAgentId")]
    pub from_agent_id: String,

    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct TrackDnaData {
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "agentId")]
    pub agent_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct SuccessResultData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "toAgentId")]
    pub to_agent_id: String,

    #[serde(rename = "successInfo")]
    pub success_info: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct FailureResultData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "toAgentId")]
    pub to_agent_id: String,

    #[serde(rename = "errorInfo")]
    pub error_info: serde_json::Value,
}

//--------------------------------------------------------------------------------------------------
// DHT Data
//--------------------------------------------------------------------------------------------------

/// Data Request from own p2p-module
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct DhtDataRequestData {
    #[serde(rename = "_id")]
    pub request_id: String,

    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "address")]
    pub data_address: String,
}

/// Data Request from some other agent
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct FetchDhtData {
    #[serde(rename = "fromAgentId")]
    pub requester_agent_id: String,

    // DhtDataRequestData
    #[serde(rename = "_id")]
    pub request_id: String,
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,
    #[serde(rename = "address")]
    pub data_address: String,
}

/// Generic DHT data message
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
pub struct DhtData {
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "agentId")]
    pub provider_agent_id: String,

    #[serde(rename = "address")]
    pub data_address: String,

    pub data_content: serde_json::Value,
}

/// DHT data response from a request
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
pub struct HandleDhtResultData {
    #[serde(rename = "_id")]
    pub request_id: String,
    pub requester_agent_id: String,

    // DhtData
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,
    #[serde(rename = "agentId")]
    pub provider_agent_id: String,
    #[serde(rename = "address")]
    pub data_address: String,
    pub data_content: serde_json::Value,
}

//--------------------------------------------------------------------------------------------------
// DHT metadata
//--------------------------------------------------------------------------------------------------

/// Metadata Request from another agent
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct FetchDhtMetaData {
    pub attribute: String,

    // FetchDhtData
    #[serde(rename = "fromAgentId")]
    pub requester_agent_id: String,
    #[serde(rename = "_id")]
    pub request_id: String,
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,
    #[serde(rename = "address")]
    pub data_address: String,
}

/// Generic DHT metadata message
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct DhtMetaData {
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "fromAgentId")]
    pub provider_agent_id: String,

    pub data_address: String,
    pub attribute: String,
    pub content: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct HandleDhtMetaResultData {
    #[serde(rename = "_id")]
    pub request_id: String,
    pub requester_agent_id: String,

    // DhtMetaData
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,
    #[serde(rename = "fromAgentId")]
    pub provider_agent_id: String,
    pub data_address: String,
    pub attribute: String,
    pub content: serde_json::Value,
}

//--------------------------------------------------------------------------------------------------
// List (publish & hold)
//--------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct GetListData {
    #[serde(rename = "_id")]
    pub request_id: String,
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct HandleListResultData {
    #[serde(rename = "dataAddressList")]
    pub data_address_list: Vec<Address>,

    // GetListData
    #[serde(rename = "_id")]
    pub request_id: String,
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,
}

//--------------------------------------------------------------------------------------------------
// JsonProtocol Enum
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
#[serde(tag = "method")]
pub enum JsonProtocol {
    /// Success response to any message with an _id field.
    #[serde(rename = "successResult")]
    SuccessResult(SuccessResultData),
    /// Failure response to any message with an _id field.
    /// Can also be a response to a mal-formed request.
    #[serde(rename = "failureResult")]
    FailureResult(FailureResultData),

    /// Order the p2p module to be part of the network of the specified DNA.
    #[serde(rename = "trackDna")]
    TrackDna(TrackDnaData),

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
    #[serde(rename = "setConfig")]
    SetConfig(ConfigData),

    /// Connect to the specified multiaddr
    #[serde(rename = "connect")]
    Connect(ConnectData),
    /// Notification of a connection from another peer.
    #[serde(rename = "peerConnected")]
    PeerConnected(PeerData),

    /// Send a message to another peer on the network
    #[serde(rename = "sendMessage")]
    SendMessage(MessageData),
    /// the response from a previous `SendMessage`
    #[serde(rename = "sendMessageResult")]
    SendMessageResult(MessageData),
    /// Request to handle a message another peer has sent us.
    #[serde(rename = "handleSendMessage")]
    HandleSendMessage(MessageData),
    /// Our response to a message from another peer.
    #[serde(rename = "handleSendMessageResult")]
    HandleSendMessageResult(MessageData),

    /// Request data from the dht network
    #[serde(rename = "fetchDht")]
    FetchDhtData(FetchDhtData),
    /// Response from requesting dht data from the network
    #[serde(rename = "fetchDhtResult")]
    FetchDhtDataResult(HandleDhtResultData),
    /// Another node, or the network module itself is requesting data from us
    #[serde(rename = "handleFetchDht")]
    HandleFetchDhtData(FetchDhtData),
    /// Successful data response for a `HandleFetchDhtData` request
    #[serde(rename = "handleFetchDhtResult")]
    HandleFetchDhtDataResult(HandleDhtResultData),

    /// Publish data to the dht.
    #[serde(rename = "publishDht")]
    PublishDhtData(DhtData),
    /// Store data on a node's dht slice.
    #[serde(rename = "handleStoreDht")]
    HandleStoreDhtData(DhtData),

    /// Request metadata from the dht
    #[serde(rename = "fetchDhtMeta")]
    FetchDhtMeta(FetchDhtMetaData),
    /// Response by the network for our metadata request
    #[serde(rename = "fetchDhtMetaResult")]
    FetchDhtMetaResult(HandleDhtMetaResultData),
    /// Another node, or the network module itself, is requesting data from us
    #[serde(rename = "handleFetchDhtMeta")]
    HandleFetchDhtMeta(FetchDhtMetaData),
    /// Successful metadata response for a `HandleGetDhtMeta` request
    #[serde(rename = "handleFetchDhtMetaResult")]
    HandleFetchDhtMetaResult(HandleDhtMetaResultData),

    /// Publish metadata to the dht.
    #[serde(rename = "publishDhtMeta")]
    PublishDhtMeta(DhtMetaData),
    /// Store metadata on a node's dht slice.
    #[serde(rename = "handleStoreDhtMeta")]
    HandleStoreDhtMeta(DhtMetaData),



    #[serde(rename = "handleGetPublishingDataList")]
    HandleGetPublishingDataList(GetListData),
    #[serde(rename = "handleGetPublishingDataListResult")]
    HandleGetPublishingDataListResult(HandleListResultData),

    #[serde(rename = "handleGetHoldingDataList")]
    HandleGetHoldingDataList(GetListData),
    #[serde(rename = "handleGetHoldingDataListResult")]
    HandleGetHoldingDataListResult(HandleListResultData),
//    #[serde(rename = "handleGetDhtData")]
//    HandleGetDhtData(GetDhtData),
//    #[serde(rename = "handleGetDhtDataResult")]
//    HandleGetDhtDataResult(DhtData),
    #[serde(rename = "handleDropDhtData")]
    HandleDropDhtData(DhtDataRequestData),

}

impl<'a> TryFrom<&'a Protocol> for JsonProtocol {
    type Error = Error;
    fn try_from(p: &Protocol) -> Result<Self, Error> {
        if let Protocol::Json(json) = p {
            match JsonProtocol::try_from(json) {
                Ok(w) => {
                    return Ok(w);
                }
                Err(e) => bail!("{:?}", e),
            };
        }
        bail!("could not convert into JsonProtocol: {:?}", p);
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

    #[test]
    fn it_can_convert_request_state() {
        test_convert!(JsonProtocol::GetState);
    }

    #[test]
    fn it_can_convert_state() {
        test_convert!(JsonProtocol::GetStateResult(StateData {
            state: "test_state".to_string(),
            id: "test_id".to_string(),
            bindings: vec!["test_binding".to_string()],
        }));
    }

    #[test]
    fn it_can_convert_funky_state() {
        let w = JsonProtocol::try_from(JsonString::from(
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
    fn it_can_convert_request_default_config() {
        test_convert!(JsonProtocol::GetDefaultConfig);
    }

    #[test]
    fn it_can_convert_default_config() {
        test_convert!(JsonProtocol::GetDefaultConfigResult(ConfigData {
            config: "test".to_string(),
        }));
    }

    #[test]
    fn it_can_convert_set_config() {
        test_convert!(JsonProtocol::SetConfig(ConfigData {
            config: "test".to_string(),
        }));
    }

    #[test]
    fn it_can_convert_set_connect() {
        test_convert!(JsonProtocol::Connect(ConnectData {
            address: "test".into(),
        }));
    }

    #[test]
    fn it_can_convert_peer_connected() {
        test_convert!(JsonProtocol::PeerConnected(PeerData {
            dna_address: "test_dna".into(),
            agent_id: "test_id".to_string(),
        }));
    }

    #[test]
    fn it_can_convert_send_message() {
        test_convert!(JsonProtocol::SendMessage(MessageData {
            dna_address: "test_dna".into(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            msg_id: "test_id".to_string(),
            data: json!("hello"),
        }));
    }

    #[test]
    fn it_can_convert_send_result() {
        test_convert!(JsonProtocol::SendMessageResult(MessageData {
            dna_address: "test_dna".into(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            msg_id: "test_id".to_string(),
            data: json!("hello"),
        }));
    }

    #[test]
    fn it_can_convert_handle_send() {
        test_convert!(JsonProtocol::HandleSendMessage(MessageData {
            dna_address: "test_dna".into(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            msg_id: "test_id".to_string(),
            data: json!("hello"),
        }));
    }

    #[test]
    fn it_can_convert_handle_send_result() {
        test_convert!(JsonProtocol::HandleSendMessageResult(MessageData {
            dna_address: "test_dna".into(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            msg_id: "test_id".to_string(),
            data: json!("hello"),
        }));
    }

    #[test]
    fn it_can_convert_HandleGetPublishingDataList() {
        test_convert!(JsonProtocol::HandleGetPublishingDataList(GetListData {
            msg_id: "test_id".to_string(),
            dna_address: "test_dna".into(),
        }));
    }

    #[test]
    fn it_can_convert_HandleGetPublishingDataListResult() {
        test_convert!(JsonProtocol::HandleGetPublishingDataListResult(ListData {
            msg_id: "test_id".to_string(),
            dna_address: "test_dna".into(),
            address_list: vec!["data1", "data2"],
        }));
    }

    #[test]
    fn it_can_convert_HandleGetHoldingDataList() {
        test_convert!(JsonProtocol::HandleGetHoldingDataList(GetListData {
            msg_id: "test_id".to_string(),
            dna_address: "test_dna".into(),
        }));
    }

    #[test]
    fn it_can_convert_HandleGetHoldingDataListResult() {
        test_convert!(JsonProtocol::HandleGetHoldingDataListResult(ListData {
            msg_id: "test_id".to_string(),
            dna_address: "test_dna".into(),
            address_list: vec!["data1", "data2"],
        }));
    }

    #[test]
    fn it_can_convert_HandleDropDhtData() {
        test_convert!(JsonProtocol::HandleDropDhtData(DropData {
            dna_address: "test_dna".into(),
            data_address: "data1".into(),
        }));
    }
}
