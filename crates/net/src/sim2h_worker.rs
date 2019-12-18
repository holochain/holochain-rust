//! provides worker that makes use of sim2h

use crate::connection::{
    net_connection::{NetHandler, NetWorker},
    NetResult,
};
use failure::_core::time::Duration;
use holochain_conductor_lib_api::{ConductorApi, CryptoMethod};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_metrics::{DefaultMetricPublisher, MetricPublisher};
use in_stream::*;
use lib3h_protocol::{
    data_types::{FetchEntryData, GenericResultData, Opaque, SpaceData, StoreEntryAspectData},
    protocol::*,
    protocol_client::Lib3hClientProtocol,
    protocol_server::Lib3hServerProtocol,
    types::{AgentPubKey, SpaceHash},
    uri::Lib3hUri,
    Address,
};
use log::*;
use sim2h::{
    crypto::{Provenance, SignedWireMessage},
    WireError, WireMessage,
};
use std::{convert::TryFrom, time::Instant};
use url::Url;
use url2::prelude::*;

const RECONNECT_INTERVAL: Duration = Duration::from_secs(1);
const SIM2H_WORKER_INTERNAL_REQUEST_ID: &str = "SIM2H_WORKER";

fn connect(url: Lib3hUri) -> NetResult<InStreamWss<InStreamTls<InStreamTcp>>> {
    let config = WssConnectConfig::new(TlsConnectConfig::new(TcpConnectConfig::default()));
    Ok(InStreamWss::connect(&url::Url::from(url).into(), config)?)
}

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct Sim2hConfig {
    pub sim2h_url: String,
}

/// removed lifetime parameter because compiler says ghost engine needs lifetime that could live statically
#[allow(non_snake_case, dead_code)]
pub struct Sim2hWorker {
    handler: NetHandler,
    connection: Option<InStreamWss<InStreamTls<InStreamTcp>>>,
    inbox: Vec<Lib3hClientProtocol>,
    to_core: Vec<Lib3hServerProtocol>,
    server_url: Lib3hUri,
    space_data: Option<SpaceData>,
    agent_id: Address,
    conductor_api: ConductorApi,
    time_of_last_connection_attempt: Instant,
    metric_publisher: std::sync::Arc<std::sync::RwLock<dyn MetricPublisher>>,
    outgoing_message_buffer: Vec<WireMessage>,
    ws_frame: Option<WsFrame>,
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
        let mut instance = Self {
            handler,
            connection: None,
            inbox: Vec::new(),
            to_core: Vec::new(),
            server_url: url::Url::from(url2!("{}", config.sim2h_url)).into(),
            space_data: None,
            agent_id,
            conductor_api,
            time_of_last_connection_attempt: Instant::now()
                .checked_sub(RECONNECT_INTERVAL)
                .unwrap(),
            metric_publisher: std::sync::Arc::new(std::sync::RwLock::new(
                DefaultMetricPublisher::default(),
            )),
            outgoing_message_buffer: Vec::new(),
            ws_frame: None,
        };

        instance.check_reconnect();

        Ok(instance)
    }

    /// check to see if we need to re-connect
    /// if we don't have a ready connection within RECONNECT_INTERVAL
    fn check_reconnect(&mut self) {
        if let Some(c) = &self.connection {
            if c.is_ready() {
                return;
            }
        }

        if self.time_of_last_connection_attempt.elapsed() < RECONNECT_INTERVAL {
            return;
        }

        self.time_of_last_connection_attempt = Instant::now();
        self.connection = None;
        if let Ok(connection) = connect(self.server_url.clone()) {
            self.connection = Some(connection);
        }
    }

    fn connection_ready(&self) -> bool {
        match &self.connection {
            Some(c) if c.is_ready() => true,
            _ => false,
        }
    }

    /// if we have queued wire messages and our connection is ready,
    /// try to send them
    fn try_send_from_outgoing_buffer(&mut self) {
        loop {
            if self.outgoing_message_buffer.is_empty() || !self.connection_ready() {
                return;
            }
            let message = self.outgoing_message_buffer.get(0).unwrap();
            let payload: String = message.clone().into();
            let signature = self
                .conductor_api
                .execute(payload.clone(), CryptoMethod::Sign)
                .unwrap_or_else(|e| {
                    panic!(
                        "Couldn't sign wire message in sim2h worker: payload={}, error={:?}",
                        payload, e
                    )
                });
            let signed_wire_message = SignedWireMessage::new(
                message.clone(),
                Provenance::new(self.agent_id.clone(), signature.into()),
            );
            let to_send: Opaque = signed_wire_message.into();
            // safe to unwrap because we check connection_ready() above
            if let Err(e) = self
                .connection
                .as_mut()
                .unwrap()
                .write(to_send.to_vec().into())
            {
                error!(
                    "TransportError trying to send message to sim2h server: {:?}",
                    e
                );
                self.connection = None;
                self.check_reconnect();
                return;
            }
            // if we made it here, we successfully sent the first message
            // we can remove it from the outgoing buffer queue
            self.outgoing_message_buffer.remove(0);
        }
    }

    /// queue a wire message for send
    fn send_wire_message(&mut self, message: WireMessage) -> NetResult<()> {
        // we always put messages in the outgoing buffer,
        // they'll be sent when the connection is ready
        self.outgoing_message_buffer.push(message);
        Ok(())
    }

    #[allow(dead_code)]
    fn handle_client_message(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
        match data {
            // Success response to a request (any Command with an `request_id` field.)
            Lib3hClientProtocol::SuccessResult(generic_result_data) => {
                self.to_core
                    .push(Lib3hServerProtocol::FailureResult(generic_result_data));
                Ok(())
            }
            // Connect to the specified multiaddr
            Lib3hClientProtocol::Connect(connect_data) => {
                self.to_core
                    .push(Lib3hServerProtocol::FailureResult(GenericResultData {
                        request_id: connect_data.request_id,
                        space_address: SpaceHash::default().into(),
                        to_agent_id: AgentPubKey::default(),
                        result_info: Opaque::new(),
                    }));
                Ok(())
            }

            // -- Space -- //
            // Order the p2p module to be part of the network of the specified space.
            Lib3hClientProtocol::JoinSpace(space_data) => {
                //let log_context = "ClientToLib3h::JoinSpace";
                self.space_data = Some(space_data.clone());
                self.send_wire_message(WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(
                    space_data,
                )))
            }
            // Order the p2p module to leave the network of the specified space.
            Lib3hClientProtocol::LeaveSpace(space_data) => {
                //error!("Leave space not implemented for sim2h yet");
                self.send_wire_message(WireMessage::ClientToLib3h(ClientToLib3h::LeaveSpace(
                    space_data,
                )))
            }

            // -- Direct Messaging -- //
            // Send a message directly to another agent on the network
            Lib3hClientProtocol::SendDirectMessage(dm_data) => {
                //let log_context = "ClientToLib3h::SendDirectMessage";
                self.send_wire_message(WireMessage::ClientToLib3h(
                    ClientToLib3h::SendDirectMessage(dm_data),
                ))
            }
            // Our response to a direct message from another agent.
            Lib3hClientProtocol::HandleSendDirectMessageResult(dm_data) => {
                //let log_context = "ClientToLib3h::HandleSendDirectMessageResult";
                self.send_wire_message(WireMessage::Lib3hToClientResponse(
                    Lib3hToClientResponse::HandleSendDirectMessageResult(dm_data),
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
                if fetch_entry_result_data.request_id == SIM2H_WORKER_INTERNAL_REQUEST_ID {
                    for aspect in fetch_entry_result_data.entry.aspect_list {
                        self.to_core
                            .push(Lib3hServerProtocol::HandleStoreEntryAspect(
                                StoreEntryAspectData {
                                    request_id: "".into(),
                                    space_address: fetch_entry_result_data.space_address.clone(),
                                    provider_agent_id: fetch_entry_result_data
                                        .provider_agent_id
                                        .clone(),
                                    entry_address: fetch_entry_result_data
                                        .entry
                                        .entry_address
                                        .clone(),
                                    entry_aspect: aspect,
                                },
                            ));
                    }
                    Ok(())
                } else {
                    self.send_wire_message(WireMessage::Lib3hToClientResponse(
                        Lib3hToClientResponse::HandleFetchEntryResult(fetch_entry_result_data),
                    ))
                }
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
                    self.to_core
                        .push(Lib3hServerProtocol::HandleStoreEntryAspect(
                            StoreEntryAspectData {
                                request_id: "".into(),
                                space_address: provided_entry_data.space_address.clone(),
                                provider_agent_id: provided_entry_data.provider_agent_id.clone(),
                                entry_address: provided_entry_data.entry.entry_address.clone(),
                                entry_aspect: aspect.clone(),
                            },
                        ));
                }
                self.send_wire_message(WireMessage::ClientToLib3h(ClientToLib3h::PublishEntry(
                    provided_entry_data,
                )))
            }
            // Request some info / data from a Entry
            Lib3hClientProtocol::QueryEntry(query_entry_data) => {
                // For now, sim2h implements a full-sync mirror DHT
                // which means queries should always be handled locally.
                // Thus, we don't even need to ask the central sim2h instance
                // to handle a query - we just send it back to core directly.
                self.to_core
                    .push(Lib3hServerProtocol::HandleQueryEntry(query_entry_data));
                Ok(())
            }
            // Response to a `HandleQueryEntry` request
            Lib3hClientProtocol::HandleQueryEntryResult(query_entry_result_data) => {
                // See above QueryEntry implementation.
                // All queries are handled locally - we just reflect them back to core:
                self.to_core.push(Lib3hServerProtocol::QueryEntryResult(
                    query_entry_result_data,
                ));
                Ok(())
            }

            // -- Entry lists -- //
            Lib3hClientProtocol::HandleGetAuthoringEntryListResult(entry_list_data) => {
                //let log_context = "ClientToLib3h::HandleGetAuthoringEntryListResult";
                for (entry_hash, aspect_hashes) in &entry_list_data.address_map {
                    self.to_core
                        .push(Lib3hServerProtocol::HandleFetchEntry(FetchEntryData {
                            space_address: entry_list_data.space_address.clone(),
                            entry_address: entry_hash.clone(),
                            request_id: SIM2H_WORKER_INTERNAL_REQUEST_ID.to_string(),
                            provider_agent_id: entry_list_data.provider_agent_id.clone(),
                            aspect_address_list: Some(aspect_hashes.clone()),
                        }))
                }
                self.send_wire_message(WireMessage::Lib3hToClientResponse(
                    Lib3hToClientResponse::HandleGetAuthoringEntryListResult(entry_list_data),
                ))
            }
            Lib3hClientProtocol::HandleGetGossipingEntryListResult(entry_list_data) => {
                //let log_context = "ClientToLib3h::HandleGetGossipingEntryListResult";
                self.send_wire_message(WireMessage::Lib3hToClientResponse(
                    Lib3hToClientResponse::HandleGetGossipingEntryListResult(entry_list_data),
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
            WireMessage::Pong => {}
            WireMessage::Lib3hToClient(m) => self.to_core.push(Lib3hServerProtocol::from(m)),
            WireMessage::ClientToLib3hResponse(m) => {
                self.to_core.push(Lib3hServerProtocol::from(m))
            }
            WireMessage::Lib3hToClientResponse(m) => error!(
                "Got a Lib3hToClientResponse from the Sim2h server, weird! Ignoring: {:?}",
                m
            ),
            WireMessage::ClientToLib3h(m) => error!(
                "Got a ClientToLib3h from the Sim2h server, weird! Ignoring: {:?}",
                m
            ),
            WireMessage::Err(sim2h_error) => match sim2h_error {
                WireError::MessageWhileInLimbo => {
                    if let Some(space_data) = self.space_data.clone() {
                        self.send_wire_message(WireMessage::ClientToLib3h(
                            ClientToLib3h::JoinSpace(space_data),
                        ))?;
                    } else {
                        error!("Uh oh, we got a MessageWhileInLimbo errro and we don't have space data. Did core send a message before sending a join? This should not happen.");
                    }
                }
                WireError::Other(e) => error!("Got error from Sim2h server: {:?}", e),
            },
        };
        Ok(())
    }

    #[allow(dead_code)]
    fn send_ping(&mut self) {
        trace!("Ping");
        if let Err(e) = self.send_wire_message(WireMessage::Ping) {
            debug!("Ping failed with: {:?}", e);
        }
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
        let clock = std::time::SystemTime::now();

        let mut did_something = false;

        if self.ws_frame.is_none() {
            self.ws_frame = Some(WsFrame::default());
        }

        if self.connection_ready() {
            self.try_send_from_outgoing_buffer();
            // safe to unwrap because we check connection_ready()
            match self
                .connection
                .as_mut()
                .unwrap()
                .read(&mut self.ws_frame.as_mut().unwrap())
            {
                Ok(_) => {
                    did_something = true;
                    let frame = self.ws_frame.take().unwrap();
                    if let WsFrame::Binary(payload) = frame {
                        let payload: Opaque = payload.into();
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
                    } else {
                        trace!("unhandled websocket message type: {:?}", frame);
                    }
                }
                Err(e) if e.would_block() => (),
                Err(e) => {
                    error!(
                        "TransportError trying to read message from sim2h server: {:?}",
                        e
                    );
                    self.connection = None;
                    self.check_reconnect();
                }
            }
        } else {
            self.check_reconnect();
        }

        let client_messages = self.inbox.drain(..).collect::<Vec<_>>();
        for data in client_messages {
            debug!("CORE >> Sim2h: {:?}", data);
            // outgoing messages triggered by `self.hand_client_message` that fail because of
            // connection status, will automatically be re-sent via `self.outgoing_message_buffer`
            if let Err(error) = self.handle_client_message(data) {
                error!("Error handling client message in Sim2hWorker: {:?}", error);
            }
            did_something = true;
        }

        let server_messages = self.to_core.drain(..).collect::<Vec<_>>();
        for data in server_messages {
            debug!("Sim2h >> CORE: {:?}", data);
            if let Err(error) = self.handler.handle(Ok(data)) {
                error!(
                    "Error handling server message in core's handler: {:?}",
                    error
                );
            }
            did_something = true;
        }

        if did_something {
            let latency = clock.elapsed().unwrap().as_millis();
            let metric_name = "sim2h_worker.tick.latency";
            let metric = holochain_metrics::Metric::new(metric_name, latency as f64);
            trace!("publishing: {}", latency);
            self.metric_publisher.write().unwrap().publish(&metric);
        }
        Ok(did_something)
    }

    /// Set the advertise as worker's endpoint
    fn p2p_endpoint(&self) -> Option<url::Url> {
        Some(self.server_url.clone().into())
    }

    /// Set the advertise as worker's endpoint
    fn endpoint(&self) -> Option<String> {
        Some("".into())
    }
}
