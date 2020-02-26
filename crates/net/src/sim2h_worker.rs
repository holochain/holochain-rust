//! provides worker that makes use of sim2h

use crate::{
    connection::{
        net_connection::{NetHandler, NetWorker},
        NetResult,
    },
    p2p_network::Lib3hClientProtocolWrapped,
    NEW_RELIC_LICENSE_KEY,
};
use failure::_core::time::Duration;
use holochain_conductor_lib_api::{ConductorApi, CryptoMethod};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_metrics::{DefaultMetricPublisher, MetricPublisher};
use in_stream::*;
use lib3h_protocol::{
    data_types::{
        EntryListData, FetchEntryData, GenericResultData, Opaque, SpaceData, StoreEntryAspectData,
    },
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
    TcpWss, WireError, WireMessage, RECEIPT_HASH_SEED, WIRE_VERSION,
};
use std::{
    convert::TryFrom,
    hash::{Hash, Hasher},
    time::Instant,
};

use twox_hash::XxHash64;
use url::Url;
use url2::prelude::*;

const INITIAL_CONNECTION_TIMEOUT_MS: u64 = 2000; // The real initial is 4 seconds because one backoff happens to start
const MAX_CONNECTION_TIMEOUT_MS: u64 = 60000;
const SIM2H_WORKER_INTERNAL_REQUEST_ID: &str = "SIM2H_WORKER";
const RESEND_WIRE_MESSAGE_MS: u64 = 10000;

fn connect(url: Lib3hUri, timeout_ms: u64) -> NetResult<TcpWss> {
    //    let config = WssConnectConfig::new(TlsConnectConfig::new(TcpConnectConfig::default()));
    let config = WssConnectConfig::new(TcpConnectConfig {
        connect_timeout_ms: Some(timeout_ms),
    });
    Ok(InStreamWss::connect(&url::Url::from(url).into(), config)?)
}

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct Sim2hConfig {
    pub sim2h_url: String,
}

struct BufferedMessage {
    pub wire_message: WireMessage,
    pub hash: u64,
    pub last_sent: Option<Instant>,
}

impl From<WireMessage> for BufferedMessage {
    fn from(wire_message: WireMessage) -> BufferedMessage {
        BufferedMessage {
            wire_message,
            hash: 0,
            last_sent: None,
        }
    }
}

/// removed lifetime parameter because compiler says ghost engine needs lifetime that could live statically
#[allow(non_snake_case, dead_code)]
pub struct Sim2hWorker {
    handler: NetHandler,
    connection: Option<TcpWss>,
    inbox: Vec<ht::EncodedSpanWrap<Lib3hClientProtocol>>,
    to_core: Vec<ht::EncodedSpanWrap<Lib3hServerProtocol>>,
    server_url: Lib3hUri,
    space_data: Option<SpaceData>,
    agent_id: Address,
    conductor_api: ConductorApi,
    time_of_last_connection_attempt: Instant,
    connection_timeout_backoff: u64,
    reconnect_interval: Duration,
    metric_publisher: std::sync::Arc<std::sync::RwLock<dyn MetricPublisher>>,
    outgoing_message_buffer: Vec<BufferedMessage>,
    ws_frame: Option<WsFrame>,
    initial_authoring_list: Option<EntryListData>,
    initial_gossiping_list: Option<EntryListData>,
    has_self_stored_authored_aspects: bool,
    is_full_sync_DHT: bool,
    tracer: Option<ht::Tracer>,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_NET)]
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
        tracer: Option<ht::Tracer>,
    ) -> NetResult<Self> {
        let reconnect_interval = Duration::from_millis(INITIAL_CONNECTION_TIMEOUT_MS);
        let mut instance = Self {
            handler,
            connection: None,
            inbox: Vec::new(),
            to_core: Vec::new(),
            server_url: url::Url::from(url2!("{}", config.sim2h_url)).into(),
            space_data: None,
            agent_id,
            conductor_api,
            connection_timeout_backoff: INITIAL_CONNECTION_TIMEOUT_MS,
            reconnect_interval,
            time_of_last_connection_attempt: Instant::now()
                .checked_sub(reconnect_interval)
                .unwrap(),
            metric_publisher: std::sync::Arc::new(std::sync::RwLock::new(
                DefaultMetricPublisher::default(),
            )),
            outgoing_message_buffer: Vec::new(),
            ws_frame: None,
            initial_authoring_list: None,
            initial_gossiping_list: None,
            has_self_stored_authored_aspects: false,
            is_full_sync_DHT: false,
            tracer,
        };

        instance.send_wire_message(WireMessage::Hello(WIRE_VERSION))?;
        instance.check_reconnect();
        Ok(instance)
    }

    fn backoff(&mut self) {
        let new_backoff = std::cmp::max(
            MAX_CONNECTION_TIMEOUT_MS,
            self.connection_timeout_backoff * 2,
        );
        if self.connection_timeout_backoff != new_backoff {
            self.inner_set_backoff(self.connection_timeout_backoff * 2);
        }
    }

    fn inner_set_backoff(&mut self, backoff: u64) {
        self.connection_timeout_backoff = backoff;
        debug!(
            "BACKOFF setting reconnect interval to {}",
            self.connection_timeout_backoff
        );
        self.reconnect_interval = Duration::from_millis(self.connection_timeout_backoff)
    }

    fn reset_backoff(&mut self) {
        if self.connection_timeout_backoff > INITIAL_CONNECTION_TIMEOUT_MS {
            self.inner_set_backoff(INITIAL_CONNECTION_TIMEOUT_MS);
        }
    }

    /// check to see if we need to re-connect
    /// if we don't have a ready connection within reconnect_interval
    fn check_reconnect(&mut self) {
        if self.connection_ready() {
            self.reset_backoff();
            return;
        }

        if self.time_of_last_connection_attempt.elapsed() < self.reconnect_interval {
            return;
        }

        //if self.connection.is_none() {
        warn!(
            "BACKOFF attempting reconnect, connection state: {:?}",
            self.connection
        );
        //}

        self.backoff();

        self.time_of_last_connection_attempt = Instant::now();
        self.connection = None;
        if let Ok(connection) = connect(self.server_url.clone(), self.connection_timeout_backoff) {
            let mut span: ht::Span = self
                .tracer
                .clone()
                .unwrap_or_else(|| ht::null_tracer())
                .span(format!("Sending Join {}:{}", file!(), line!()))
                .start()
                .into();
            self.connection = Some(connection);
            let msg = match &self.space_data {
                None => return,
                Some(space_data) => {
                    span.event(format!("Space Data {:?}", &space_data));
                    WireMessage::ClientToLib3h(
                        span.wrap(ClientToLib3h::JoinSpace(space_data.clone()))
                            .into(),
                    )
                }
            };
            debug!("SENDING JOIN {:#?}", msg);
            self.prepend_wire_message(msg)
                .expect("can send JoinSpace on reconnect");
        }
    }

    fn connection_ready(&mut self) -> bool {
        match &mut self.connection {
            Some(c) => match c.check_ready() {
                Ok(true) => true,
                Ok(false) => false,
                Err(e) => {
                    error!("connection handshake error: {:?}", e);
                    self.connection = None;
                    false
                }
            },
            _ => false,
        }
    }

    /// if we have queued wire messages and our connection is ready,
    /// try to send one
    /// return if we did something
    fn try_send_from_outgoing_buffer(&mut self) -> bool {
        if self.outgoing_message_buffer.is_empty() || !self.connection_ready() {
            return false;
        }
        let buffered_message = self.outgoing_message_buffer.get_mut(0).unwrap();
        if let Some(instant_last_sent) = buffered_message.last_sent {
            if instant_last_sent.elapsed() < Duration::from_millis(RESEND_WIRE_MESSAGE_MS) {
                return false;
            }
        }
        let message = &buffered_message.wire_message;
        debug!("WireMessage: preparing to send {:?}", message);
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
        let payload: Opaque = payload.into();
        let signed_wire_message = SignedWireMessage::new(
            payload.clone(),
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
            return true;
        }
        let mut hasher = XxHash64::with_seed(RECEIPT_HASH_SEED);
        payload.hash(&mut hasher);
        buffered_message.hash = hasher.finish();
        buffered_message.last_sent = Some(Instant::now());
        true
    }

    /// if we re-connected, we may need to send a join first,
    /// before other queued messages
    fn prepend_wire_message(&mut self, message: WireMessage) -> NetResult<()> {
        debug!("WireMessage: queueing {:?}", message);
        for buffered_message in self.outgoing_message_buffer.iter_mut() {
            buffered_message.last_sent = None;
        }
        self.outgoing_message_buffer.insert(0, message.into());
        Ok(())
    }

    /// queue a wire message for send
    fn send_wire_message(&mut self, message: WireMessage) -> NetResult<()> {
        // we always put messages in the outgoing buffer,
        // they'll be sent when the connection is ready
        debug!("WireMessage: queueing {:?}", message);
        self.outgoing_message_buffer.push(message.into());
        Ok(())
    }

    #[allow(dead_code)]
    fn handle_client_message(&mut self, span_wrap: Lib3hClientProtocolWrapped) -> NetResult<()> {
        match span_wrap.data.clone() {
            // Success response to a request (any Command with an `request_id` field.)
            Lib3hClientProtocol::SuccessResult(generic_result_data) => {
                self.to_core.push(
                    span_wrap.swapped(Lib3hServerProtocol::FailureResult(generic_result_data)),
                );
                Ok(())
            }
            // Connect to the specified multiaddr
            Lib3hClientProtocol::Connect(connect_data) => {
                let msg = Lib3hServerProtocol::FailureResult(GenericResultData {
                    request_id: connect_data.request_id,
                    space_address: SpaceHash::default().into(),
                    to_agent_id: AgentPubKey::default(),
                    result_info: Opaque::new(),
                });
                self.to_core.push(span_wrap.swapped(msg));
                Ok(())
            }

            // -- Space -- //
            // Order the p2p module to be part of the network of the specified space.
            Lib3hClientProtocol::JoinSpace(space_data) => {
                //let log_context = "ClientToLib3h::JoinSpace";
                self.space_data = Some(space_data.clone());
                self.send_wire_message(WireMessage::ClientToLib3h(
                    span_wrap.swapped(ClientToLib3h::JoinSpace(space_data)),
                ))
            }
            // Order the p2p module to leave the network of the specified space.
            Lib3hClientProtocol::LeaveSpace(space_data) => {
                //error!("Leave space not implemented for sim2h yet");
                self.send_wire_message(WireMessage::ClientToLib3h(
                    span_wrap.swapped(ClientToLib3h::LeaveSpace(space_data)),
                ))
            }

            // -- Direct Messaging -- //
            // Send a message directly to another agent on the network
            Lib3hClientProtocol::SendDirectMessage(dm_data) => {
                //let log_context = "ClientToLib3h::SendDirectMessage";
                self.send_wire_message(WireMessage::ClientToLib3h(
                    span_wrap.swapped(ClientToLib3h::SendDirectMessage(dm_data)),
                ))
            }
            // Our response to a direct message from another agent.
            Lib3hClientProtocol::HandleSendDirectMessageResult(dm_data) => {
                //let log_context = "ClientToLib3h::HandleSendDirectMessageResult";
                self.send_wire_message(WireMessage::Lib3hToClientResponse(span_wrap.swapped(
                    Lib3hToClientResponse::HandleSendDirectMessageResult(dm_data),
                )))
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
                        let msg =
                            Lib3hServerProtocol::HandleStoreEntryAspect(StoreEntryAspectData {
                                request_id: "".into(),
                                space_address: fetch_entry_result_data.space_address.clone(),
                                provider_agent_id: fetch_entry_result_data
                                    .provider_agent_id
                                    .clone(),
                                entry_address: fetch_entry_result_data.entry.entry_address.clone(),
                                entry_aspect: aspect,
                            });
                        self.to_core.push(span_wrap.swapped(msg));
                    }
                    Ok(())
                } else {
                    self.send_wire_message(WireMessage::Lib3hToClientResponse(span_wrap.swapped(
                        Lib3hToClientResponse::HandleFetchEntryResult(fetch_entry_result_data),
                    )))
                }
            }
            // Publish data to the dht.
            Lib3hClientProtocol::PublishEntry(provided_entry_data) => {
                //let log_context = "ClientToLib3h::PublishEntry";

                if self.is_full_sync_DHT {
                    // As with QueryEntry, if we are in full-sync DHT mode,
                    // this means that we can play back PublishEntry messages already locally
                    // as HandleStoreEntryAspects.
                    // This makes instances with Sim2hWorker work even if offline,
                    // i.e. not connected to the sim2h node.
                    for aspect in &provided_entry_data.entry.aspect_list {
                        let msg =
                            Lib3hServerProtocol::HandleStoreEntryAspect(StoreEntryAspectData {
                                request_id: "".into(),
                                space_address: provided_entry_data.space_address.clone(),
                                provider_agent_id: provided_entry_data.provider_agent_id.clone(),
                                entry_address: provided_entry_data.entry.entry_address.clone(),
                                entry_aspect: aspect.clone(),
                            });
                        self.to_core.push(span_wrap.swapped(msg));
                    }
                }

                self.send_wire_message(WireMessage::ClientToLib3h(
                    span_wrap.swapped(ClientToLib3h::PublishEntry(provided_entry_data)),
                ))
            }
            // Request some info / data from a Entry
            Lib3hClientProtocol::QueryEntry(query_entry_data) => {
                if self.is_full_sync_DHT {
                    // In a full-sync DHT queries should always be handled locally.
                    // Thus, we don't even need to ask the central sim2h instance
                    // to handle a query - we just send it back to core directly.
                    let msg = Lib3hServerProtocol::HandleQueryEntry(query_entry_data);
                    self.to_core.push(span_wrap.swapped(msg));
                    Ok(())
                } else {
                    self.send_wire_message(WireMessage::ClientToLib3h(
                        span_wrap.swapped(ClientToLib3h::QueryEntry(query_entry_data)),
                    ))
                }
            }
            // Response to a `HandleQueryEntry` request
            Lib3hClientProtocol::HandleQueryEntryResult(query_entry_result_data) => {
                if self.is_full_sync_DHT {
                    // See above QueryEntry implementation.
                    // All queries are handled locally - we just reflect them back to core:
                    let msg = Lib3hServerProtocol::QueryEntryResult(query_entry_result_data);
                    self.to_core.push(span_wrap.swapped(msg));
                    Ok(())
                } else {
                    self.send_wire_message(WireMessage::Lib3hToClientResponse(span_wrap.swapped(
                        Lib3hToClientResponse::HandleQueryEntryResult(query_entry_result_data),
                    )))
                }
            }

            // -- Entry lists -- //
            Lib3hClientProtocol::HandleGetAuthoringEntryListResult(entry_list_data) => {
                //let log_context = "ClientToLib3h::HandleGetAuthoringEntryListResult";
                self.initial_authoring_list = Some(entry_list_data.clone());
                if self.is_full_sync_DHT {
                    self.self_store_authored_aspects();
                }
                self.send_wire_message(WireMessage::Lib3hToClientResponse(span_wrap.swapped(
                    Lib3hToClientResponse::HandleGetAuthoringEntryListResult(entry_list_data),
                )))
            }
            Lib3hClientProtocol::HandleGetGossipingEntryListResult(entry_list_data) => {
                //let log_context = "ClientToLib3h::HandleGetGossipingEntryListResult";
                self.initial_gossiping_list = Some(entry_list_data.clone());
                if self.is_full_sync_DHT {
                    self.self_store_authored_aspects();
                }
                self.send_wire_message(WireMessage::Lib3hToClientResponse(span_wrap.swapped(
                    Lib3hToClientResponse::HandleGetGossipingEntryListResult(entry_list_data),
                )))
            }

            // -- deprecated unctinonality -- //
            Lib3hClientProtocol::Shutdown => {
                debug!("Got Lib3hClientProtocol::Shutdown from core in sim2h worker");
                Ok(())
            }
        }
    }

    #[autotrace]
    fn self_store_authored_aspects(&mut self) {
        if !self.has_self_stored_authored_aspects
            && self.initial_gossiping_list.is_some()
            && self.initial_authoring_list.is_some()
        {
            let authoring_list = self.initial_authoring_list.take().unwrap();
            let gossiping_list = self.initial_gossiping_list.take().unwrap();

            for (entry_hash, aspect_hashes) in &authoring_list.address_map {
                // Check if we have that entry in the gossip list already:
                if let Some(gossiping_aspects) = gossiping_list.address_map.get(entry_hash) {
                    // If it's in, check if we are holding all aspects...
                    let mut authoring_aspects = aspect_hashes.clone();
                    // ...by removing all we are holding...
                    for aspect in gossiping_aspects {
                        authoring_aspects.remove_item(aspect);
                    }
                    // ...and checking if we are left with anything to hold.
                    if authoring_aspects.is_empty() {
                        continue;
                    }
                }
                let msg = Lib3hServerProtocol::HandleFetchEntry(FetchEntryData {
                    space_address: authoring_list.space_address.clone(),
                    entry_address: entry_hash.clone(),
                    request_id: SIM2H_WORKER_INTERNAL_REQUEST_ID.to_string(),
                    provider_agent_id: authoring_list.provider_agent_id.clone(),
                    aspect_address_list: Some(aspect_hashes.clone()),
                });
                let span = ht::top_follower("pre-send");
                self.to_core.push(span.wrap(msg).into())
            }
            self.has_self_stored_authored_aspects = true;
        }
    }

    fn handle_server_message(&mut self, message: WireMessage) -> NetResult<()> {
        let span = ht::with_top_or_null(|s| s.child("handle_server_message"));
        match message {
            WireMessage::Ping => self.send_wire_message(WireMessage::Pong)?,
            WireMessage::Pong => {}
            // Todo update the filter
            WireMessage::TraceFilter(_filter) => {}
            WireMessage::TraceFilterResponse(_) => {}
            WireMessage::Lib3hToClient(span_wrap) => self.to_core.push(span_wrap.map(|m| Lib3hServerProtocol::from(m))),
            WireMessage::MultiSend(messages) => {
                for span_wrap in messages.into_iter() {
                    self.to_core.push(span_wrap.map(Lib3hServerProtocol::from));
                }
            }
            WireMessage::ClientToLib3hResponse(span_wrap) => {
                self.to_core.push(span_wrap.map(Lib3hServerProtocol::from))
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
                            span.wrap(ClientToLib3h::JoinSpace(space_data)).into(),
                        ))?;
                    } else {
                        error!("Uh oh, we got a MessageWhileInLimbo errro and we don't have space data. Did core send a message before sending a join? This should not happen.");
                    }
                }
                WireError::Other(e) => error!("Got error from Sim2h server: {:?}", e),
            },
            WireMessage::Status => error!("Got a Status from the Sim2h server, weird! Ignoring"),
            WireMessage::Hello(_) => error!("Got a Hello from the Sim2h server, weird! Ignoring"),
            WireMessage::HelloResponse(response) => {
                if WIRE_VERSION != response.version {
                    panic!("holochain SIM2H WIRE_VERSION ({}) does not match SIM2H server WIRE_VERSION ({}) - cannot continue", WIRE_VERSION, response.version);
                }
                debug!("HelloResponse {:?}", response);
                self.set_full_sync(response.redundant_count == 0);
            }
            WireMessage::StatusResponse(_) => error!("Got a StatusResponse from the Sim2h server, weird! Ignoring (I use Hello not Status)"),
            WireMessage::Ack(hash) => {
                if self.outgoing_message_buffer
                    .first()
                    .and_then(|buffered_message| Some(buffered_message.hash == hash))
                    .unwrap_or(false)
                {
                    debug!("WireMessage::Ack received => dequeuing sent message {:?}", message);
                    // if we made it here, we successfully sent the first message
                    // we can remove it from the outgoing buffer queue
                    self.outgoing_message_buffer.remove(0);
                } else {
                    warn!(
                        "WireMessage::Ack received that came out of order! Got hash: {}, have top hash: {:?}",
                        hash,
                        self.outgoing_message_buffer
                            .first()
                            .map(|buffered_message| buffered_message.hash)
                    );
                }
            }
        };
        Ok(())
    }

    pub fn set_full_sync(&mut self, full_sync: bool) {
        self.is_full_sync_DHT = full_sync;
    }

    #[allow(dead_code)]
    fn send_ping(&mut self) {
        trace!("Ping");
        if let Err(e) = self.send_wire_message(WireMessage::Ping) {
            debug!("Ping failed with: {:?}", e);
        }
    }

    /// test function for proving out reconnects
    /// note this cannot be cfg(test) because we want to invoke it
    /// from integration testing
    pub fn test_close_connection_cause_reconnect(&mut self) {
        self.connection = None;
        self.time_of_last_connection_attempt = std::time::Instant::now()
            .checked_sub(self.reconnect_interval * 2)
            .unwrap();
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_NET)]
impl NetWorker for Sim2hWorker {
    /// We got a message from core
    /// -> forward it to the NetworkEngine
    fn receive(&mut self, data: Lib3hClientProtocolWrapped) -> NetResult<()> {
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
            self.reset_backoff();
            if self.try_send_from_outgoing_buffer() {
                did_something = true;
            }

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
            let metric = holochain_metrics::Metric::new(
                metric_name,
                None,
                Some(clock.into()),
                latency as f64,
            );
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
