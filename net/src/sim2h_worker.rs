//! provides worker that makes use of sim2h

use crate::connection::{
    net_connection::{NetHandler, NetWorker},
    NetResult,
};
use holochain_json_api::{error::JsonError, json::JsonString};
use lib3h_protocol::{protocol_client::Lib3hClientProtocol, protocol_server::Lib3hServerProtocol, Address};
use log::*;
use sim2h::WireMessage;
use url::Url;
use lib3h::transport::websocket::actor::{GhostTransportWebsocket};
use lib3h::transport::websocket::tls::TlsConfig;
use lib3h::transport::protocol::{TransportActorParentWrapper, RequestToChild, RequestToChildResponse};
use holochain_tracing::Span;
use lib3h_zombie_actor::{GhostParentWrapper, GhostCallbackData};
use lib3h_zombie_actor::GhostCanTrack;
use failure::err_msg;
use lib3h_protocol::protocol::*;
use lib3h_protocol::data_types::{GenericResultData, Opaque};

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct Sim2hConfig {
    pub sim2h_url: String,
}

/// removed lifetime parameter because compiler says ghost engine needs lifetime that could live statically
#[allow(non_snake_case, dead_code)]
pub struct Sim2hWorker {
    handler: NetHandler,
    transport: TransportActorParentWrapper<Sim2hWorker, GhostTransportWebsocket>,
    inbox: Vec<Lib3hClientProtocol>,
    to_core: Vec<Lib3hServerProtocol>,
    num_ticks: u32,
    config: Sim2hConfig,
}

impl Sim2hWorker {
    pub fn advertise(self) -> url::Url {
        Url::parse("ws://example.com").unwrap()
    }

    /// Create a new worker connected to the sim2h instance
    pub fn new(handler: NetHandler, config: Sim2hConfig) -> NetResult<Self> {
        let transport_raw = GhostTransportWebsocket::new(
            Address::from("sim2h-worker-transport"),
            TlsConfig::Unencrypted,
            Address::from("sim2h-network"),
        );

        let mut transport: TransportActorParentWrapper<Sim2hWorker, GhostTransportWebsocket> =
            GhostParentWrapper::new(
                transport_raw,
                "t1_requests", // prefix for request ids in the tracker
            );


        // bind to some port:
        // channel for making an async call sync
        let (tx, rx) = crossbeam_channel::unbounded();
        transport.request(
            Span::todo("Find out how to use spans the right way"),
            RequestToChild::Bind {
                spec: Url::parse("wss://localhost:38220").expect("can parse url").into(),
            },
            // callback just notifies channel so
            Box::new(move |_owner, response| {
                let result = match response {
                    GhostCallbackData::Timeout(bt) => Err(format!("Bind timed out. Backtrace: {:?}", bt)),
                    GhostCallbackData::Response(r) => match r {
                        Ok(response) => match response {
                            RequestToChildResponse::Bind(bind_result_data) => Ok(bind_result_data.bound_url),
                            _ => Err(String::from("Got unexpected response from transport actor during bind")),
                        }
                        Err(transport_error) => Err(format!("Error during bind: {:?}", transport_error)),
                    }
                };
                let _ = tx.send(result);
                Ok(())
            }),
        )?;

        let result = rx.recv()?;
        let _bound_url = result.map_err(|bind_error| err_msg(bind_error))?;

        Ok(Self {
            handler,
            transport,
            inbox: Vec::new(),
            to_core: Vec::new(),
            num_ticks: 0,
            config,
        })
    }

    fn send_wire_message(&mut self, message: WireMessage) -> NetResult<()> {
        self.transport.request(
            Span::todo("Find out how to use spans the right way"),
            RequestToChild::SendMessage {
                uri: Url::parse(&self.config.sim2h_url)
                    .expect("Sim2h URL can't be parsed")
                    .into(),
                payload: message.into(),
            },
            // callback just notifies channel so
            Box::new(move |_owner, response| {
                match response {
                    GhostCallbackData::Response(Ok(RequestToChildResponse::SendMessageSuccess))
                        => trace!("Success sending wire message"),
                    GhostCallbackData::Response(Err(e))
                        => error!("Error sending wire message: {:?}",e),
                    GhostCallbackData::Timeout(bt)
                        => error!("Timeout sending wire message: {:?}", bt),
                    _ => error!("Got bad response type from transport actor when sending wire message"),
                };
                Ok(())
            }),
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    fn handle_client_message(
        &mut self,
        data: Lib3hClientProtocol,
    ) -> NetResult<()> {
        debug!("handle_client_message: {:?}", data);
        match data {
            // Success response to a request (any Command with an `request_id` field.)
            Lib3hClientProtocol::SuccessResult(generic_result_data) => {
                self.to_core.push(Lib3hServerProtocol::FailureResult(generic_result_data));
                Ok(())
            }
            // Connect to the specified multiaddr
            Lib3hClientProtocol::Connect(connect_data) => {
                self.to_core.push(Lib3hServerProtocol::FailureResult(GenericResultData {
                    request_id: connect_data.request_id,
                    space_address: Address::new().into(),
                    to_agent_id: Address::new(),
                    result_info: Opaque::new(),
                }));
                Ok(())
            }

            // -- Space -- //
            // Order the p2p module to be part of the network of the specified space.
            Lib3hClientProtocol::JoinSpace(space_data) => {
                //let log_context = "ClientToLib3h::JoinSpace";
                self.send_wire_message(WireMessage::ClientToLib3h(
                    ClientToLib3h::JoinSpace(space_data)
                ))
            }
            // Order the p2p module to leave the network of the specified space.
            Lib3hClientProtocol::LeaveSpace(_space_data) => {
                error!("Leave space not implemented for sim2h yet");
                //let log_context = "ClientToLib3h::LeaveSpace";
                //let _ = leave_space(&log_context, &self.dynamo_db_client, &space_data)?;
                Ok(())
            }

            // -- Direct Messaging -- //
            // Send a message directly to another agent on the network
            Lib3hClientProtocol::SendDirectMessage(dm_data) => {
                //let log_context = "ClientToLib3h::SendDirectMessage";
                self.send_wire_message(WireMessage::ClientToLib3h(
                    ClientToLib3h::SendDirectMessage(dm_data)
                ))
            }
            // Our response to a direct message from another agent.
            Lib3hClientProtocol::HandleSendDirectMessageResult(dm_data) => {
                //let log_context = "ClientToLib3h::HandleSendDirectMessageResult";
                self.send_wire_message(WireMessage::ClientToLib3hResponse(
                    ClientToLib3hResponse::SendDirectMessageResult(dm_data)
                ))
            }
            // -- Entry -- //
            // Request an Entry from the dht network
            Lib3hClientProtocol::FetchEntry(_fetch_entry_data) => {
                panic!("FetchEntry send by core - this should never happen");
            }
            // Successful data response for a `HandleFetchEntryData` request
            Lib3hClientProtocol::HandleFetchEntryResult(fetch_entry_result_data) => {
                //let log_context = "ClientToLib3h::HandleFetchEntryResult";
                self.send_wire_message(WireMessage::ClientToLib3hResponse(
                    ClientToLib3hResponse::FetchEntryResult(fetch_entry_result_data)
                ))
            }
            // Publish data to the dht.
            Lib3hClientProtocol::PublishEntry(provided_entry_data) => {
                //let log_context = "ClientToLib3h::PublishEntry";
                self.send_wire_message(WireMessage::ClientToLib3h(
                    ClientToLib3h::PublishEntry(provided_entry_data)
                ))
            }
            // Request some info / data from a Entry
            Lib3hClientProtocol::QueryEntry(query_entry_data) => {
                // For now, sim2h implements a full-sync mirror DHT
                // which means queries should always be handled locally.
                // Thus, we don't even need to ask the central sim2h instance
                // to handle a query - we just send it back to core directly.
                self.to_core.push(Lib3hServerProtocol::HandleQueryEntry(query_entry_data));
                Ok(())
            }
            // Response to a `HandleQueryEntry` request
            Lib3hClientProtocol::HandleQueryEntryResult(query_entry_result_data) => {
                // See above QueryEntry implementation.
                // All queries are handled locally - we just reflect them back to core:
                self.to_core.push(Lib3hServerProtocol::QueryEntryResult(query_entry_result_data));
                Ok(())
            }

            // -- Entry lists -- //
            Lib3hClientProtocol::HandleGetAuthoringEntryListResult(entry_list_data) => {
                //let log_context = "ClientToLib3h::HandleGetAuthoringEntryListResult";
                self.send_wire_message(WireMessage::Lib3hToClientResponse(
                    Lib3hToClientResponse::HandleGetAuthoringEntryListResult(entry_list_data)
                ))
            }
            Lib3hClientProtocol::HandleGetGossipingEntryListResult(entry_list_data) => {
                //let log_context = "ClientToLib3h::HandleGetGossipingEntryListResult";
                self.send_wire_message(WireMessage::Lib3hToClientResponse(
                    Lib3hToClientResponse::HandleGetGossipingEntryListResult(entry_list_data)
                ))
            }

            // -- N3h specific functinonality -- //
            Lib3hClientProtocol::Shutdown => {
                debug!("Got Lib3hClientProtocol::Shutdown from core in sim2h worker");
                Ok(())
            }
        }
    }
}

impl NetWorker for Sim2hWorker {
    /// We got a message from core
    /// -> forward it to the NetworkEngine
    fn receive(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
        debug!("CORE>>>NET {:?}", data);
        self.inbox.push(data);
        Ok(())
    }

    /// Check for messages from our NetworkEngine
    fn tick(&mut self) -> NetResult<bool> {
        self.num_ticks += 1;
        //if self.num_ticks % 100 == 0 {
        //    io::stdout().flush()?;
        //}
        let mut did_something = false;


        let messages = self.inbox.drain(..).collect::<Vec<_>>();
        for data in messages {
            debug!("CORE >> Sim2h: {:?}", data);
            if let Err(error) = self.handle_client_message(data.clone()) {
                warn!("Error handling client message in Sim2hWorker: {:?}", error);
            }
            did_something = true;
        }
        Ok(did_something)

    }
    /// Set the advertise as worker's endpoint
    fn p2p_endpoint(&self) -> Option<url::Url> {
        None
    }

    /// Set the advertise as worker's endpoint
    fn endpoint(&self) -> Option<String> {
        Some("".into())
    }
}

//#[cfg(feature = "sim2h")]
#[cfg(test)]
mod tests {

    use super::*;
    use lib3h_protocol::{
         protocol_server::Lib3hServerProtocol,
    };
    //use url::Url;

    #[allow(dead_code)]
    fn test_worker() -> (
        Sim2hWorker,
        crossbeam_channel::Receiver<NetResult<Lib3hServerProtocol>>,
    ) {
        let (s, r) = crossbeam_channel::unbounded();
        let handler = NetHandler::new(Box::new(move |message| {
            s.send(message).map_err(|e| e.into())
        }));
        (
            Sim2hWorker::new(
                handler,
                Sim2hConfig {
                    sim2h_url: "http://localhost:8000".into(),
                },
            )
            .unwrap(),
            r,
        )
    }
}
