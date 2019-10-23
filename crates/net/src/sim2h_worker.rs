//! provides worker that makes use of sim2h

use crate::connection::{
    net_connection::{NetHandler, NetWorker},
    NetResult,
};
use holochain_conductor_lib_api::{ConductorApi, CryptoMethod};
use holochain_json_api::{
    error::JsonError,
    json::{JsonString, RawString},
};

use lib3h_zombie_actor::WorkWasDone;

use failure::_core::time::Duration;
use lib3h_protocol::{
    data_types::{GenericResultData, Opaque, SpaceData, StoreEntryAspectData},
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
    websocket::{
        streams::{ConnectionStatus, StreamEvent, StreamManager},
        tls::{TlsCertificate, TlsConfig},
    },
    WireError, WireMessage,
};
use std::{convert::TryFrom, time::Instant};
use url::Url;

const PING_DURATION_SECS: u64 = 10;

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct Sim2hConfig {
    pub sim2h_url: String,
}

/// removed lifetime parameter because compiler says ghost engine needs lifetime that could live statically
#[allow(non_snake_case, dead_code)]
pub struct Sim2hWorker {
    handler: NetHandler,
    stream_manager: StreamManager<std::net::TcpStream>,
    inbox: Vec<Lib3hClientProtocol>,
    to_core: Vec<Lib3hServerProtocol>,
    stream_events: Vec<StreamEvent>,
    server_url: Lib3hUri,
    space_data: Option<SpaceData>,
    agent_id: Address,
    conductor_api: ConductorApi,
    time_of_last_sent: Instant,
}

fn wire_message_into_escaped_string(message: &WireMessage) -> String {
    match message {
        WireMessage::Ping => String::from("\\\"Ping\\\""),
        WireMessage::Pong => String::from("\\\"Pong\\\""),
        _ => {
            let payload: String = message.clone().into();
            let json_string: JsonString = RawString::from(payload).into();
            let mut string: String = json_string.into();
            string = String::from(string.trim_start_matches("\""));
            string = String::from(string.trim_end_matches("\""));
            string
        }
    }
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
        let mut stream_manager = StreamManager::with_std_tcp_stream(
            TlsConfig::SuppliedCertificate(TlsCertificate::build_from_entropy()),
        );

        // bind to some port:
        // channel for making an async call sync
        debug!("Trying to bind to network...");
        let uri = lib3h_protocol::uri::Builder::with_raw_url("wss://127.0.0.1")
            .unwrap()
            .with_port(0)
            .build();

        let bound_url = stream_manager.bind(&uri)?;

        let mut instance = Self {
            handler,
            stream_manager,
            inbox: Vec::new(),
            to_core: Vec::new(),
            stream_events: Vec::new(),
            server_url: Url::parse(&config.sim2h_url)
                .expect("Sim2h URL can't be parsed")
                .into(),
            space_data: None,
            agent_id,
            conductor_api,
            time_of_last_sent: Instant::now(),
        };

        debug!("Successfully bound to {:?}", bound_url);

        let connection_status =
            match instance.try_connect(std::time::Duration::from_millis(5000))? {
                ConnectionStatus::Ready => "Ready",
                ConnectionStatus::None => "None",
                ConnectionStatus::Initializing => "Initializing",
            };

        debug!("Connection status: {:?}", connection_status);
        Ok(instance)
    }

    fn try_connect(&mut self, timeout: std::time::Duration) -> NetResult<ConnectionStatus> {
        let url: url::Url = self.server_url.clone().into();
        let clock = std::time::SystemTime::now();
        let mut status: NetResult<ConnectionStatus> = Ok(ConnectionStatus::None);
        loop {
            match self.stream_manager.connection_status(&url) {
                ConnectionStatus::Ready => return Ok(ConnectionStatus::Ready),
                ConnectionStatus::None => {
                    let url = self.server_url.clone().into();
                    if let Err(e) = self.stream_manager.connect(&url) {
                        status = Err(e.into());
                    }
                }
                s => {
                    status = Ok(s);
                    let (_did_work, mut events) = self.stream_manager.process()?;
                    self.stream_events.append(&mut events);
                    std::thread::sleep(std::time::Duration::from_millis(10))
                }
            };
            if clock.elapsed().unwrap() > timeout {
                error!("Timed out waiting for connection for url {:?}", url);
                return status;
            }
        }
    }

    fn send_wire_message(&mut self, message: WireMessage) -> NetResult<()> {
        self.time_of_last_sent = Instant::now();
        let payload = wire_message_into_escaped_string(&message);
        let signature = self
            .conductor_api
            .execute(payload.clone(), CryptoMethod::Sign)
            .expect(&format!(
                "Couldn't sign wire message in sim2h worker: {}",
                payload
            ));

        let signed_wire_message = SignedWireMessage::new(
            message,
            Provenance::new(self.agent_id.clone(), signature.into()),
        );
        let to_send: Opaque = signed_wire_message.into();
        self.stream_manager.send(
            &self.server_url.clone().into(),
            to_send.as_bytes().as_slice(),
        )?;
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
                self.send_wire_message(WireMessage::Lib3hToClientResponse(
                    Lib3hToClientResponse::HandleFetchEntryResult(fetch_entry_result_data),
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
        if Instant::now().duration_since(self.time_of_last_sent)
            > Duration::from_secs(PING_DURATION_SECS)
        {
            self.send_ping();
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
                error!(
                    "Error handling server message in core's handler: {:?}",
                    error
                );
            }
            did_something = WorkWasDone::from(true);
        }

        let (_did_work, mut events) = match self.stream_manager.process() {
            Ok((did_work, events)) => (did_work, events),
            Err(e) => {
                error!("Transport error: {:?}", e);
                // This most likely means we have connection issues.
                // Send ping to reestablish a potentially lost connection.
                self.send_ping();
                (false.into(), vec![])
            }
        };
        self.stream_events.append(&mut events);
        for transport_message in self.stream_events.drain(..).collect::<Vec<StreamEvent>>() {
            match transport_message {
                StreamEvent::ReceivedData(uri, payload) => {
                    let uri : Lib3hUri = uri.into();
                    if uri != self.server_url {
                        warn!("Received data from unknown remote {:?} - ignoring", uri);
                    } else {
                        let payload : Opaque = payload.into();
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
                StreamEvent::IncomingConnectionEstablished(uri) =>
                    warn!("Got incoming connection from {:?} in Sim2hWorker - This should not happen and is ignored.", uri),
                StreamEvent::ErrorOccured(uri, error) =>
                    error!("Transport error occurred on connection to {:?}: {:?}", uri, error),
                StreamEvent::ConnectionClosed(_) => warn!("Got connection close! Will try to reconnect."),
                StreamEvent::ConnectResult(url, net_id) => {
                    info!("got connect result for url: {:?}, net_id: {:?}", url, net_id)
                }
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
