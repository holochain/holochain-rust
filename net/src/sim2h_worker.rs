//! provides worker that makes use of sim2h

use crate::connection::{
    net_connection::{NetHandler, NetWorker},
    NetResult,
};
use holochain_json_api::{error::JsonError, json::JsonString};
use lib3h_protocol::{protocol_client::Lib3hClientProtocol, protocol_server::Lib3hServerProtocol, Address};
use log::*;
use sim2h::{WireMessage, WireError};
use url::Url;
use lib3h::transport::websocket::actor::{GhostTransportWebsocket};
use lib3h::transport::websocket::tls::{TlsConfig, TlsCertificate};
use lib3h::transport::protocol::{
    RequestToChild, RequestToChildResponse, RequestToParent, TransportActorParentWrapper
};
use holochain_tracing::Span;
use lib3h_zombie_actor::{GhostParentWrapper, GhostCallbackData, WorkWasDone};
use lib3h_zombie_actor::GhostCanTrack;
use failure::err_msg;
use lib3h_protocol::protocol::*;
use lib3h_protocol::data_types::{GenericResultData, Opaque, SpaceData, StoreEntryAspectData};
use lib3h_protocol::uri::Lib3hUri;
use std::convert::TryFrom;
use detach::Detach;
use sim2h::crypto::{Provenance, SignedWireMessage};
use holochain_conductor_api_api::{CryptoMethod, ConductorApi};
use holochain_json_api::json::RawString;

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct Sim2hConfig {
    pub sim2h_url: String,
}

/// removed lifetime parameter because compiler says ghost engine needs lifetime that could live statically
#[allow(non_snake_case, dead_code)]
pub struct Sim2hWorker {
    handler: NetHandler,
    transport: Detach<TransportActorParentWrapper<Sim2hWorker, GhostTransportWebsocket>>,
    inbox: Vec<Lib3hClientProtocol>,
    to_core: Vec<Lib3hServerProtocol>,
    server_url: Lib3hUri,
    space_data: Option<SpaceData>,
    agent_id: Address,
    conductor_api: ConductorApi,
}

fn wire_message_into_escaped_string(message: &WireMessage) -> String {
    let payload: String = message.clone().into();
    let json_string: JsonString = RawString::from(payload).into();
    let mut string: String = json_string.into();
    string = String::from(string.trim_start_matches("\""));
    string = String::from( string.trim_end_matches("\""));
    string
}

impl Sim2hWorker {
    pub fn advertise(self) -> url::Url {
        Url::parse("ws://example.com").unwrap()
    }

    /// Create a new worker connected to the sim2h instance
    pub fn new(
        handler: NetHandler,
        config: Sim2hConfig,
        agent_id: Address,
        conductor_api: ConductorApi,
    ) -> NetResult<Self> {
        let transport_raw = GhostTransportWebsocket::new(
            // not used currently inside GhostTransportWebsocket:
            Address::from("sim2h-worker-transport"),
            TlsConfig::SuppliedCertificate(TlsCertificate::build_from_entropy()),
            // not used currently inside GhostTransportWebsocket:\
            Address::from("sim2h-network"),
        );

        let mut transport: TransportActorParentWrapper<Sim2hWorker, GhostTransportWebsocket> =
            GhostParentWrapper::new(
                transport_raw,
                "t1_requests", // prefix for request ids in the tracker
            );


        // bind to some port:
        // channel for making an async call sync
        debug!("Trying to bind to network...");
        let (tx, rx) = crossbeam_channel::unbounded();
        transport.request(
            Span::todo("Find out how to use spans the right way"),
            RequestToChild::Bind {
                spec: Url::parse("wss://127.0.0.1:0").expect("can parse url").into(),
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

        let mut instance = Self {
            handler,
            transport: Detach::new(transport),
            inbox: Vec::new(),
            to_core: Vec::new(),
            server_url: Url::parse(&config.sim2h_url)
                .expect("Sim2h URL can't be parsed")
                .into(),
            space_data: None,
            agent_id,
            conductor_api,
        };

        detach_run!(&mut instance.transport, |t| t.process(&mut instance))?;
        detach_run!(&mut instance.transport, |t| t.process(&mut instance))?;
        detach_run!(&mut instance.transport, |t| t.process(&mut instance))?;

        let result = rx.recv()?;
        let bound_url = result.map_err(|bind_error| err_msg(bind_error))?;
        debug!("Successfully bound to {:?}", bound_url);

        Ok(instance)
    }

    fn send_wire_message(&mut self, message: WireMessage) -> NetResult<()> {
        let signature = self.conductor_api
            .execute(wire_message_into_escaped_string(&message), CryptoMethod::Sign)
            .expect("Couldn't sign wire message in sim2h worker");

        let signed_wire_message = SignedWireMessage::new(
            message,
            Provenance::new(self.agent_id.clone(), signature.into()),
        );
        self.transport.request(
            Span::todo("Find out how to use spans the right way"),
            RequestToChild::SendMessage {
                uri: self.server_url.clone(),
                payload: signed_wire_message.into(),
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
                self.space_data = Some(space_data.clone());
                self.send_wire_message(WireMessage::ClientToLib3h(
                    ClientToLib3h::JoinSpace(space_data)
                ))
            }
            // Order the p2p module to leave the network of the specified space.
            Lib3hClientProtocol::LeaveSpace(space_data) => {
                //error!("Leave space not implemented for sim2h yet");
                self.send_wire_message(WireMessage::ClientToLib3h(
                    ClientToLib3h::LeaveSpace(space_data)
                ))
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
                self.send_wire_message(WireMessage::Lib3hToClientResponse(
                    Lib3hToClientResponse::HandleSendDirectMessageResult(dm_data)
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
                self.send_wire_message(WireMessage::Lib3hToClientResponse(
                    Lib3hToClientResponse::HandleFetchEntryResult(fetch_entry_result_data)
                ))
            }
            // Publish data to the dht.
            Lib3hClientProtocol::PublishEntry(provided_entry_data) => {
                //let log_context = "ClientToLib3h::PublishEntry";

                // As with QueryEntry, we assume a mirror DHT being implemented by Sim2h.
                // This means that we can play back PublishEntry messages already locally
                // as HandleStoreEntryAspects.
                // This makes instances with Sim2hWorker work even if offline,
                // i.e. not connected to the sim2h node.
                for aspect in &provided_entry_data.entry.aspect_list {
                    self.to_core.push(Lib3hServerProtocol::HandleStoreEntryAspect(
                        StoreEntryAspectData {
                            request_id: "".into(),
                            space_address: provided_entry_data.space_address.clone(),
                            provider_agent_id: provided_entry_data.provider_agent_id.clone(),
                            entry_address: provided_entry_data.entry.entry_address.clone(),
                            entry_aspect: aspect.clone(),
                        })
                    );
                }
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

    fn handle_server_message(&mut self, message: WireMessage) -> NetResult<()> {
        match message {
            WireMessage::Ping => self.send_wire_message(WireMessage::Pong)?,
            WireMessage::Pong => {},
            WireMessage::Lib3hToClient(m) =>
                self.to_core.push(Lib3hServerProtocol::from(m)),
            WireMessage::ClientToLib3hResponse(m) =>
                self.to_core.push(Lib3hServerProtocol::from(m)),
            WireMessage::Lib3hToClientResponse(m) =>
                error!("Got a Lib3hToClientResponse from the Sim2h server, weird! Ignoring: {:?}", m),
            WireMessage::ClientToLib3h(m) =>
                error!("Got a ClientToLib3h from the Sim2h server, weird! Ignoring: {:?}", m),
            WireMessage::Err(sim2h_error) => match sim2h_error {
                WireError::MessageWhileInLimbo => if let Some(space_data) = self.space_data.clone() {
                    self.send_wire_message(WireMessage::ClientToLib3h(
                        ClientToLib3h::JoinSpace(space_data)
                    ))?;
                } else {
                    error!("Uh oh, we got a MessageWhileInLimbo errro and we don't have space data. Did core send a message before sending a join? This should not happen.");
                }
                WireError::Other(e) => error!("Got error from Sim2h server: {:?}", e),
            }
        };
        Ok(())
    }
}

impl NetWorker for Sim2hWorker {
    /// We got a message from core
    /// -> forward it to the NetworkEngine
    fn receive(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
        self.inbox.push(data);
        Ok(())
    }

    /// Check for messages from our NetworkEngine
    fn tick(&mut self) -> NetResult<bool> {
        if let Err(transport_error) = detach_run!(&mut self.transport, |t| t.process(self)) {
            error!("Transport error: {:?}", transport_error);
            // This most likely means we have connection issues.
            // Send ping to reestablish a potentially lost connection.
            if let Err(e) = self.send_wire_message(WireMessage::Ping) {
                debug!("send ping failure: {:?}", e);
            }
        }
        let mut did_something = WorkWasDone::from(false);

        let client_messages = self.inbox.drain(..).collect::<Vec<_>>();
        for data in client_messages {
            debug!("CORE >> Sim2h: {:?}", data);
            if let Err(error) = self.handle_client_message(data) {
                error!("Error handling client message in Sim2hWorker: {:?}", error);
            }
            did_something = WorkWasDone::from(true);
        }

        let server_messages = self.to_core.drain(..).collect::<Vec<_>>();
        for data in server_messages {
            debug!("Sim2h >> CORE: {:?}", data);
            if let Err(error) = self.handler.handle(Ok(data)) {
                error!("Error handling server message in core's handler: {:?}", error);
            }
            did_something = WorkWasDone::from(true);
        }

        for mut transport_message in self.transport.drain_messages() {
            match transport_message.take_message().expect("GhostMessage must have a message") {
                RequestToParent::ReceivedData {uri, payload} => {
                    if uri != self.server_url {
                        warn!("Received data from unknown remote {:?} - ignoring", uri);
                    } else {
                        match WireMessage::try_from(&payload) {
                            Ok(wire_message) =>
                                if let Err(error) = self.handle_server_message(wire_message) {
                                    error!("Error handling server message in Sim2hWorker: {:?}", error);
                                },
                            Err(error) =>
                                error!(
                                    "Could not deserialize received payload into WireMessage!\nError: {:?}\nPayload was: {:?}",
                                    error,
                                    payload
                                )
                        }


                    }
                }
                RequestToParent::IncomingConnection {uri} =>
                    warn!("Got incoming connection from {:?} in Sim2hWorker - This should not happen and is ignored.", uri),
                RequestToParent::ErrorOccured {uri, error} =>
                    error!("Transport error occurred on connection to {:?}: {:?}", uri, error),
                RequestToParent::Disconnect(_) => warn!("Got disconnected! Will try to reconnect."),
                RequestToParent::Unbind(url) => error!("Got unbound form: {:?}", url),
            }
            did_something = WorkWasDone::from(true);
        }
        Ok(did_something.into())

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
