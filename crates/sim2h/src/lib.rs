#![feature(vec_remove_item)]

extern crate backtrace;
extern crate env_logger;
extern crate lib3h_crypto_api;
extern crate log;
extern crate nanoid;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate lazy_static;

#[allow(dead_code)]
mod naive_sharding;

pub mod cache;
pub mod connection_state;
pub mod crypto;
pub mod error;
use lib3h_protocol::types::{AgentPubKey, AspectHash, EntryHash};
mod message_log;
pub mod websocket;
pub mod wire_message;

pub use crate::message_log::MESSAGE_LOGGER;
use crate::{crypto::*, error::*, naive_sharding::entry_location};
use cache::*;
use connection_state::*;
use lib3h::rrdht_util::*;
use lib3h_crypto_api::CryptoSystem;
use lib3h_protocol::{
    data_types::{
        EntryData, EntryListData, FetchEntryData, GetListData, Opaque, SpaceData,
        StoreEntryAspectData,
    },
    protocol::*,
    types::SpaceHash,
    uri::Lib3hUri,
};
use url2::prelude::*;

pub use wire_message::{StatusData, WireError, WireMessage, WIRE_VERSION};

use in_stream::*;
use log::*;
use parking_lot::RwLock;
use rand::{seq::SliceRandom, thread_rng};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    sync::Arc,
};

use holochain_locksmith::Mutex;
use holochain_metrics::{
    config::MetricPublisherConfig, with_latency_publishing, Metric, MetricPublisher,
};

/// if we can't acquire a lock in 20 seconds, panic!
const MAX_LOCK_TIMEOUT: u64 = 20000;

/// extention trait for making sure deadlocks are fatal
pub(crate) trait MutexExt<T> {
    /// will attempt to aquire a lock within a time-frame and panic after
    /// this way deadlocks don't just lock forever
    fn f_lock(&self) -> holochain_locksmith::MutexGuard<T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn f_lock(&self) -> holochain_locksmith::MutexGuard<T> {
        // if we can't acquire a lock in 20 seconds, panic!
        self.try_lock_for(std::time::Duration::from_millis(MAX_LOCK_TIMEOUT))
            .expect("failed to obtain mutex lock")
    }
}

/// if a channel send fails, it means it is disconnected
/// this extension trait simplifies panic!ing in that case
/// in a lot of places, we expect the channel to always be open
/// and don't have the infrustructure to deal with degenerate cases
/// this trait makes sending more readable when we want to panic! on disconnects
pub(crate) trait SendExt<T> {
    fn f_send(&self, v: T);
}

impl<T> SendExt<T> for crossbeam_channel::Sender<T> {
    fn f_send(&self, v: T) {
        self.send(v).expect("failed to send on crossbeam_channel");
    }
}

const RETRY_FETCH_MISSING_ASPECTS_INTERVAL_MS: u64 = 10000; // 10 seconds

//pub(crate) type TcpWssServer = InStreamListenerWss<InStreamListenerTls<InStreamListenerTcp>>;
//pub(crate) type TcpWss = InStreamWss<InStreamTls<InStreamTcp>>;
pub(crate) type TcpWssServer = InStreamListenerWss<InStreamListenerTcp>;
pub type TcpWss = InStreamWss<InStreamTcp>;

mod job;
use job::*;

#[derive(Clone)]
pub enum DhtAlgorithm {
    FullSync,
    NaiveSharding { redundant_count: u64 },
}

pub struct Sim2h {
    crypto: Box<dyn CryptoSystem>,
    pub bound_uri: Option<Lib3hUri>,
    connection_states: RwLock<HashMap<Lib3hUri, ConnectionState>>,
    spaces: HashMap<SpaceHash, RwLock<Space>>,
    pool: Pool,
    wss_recv: crossbeam_channel::Receiver<TcpWss>,
    msg_send: crossbeam_channel::Sender<(Url2, FrameResult)>,
    msg_recv: crossbeam_channel::Receiver<(Url2, FrameResult)>,
    open_connections: HashMap<
        Lib3hUri,
        (
            Arc<Mutex<ConnectionJob>>,
            crossbeam_channel::Sender<WsFrame>,
        ),
    >,
    num_ticks: u64,
    /// when should we try to resync nodes that are still missing aspect data
    missing_aspects_resync: std::time::Instant,
    dht_algorithm: DhtAlgorithm,
    metric_publisher: std::sync::Arc<holochain_locksmith::RwLock<dyn MetricPublisher>>,
}

impl Sim2h {
    pub fn new(crypto: Box<dyn CryptoSystem>, bind_spec: Lib3hUri) -> Self {
        let pool = Pool::new();
        pool.push_job(Box::new(Arc::new(Mutex::new(Tick::new()))));

        let (wss_send, wss_recv) = crossbeam_channel::unbounded();
        let (msg_send, msg_recv) = crossbeam_channel::unbounded();

        let mut sim2h = Sim2h {
            crypto,
            bound_uri: None,
            connection_states: RwLock::new(HashMap::new()),
            spaces: HashMap::new(),
            pool,
            wss_recv,
            msg_send,
            msg_recv,
            open_connections: HashMap::new(),
            num_ticks: 0,
            missing_aspects_resync: std::time::Instant::now(),
            dht_algorithm: DhtAlgorithm::FullSync,
            metric_publisher: MetricPublisherConfig::default().create_metric_publisher(),
        };

        sim2h.priv_bind_listening_socket(url::Url::from(bind_spec).into(), wss_send);

        sim2h
    }

    pub fn set_dht_algorithm(&mut self, new_algo: DhtAlgorithm) {
        self.dht_algorithm = new_algo;
    }

    /// bind a listening socket, and set up the polling job to accept connections
    fn priv_bind_listening_socket(
        &mut self,
        url: Url2,
        wss_send: crossbeam_channel::Sender<TcpWss>,
    ) {
        let config = TcpBindConfig::default();
        //        let config = TlsBindConfig::new(config).dev_certificate();
        let config = WssBindConfig::new(config);
        let listen: TcpWssServer = InStreamListenerWss::bind(&url, config).unwrap();
        self.bound_uri = Some(url::Url::from(listen.binding()).into());
        self.pool
            .push_job(Box::new(Arc::new(Mutex::new(ListenJob::new(
                listen, wss_send,
            )))));
    }

    /// if our listening socket has accepted any new connections, set them up
    fn priv_check_incoming_connections(&mut self) {
        with_latency_publishing!(
            "sim2h-priv_check_incoming_connections",
            self.metric_publisher,
            || {
                if let Ok(wss) = self.wss_recv.try_recv() {
                    let url: Lib3hUri = url::Url::from(wss.remote_url()).into();
                    let (job, outgoing_send) = ConnectionJob::new(wss, self.msg_send.clone());
                    let job = Arc::new(Mutex::new(job));
                    if let Err(error) = self.handle_incoming_connect(url.clone()) {
                        error!("Error handling incoming connection: {:?}", error);
                        return;
                    }
                    self.open_connections
                        .insert(url, (job.clone(), outgoing_send));
                    self.pool.push_job(Box::new(job));
                }
            }
        )
    }

    /// we received some kind of error related to a stream/socket
    /// print some debugging and disconnect it
    fn priv_drop_connection_for_error(&mut self, uri: Lib3hUri, error: Sim2hError) {
        error!(
            "Transport error occurred on connection to {}: {:?}",
            uri, error,
        );
        info!("Dropping connection to {} because of error", uri);
        self.disconnect(&uri);
    }

    /// if our connections sent us any data, process it
    fn priv_check_incoming_messages(&mut self) {
        with_latency_publishing!(
            "sim2h-priv_check_incoming_messages",
            self.metric_publisher,
            || {
                if let Ok((url, msg)) = self.msg_recv.try_recv() {
                    let url: Lib3hUri = url::Url::from(url).into();
                    match msg {
                        Ok(frame) => match frame {
                            WsFrame::Text(s) => self.priv_drop_connection_for_error(
                                url,
                                format!("unexpected text message: {:?}", s).into(),
                            ),
                            WsFrame::Binary(b) => {
                                let payload: Opaque = b.into();
                                match self.verify_payload(payload.clone()) {
                                    Ok((source, wire_message)) => {
                                        if let Err(error) =
                                            self.handle_message(&url, wire_message, &source)
                                        {
                                            error!("Error handling message: {:?}", error);
                                        }
                                    }
                                    Err(error) => error!(
                                        "Could not verify payload!\nError: {:?}\nPayload was: {:?}",
                                        error, payload
                                    ),
                                }
                            }
                            // TODO - we should use websocket ping/pong
                            //        instead of rolling our own on top of Binary
                            WsFrame::Ping(_) => (),
                            WsFrame::Pong(_) => (),
                            WsFrame::Close(c) => {
                                debug!("Disconnecting {} after connection reset {:?}", url, c);
                                self.disconnect(&url);
                            }
                        },
                        Err(e) => self.priv_drop_connection_for_error(url, e),
                    }
                }
            }
        )
    }

    fn request_authoring_list(
        &mut self,
        uri: Lib3hUri,
        space_address: SpaceHash,
        provider_agent_id: AgentId,
    ) {
        with_latency_publishing!(
            "sim2h-request_authoring_list",
            self.metric_publisher,
            || {
                let wire_message = WireMessage::Lib3hToClient(
                    Lib3hToClient::HandleGetAuthoringEntryList(GetListData {
                        request_id: "".into(),
                        space_address,
                        provider_agent_id: provider_agent_id.clone(),
                    }),
                );
                self.send(provider_agent_id, uri, &wire_message);
            }
        )
    }

    fn request_gossiping_list(
        &mut self,
        uri: Lib3hUri,
        space_address: SpaceHash,
        provider_agent_id: AgentId,
    ) {
        with_latency_publishing!(
            "sim2h-request_gossiping_list",
            self.metric_publisher,
            || {
                let wire_message = WireMessage::Lib3hToClient(
                    Lib3hToClient::HandleGetGossipingEntryList(GetListData {
                        request_id: "".into(),
                        space_address,
                        provider_agent_id: provider_agent_id.clone(),
                    }),
                );
                self.send(provider_agent_id, uri, &wire_message);
            }
        )
    }

    fn get_or_create_space(&mut self, space_address: &SpaceHash) -> &RwLock<Space> {
        let clock = std::time::SystemTime::now();

        if !self.spaces.contains_key(space_address) {
            self.spaces.insert(
                space_address.clone(),
                RwLock::new(Space::new(self.crypto.box_clone())),
            );
            info!(
                "\n\n+++++++++++++++\nNew Space: {}\n+++++++++++++++\n",
                space_address
            );
        }

        let rw_lock = self.spaces.get(space_address).unwrap();
        self.metric_publisher
            .write()
            .unwrap()
            .publish(&Metric::new_timestamped_now(
                "sim2h-get_or_create_space.latency",
                None,
                clock.elapsed().unwrap().as_millis() as f64,
            ));
        rw_lock
    }

    // adds an agent to a space
    fn join(&mut self, uri: &Lib3hUri, data: &SpaceData) -> Sim2hResult<()> {
        with_latency_publishing!("sim2h-join", self.metric_publisher, || {
            trace!("join entered");
            let result = if let Some(ConnectionState::Limbo(pending_messages)) =
                self.get_connection(uri)
            {
                let _ = self.connection_states.write().insert(
                    uri.clone(),
                    ConnectionState::new_joined(data.space_address.clone(), data.agent_id.clone())?,
                );

                self.get_or_create_space(&data.space_address)
                    .write()
                    .join_agent(data.agent_id.clone(), uri.clone())?;
                info!(
                    "Agent {:?} joined space {:?}",
                    data.agent_id, data.space_address
                );
                self.request_authoring_list(
                    uri.clone(),
                    data.space_address.clone(),
                    data.agent_id.clone(),
                );
                self.request_gossiping_list(
                    uri.clone(),
                    data.space_address.clone(),
                    data.agent_id.clone(),
                );
                for message in *pending_messages {
                    if let Err(err) = self.handle_message(uri, message.clone(), &data.agent_id) {
                        error!(
                            "Error while handling limbo pending message {:?} for {}: {}",
                            message, uri, err
                        );
                    }
                }
                Ok(())
            } else {
                Err(format!("no agent found in limbo at {} ", uri).into())
            };
            trace!("join done");
            result
        })
    }

    // removes an agent from a space
    fn leave(&mut self, uri: &Lib3hUri, data: &SpaceData) -> Sim2hResult<()> {
        with_latency_publishing!("sim2h-disconnnect", self.metric_publisher, || {
            if let Some(ConnectionState::Joined(space_address, agent_id)) = self.get_connection(uri)
            {
                if (data.agent_id != agent_id) || (data.space_address != space_address) {
                    Err(SPACE_MISMATCH_ERR_STR.into())
                } else {
                    self.disconnect(uri);
                    Ok(())
                }
            } else {
                Err(format!("no joined agent found at {} ", &uri).into())
            }
        })
    }

    // removes a uri from connection and from spaces
    fn disconnect(&mut self, uri: &Lib3hUri) {
        with_latency_publishing!("sim2h-disconnnect", self.metric_publisher, || {
            trace!("disconnect entered");

            if let Some((con, _outgoing_send)) = self.open_connections.remove(uri) {
                con.f_lock().stop();
            }

            if let Some(ConnectionState::Joined(space_address, agent_id)) =
                self.connection_states.write().remove(uri)
            {
                if let Some(space_lock) = self.spaces.get(&space_address) {
                    if space_lock.write().remove_agent(&agent_id) == 0 {
                        self.spaces.remove(&space_address);
                    }
                }
            }
            trace!("disconnect done");
        })
    }

    // get the connection status of an agent
    fn get_connection(&self, uri: &Lib3hUri) -> Option<ConnectionState> {
        with_latency_publishing!("sim2h-get_connection", self.metric_publisher, || {
            let reader = self.connection_states.read();
            reader.get(uri).map(|ca| (*ca).clone())
        })
    }

    // find out if an agent is in a space or not and return its URI
    fn lookup_joined(&self, space_address: &SpaceHash, agent_id: &AgentId) -> Option<Lib3hUri> {
        with_latency_publishing!(
            "sim2h-handle_incoming_connect",
            self.metric_publisher,
            || {
                self.spaces
                    .get(space_address)?
                    .read()
                    .agent_id_to_uri(agent_id)
            }
        )
    }

    // handler for incoming connections
    fn handle_incoming_connect(&self, uri: Lib3hUri) -> Sim2hResult<bool> {
        with_latency_publishing!(
            "sim2h-handle_incoming_connect",
            self.metric_publisher,
            || {
                trace!("handle_incoming_connect entered");
                info!("New connection from {:?}", uri);
                if let Some(_old) = self
                    .connection_states
                    .write()
                    .insert(uri.clone(), ConnectionState::new())
                {
                    println!("TODO should remove {}", uri); //TODO
                };
                trace!("handle_incoming_connect done");
                Ok(true)
            }
        )
    }

    // handler for messages sent to sim2h
    fn handle_message(
        &mut self,
        uri: &Lib3hUri,
        message: WireMessage,
        signer: &AgentId,
    ) -> Sim2hResult<()> {
        with_latency_publishing!("sim2h-handle_messsage", self.metric_publisher, || {
            // TODO: anyway, but especially with this Ping/Pong, mitigate DoS attacks.
            if message == WireMessage::Ping {
                trace!("Ping -> Pong");
                self.send(signer.clone(), uri.clone(), &WireMessage::Pong);
                return Ok(());
            }
            if message == WireMessage::Status {
                trace!("Status -> StatusResponse");
                self.send(
                    signer.clone(),
                    uri.clone(),
                    &WireMessage::StatusResponse(StatusData {
                        spaces: self.spaces.len(),
                        connections: self.open_connections.len(),
                        redundant_count: match self.dht_algorithm {
                            DhtAlgorithm::FullSync => 0,
                            DhtAlgorithm::NaiveSharding { redundant_count } => redundant_count,
                        },
                        version: WIRE_VERSION,
                    }),
                );
                return Ok(());
            }
            MESSAGE_LOGGER
                .lock()
                .log_in(signer.clone(), uri.clone(), message.clone());
            trace!("handle_message entered");
            let mut agent = self
                .get_connection(uri)
                .ok_or_else(|| format!("no connection for {}", uri))?;

            match agent {
                // if the agent sending the message is in limbo, then the only message
                // allowed is a join message.
                ConnectionState::Limbo(ref mut pending_messages) => {
                    if let WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(data)) = message {
                        if &data.agent_id != signer {
                            return Err(SIGNER_MISMATCH_ERR_STR.into());
                        }
                        self.join(uri, &data)
                    } else {
                        // TODO: maybe have some upper limit on the number of messages
                        // we allow to queue before dropping the connections
                        pending_messages.push(message);
                        let _ = self.connection_states.write().insert(uri.clone(), agent);
                        self.send(
                            signer.clone(),
                            uri.clone(),
                            &WireMessage::Err(WireError::MessageWhileInLimbo),
                        );
                        Ok(())
                    }
                }

                // if the agent sending the messages has been vetted and is in the space
                // then build a message to be proxied to the correct destination, and forward it
                ConnectionState::Joined(space_address, agent_id) => {
                    if &agent_id != signer {
                        return Err(SIGNER_MISMATCH_ERR_STR.into());
                    }
                    self.handle_joined(uri, &space_address, &agent_id, message)
                }
            }
        })
    }

    fn verify_payload(&self, payload: Opaque) -> Sim2hResult<(AgentId, WireMessage)> {
        with_latency_publishing!("sim2h-verify_payload", self.metric_publisher, || {
            let signed_message = SignedWireMessage::try_from(payload)?;
            let result = signed_message.verify().unwrap();
            if !result {
                return Err(VERIFY_FAILED_ERR_STR.into());
            }
            let wire_message = WireMessage::try_from(signed_message.payload)?;
            Ok((signed_message.provenance.source().into(), wire_message))
        })
    }

    // process transport and  incoming messages from it
    pub fn process(&mut self) -> Sim2hResult<()> {
        self.num_ticks += 1;
        if self.num_ticks % 60000 == 0 {
            debug!(".");
            self.num_ticks = 0;
        }

        self.priv_check_incoming_connections();
        self.priv_check_incoming_messages();

        if std::time::Instant::now() >= self.missing_aspects_resync {
            self.missing_aspects_resync = std::time::Instant::now()
                .checked_add(std::time::Duration::from_millis(
                    RETRY_FETCH_MISSING_ASPECTS_INTERVAL_MS,
                ))
                .expect("can add interval ms");

            self.retry_sync_missing_aspects();
        }

        Ok(())
    }

    fn handle_unseen_aspects(
        &mut self,
        uri: &Lib3hUri,
        space_address: &SpaceHash,
        agent_id: &AgentId,
        list_data: &EntryListData,
    ) {
        let unseen_aspects = AspectList::from(list_data.address_map.clone())
            .diff(self.get_or_create_space(space_address).read().all_aspects());
        debug!("UNSEEN ASPECTS:\n{}", unseen_aspects.pretty_string());
        for entry_address in unseen_aspects.entry_addresses() {
            if let Some(aspect_address_list) = unseen_aspects.per_entry(entry_address) {
                let wire_message =
                    WireMessage::Lib3hToClient(Lib3hToClient::HandleFetchEntry(FetchEntryData {
                        request_id: "".into(),
                        space_address: space_address.clone(),
                        provider_agent_id: agent_id.clone(),
                        entry_address: entry_address.clone(),
                        aspect_address_list: Some(aspect_address_list.clone()),
                    }));
                self.send(agent_id.clone(), uri.clone(), &wire_message);
            }
        }
    }

    // given an incoming messages, prepare a proxy message and whether it's an publish or request
    #[allow(clippy::cognitive_complexity)]
    fn handle_joined(
        &mut self,
        uri: &Lib3hUri,
        space_address: &SpaceHash,
        agent_id: &AgentId,
        message: WireMessage,
    ) -> Sim2hResult<()> {
        trace!("handle_joined entered");
        debug!(
            "<<IN<< {} from {}",
            message.message_type(),
            agent_id.to_string()
        );
        match message {
            // First make sure we are not receiving a message in the wrong direction.
            // Panic for now so we can easily spot a mistake.
            // Should maybe break up WireMessage into two different structs so we get the
            // error already when parsing an incoming payload.
            WireMessage::Lib3hToClient(_) | WireMessage::ClientToLib3hResponse(_) =>
                panic!("This is soo wrong. Clients should never send a message that only servers can send."),
            // -- Space -- //
            WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(_)) => {
                Err("join message should have been processed elsewhere and can't be proxied".into())
            }
            WireMessage::ClientToLib3h(ClientToLib3h::LeaveSpace(data)) => {
                self.leave(uri, &data)
            }

            // -- Direct Messaging -- //
            // Send a message directly to another agent on the network
            WireMessage::ClientToLib3h(ClientToLib3h::SendDirectMessage(dm_data)) => {
                if (dm_data.from_agent_id != *agent_id) || (dm_data.space_address != *space_address)
                {
                    return Err(SPACE_MISMATCH_ERR_STR.into());
                }
                let to_url = self
                    .lookup_joined(space_address, &dm_data.to_agent_id)
                    .ok_or_else(|| format!("unvalidated proxy agent {}", &dm_data.to_agent_id))?;
                self.send(
                    dm_data.to_agent_id.clone(),
                    to_url,
                    &WireMessage::Lib3hToClient(Lib3hToClient::HandleSendDirectMessage(dm_data))
                );
                Ok(())
            }
            // Direct message response
            WireMessage::Lib3hToClientResponse(Lib3hToClientResponse::HandleSendDirectMessageResult(
                dm_data,
            )) => {
                if (dm_data.from_agent_id != *agent_id) || (dm_data.space_address != *space_address)
                {
                    return Err(SPACE_MISMATCH_ERR_STR.into());
                }
                let to_url = self
                    .lookup_joined(space_address, &dm_data.to_agent_id)
                    .ok_or_else(|| format!("unvalidated proxy agent {}", &dm_data.to_agent_id))?;
                self.send(
                    dm_data.to_agent_id.clone(),
                    to_url,
                    &WireMessage::Lib3hToClient(Lib3hToClient::SendDirectMessageResult(dm_data))
                );
                Ok(())
            }
            WireMessage::ClientToLib3h(ClientToLib3h::PublishEntry(data)) => {
                if (data.provider_agent_id != *agent_id) || (data.space_address != *space_address) {
                    return Err(SPACE_MISMATCH_ERR_STR.into());
                }
                self.handle_new_entry_data(data.entry, space_address.clone(), agent_id.clone());
                Ok(())
            }
            WireMessage::Lib3hToClientResponse(Lib3hToClientResponse::HandleGetAuthoringEntryListResult(list_data)) => {
                debug!("GOT AUTHORING LIST from {}", agent_id);
                if (list_data.provider_agent_id != *agent_id) || (list_data.space_address != *space_address) {
                    return Err(SPACE_MISMATCH_ERR_STR.into());
                }
                self.handle_unseen_aspects(uri, space_address, agent_id, &list_data);
                Ok(())
            }
            WireMessage::Lib3hToClientResponse(Lib3hToClientResponse::HandleGetGossipingEntryListResult(list_data)) => {
                debug!("GOT GOSSIPING LIST from {}", agent_id);
                if (list_data.provider_agent_id != *agent_id) || (list_data.space_address != *space_address) {
                    return Err(SPACE_MISMATCH_ERR_STR.into());
                }
                self.handle_unseen_aspects(uri, space_address, agent_id, &list_data);

                let dht_algorithm = self.dht_algorithm.clone();

                // Check if the node is missing any aspects
                let aspects_missing_at_node = match dht_algorithm {
                    DhtAlgorithm::FullSync => self
                        .get_or_create_space(&space_address)
                        .read()
                        .all_aspects()
                        .diff(&AspectList::from(list_data.address_map)),
                    DhtAlgorithm::NaiveSharding {redundant_count} => self
                        .get_or_create_space(&space_address)
                        .read()
                        .aspects_in_shard_for_agent(agent_id, redundant_count)
                        .diff(&AspectList::from(list_data.address_map))
                };

                if aspects_missing_at_node.entry_addresses().count() > 0 {
                    warn!("MISSING ASPECTS at {}:\n{}", agent_id, aspects_missing_at_node.pretty_string());

                    // Cache info about what this agent is missing so we can make sure it got it
                    let missing_hashes: HashSet<(EntryHash, AspectHash)> = (&aspects_missing_at_node).into();
                    if missing_hashes.len() > 0 {
                        let mut space = self
                            .get_or_create_space(&space_address)
                            .write();
                        for (entry_hash, aspect_hash) in missing_hashes {
                            space.add_missing_aspect(agent_id.clone(), entry_hash, aspect_hash);
                        }
                    }

                    match dht_algorithm {

                        DhtAlgorithm::FullSync => {
                            let all_agents_in_space = self
                                .get_or_create_space(&space_address)
                                .read()
                                .all_agents()
                                .keys()
                                .cloned()
                                .collect::<Vec<AgentPubKey>>();
                            if all_agents_in_space.len() == 1 {
                                error!("MISSING ASPECTS and no way to get them. Agent is alone in space..");
                            } else {
                                self.fetch_aspects_from_arbitrary_agent(
                                    aspects_missing_at_node,
                                    agent_id.clone(),
                                    all_agents_in_space,
                                    space_address.clone()
                                );
                            }
                        },

                        DhtAlgorithm::NaiveSharding {redundant_count} => {
                            for entry_address in aspects_missing_at_node.entry_addresses() {
                                let entry_loc = entry_location(&self.crypto, entry_address);
                                let agent_pool = self
                                    .get_or_create_space(&space_address)
                                    .read()
                                    .agents_supposed_to_hold_entry(entry_loc, redundant_count)
                                    .keys()
                                    .cloned()
                                    .collect::<Vec<AgentPubKey>>();
                                self.fetch_aspects_from_arbitrary_agent(
                                    aspects_missing_at_node.filtered_by_entry_hash(|e| e == entry_address),
                                    agent_id.clone(),
                                    agent_pool,
                                    space_address.clone()
                                );
                            }
                        }
                    }
                }

                Ok(())
            }
            WireMessage::Lib3hToClientResponse(
                Lib3hToClientResponse::HandleFetchEntryResult(fetch_result)) => {
                if (fetch_result.provider_agent_id != *agent_id) || (fetch_result.space_address != *space_address) {
                    return Err(SPACE_MISMATCH_ERR_STR.into());
                }
                debug!("HANDLE FETCH ENTRY RESULT: {:?}", fetch_result);
                if fetch_result.request_id == "" {
                    debug!("Got FetchEntry result form {} without request id - must be from authoring list", agent_id);
                    self.handle_new_entry_data(fetch_result.entry, space_address.clone(), agent_id.clone());
                } else {
                    debug!("Got FetchEntry result with request id {} - this is for gossiping to agent with incomplete data", fetch_result.request_id);
                    let to_agent_id = AgentPubKey::from(fetch_result.request_id);
                    let maybe_url = self.lookup_joined(space_address, &to_agent_id);
                    if maybe_url.is_none() {
                        error!("Got FetchEntryResult with request id that is not a known agent id. I guess we lost that agent before we could deliver missing aspects.");
                        return Ok(())
                    }
                    let url = maybe_url.unwrap();
                    for aspect in fetch_result.entry.aspect_list {
                        self
                            .get_or_create_space(&space_address)
                            .write()
                            .remove_missing_aspect(&to_agent_id, &fetch_result.entry.entry_address, &aspect.aspect_address);
                        let store_message = WireMessage::Lib3hToClient(Lib3hToClient::HandleStoreEntryAspect(
                            StoreEntryAspectData {
                                request_id: "".into(),
                                space_address: space_address.clone(),
                                provider_agent_id: agent_id.clone(),
                                entry_address: fetch_result.entry.entry_address.clone(),
                                entry_aspect: aspect,
                            },
                        ));
                        self.send(to_agent_id.clone(), url.clone(), &store_message);
                    }
                }

                Ok(())
            }
            WireMessage::ClientToLib3h(ClientToLib3h::QueryEntry(query_data)) => {
                if let DhtAlgorithm::NaiveSharding {redundant_count} = self.dht_algorithm {
                    let entry_loc = entry_location(&self.crypto, &query_data.entry_address);
                    let agent_pool = self
                        .get_or_create_space(&space_address)
                        .read()
                        .agents_supposed_to_hold_entry(entry_loc, redundant_count)
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>();

                    let query_target = if agent_pool.is_empty() {
                        // If there is nobody we could ask, just send the query back
                        query_data.requester_agent_id.clone()
                    } else {
                        let agents_with_all_aspects_for_entry = agent_pool.iter()
                            .filter(|agent|{
                                !self
                                    .get_or_create_space(&space_address)
                                    .read()
                                    .agent_is_missing_some_aspect_for_entry(agent, &query_data.entry_address)
                            })
                            .cloned()
                            .collect::<Vec<AgentId>>();

                        let mut agents_to_sample_from = if agents_with_all_aspects_for_entry.is_empty() {
                            // If there is nobody who as all aspects of an entry, just
                            // ask somebody of that shard:
                            agent_pool
                        } else {
                            agents_with_all_aspects_for_entry
                        };

                        let agent_slice = &mut agents_to_sample_from[..];
                        agent_slice.shuffle(&mut thread_rng());
                        agent_slice[0].clone()
                    };


                    let maybe_url = self.lookup_joined(space_address, &query_target);
                    if maybe_url.is_none() {
                        error!("Got FetchEntryResult with request id that is not a known agent id. I guess we lost that agent before we could deliver missing aspects.");
                        return Ok(())
                    }
                    let url = maybe_url.unwrap();
                    let query_message = WireMessage::Lib3hToClient(Lib3hToClient::HandleQueryEntry(query_data));
                    self.send(query_target, url, &query_message);
                    Ok(())
                } else {
                    Err("Got ClientToLib3h::QueryEntry in full-sync mode".into())
                }
            }
            WireMessage::Lib3hToClientResponse(Lib3hToClientResponse::HandleQueryEntryResult(query_result)) => {
                if (query_result.responder_agent_id != *agent_id) || (query_result.space_address != *space_address)
                {
                    return Err(SPACE_MISMATCH_ERR_STR.into());
                }
                let to_url = self
                    .lookup_joined(space_address, &query_result.requester_agent_id)
                    .ok_or_else(|| format!("unvalidated proxy agent {}", &query_result.requester_agent_id))?;
                self.send(
                    query_result.requester_agent_id.clone(),
                    to_url,
                    &WireMessage::ClientToLib3hResponse(ClientToLib3hResponse::QueryEntryResult(query_result))
                );
                Ok(())
            }
            _ => {
                warn!("Ignoring unimplemented message: {:?}", message );
                Err(format!("Message not implemented: {:?}", message).into())
            }
        }
    }

    fn fetch_aspects_from_arbitrary_agent(
        &mut self,
        aspects_to_fetch: AspectList,
        for_agent_id: AgentId,
        mut agent_pool: Vec<AgentId>,
        space_address: SpaceHash,
    ) {
        let agent_pool = &mut agent_pool[..];
        agent_pool.shuffle(&mut thread_rng());
        for entry_address in aspects_to_fetch.entry_addresses() {
            if let Some(aspect_address_list) = aspects_to_fetch.per_entry(entry_address) {
                if let Some(arbitrary_agent) = self.get_agent_not_missing_aspects(
                    entry_address,
                    aspect_address_list,
                    &for_agent_id,
                    agent_pool,
                    &space_address,
                ) {
                    debug!(
                        "FETCHING missing contents from RANDOM AGENT: {}",
                        arbitrary_agent
                    );

                    let maybe_url = self.lookup_joined(&space_address, &arbitrary_agent);
                    if maybe_url.is_none() {
                        error!("Could not find URL for randomly selected agent. This should not happen!");
                        return;
                    }
                    let random_url = maybe_url.unwrap();

                    let wire_message = WireMessage::Lib3hToClient(Lib3hToClient::HandleFetchEntry(
                        FetchEntryData {
                            request_id: for_agent_id.clone().into(),
                            space_address: space_address.clone(),
                            provider_agent_id: arbitrary_agent.clone(),
                            entry_address: entry_address.clone(),
                            aspect_address_list: Some(aspect_address_list.clone()),
                        },
                    ));
                    debug!("SENDING fetch with request ID: {:?}", wire_message);
                    self.send(arbitrary_agent.clone(), random_url.clone(), &wire_message);
                } else {
                    warn!("Could not find an agent that has any of the missing aspects. Trying again later...")
                }
            }
        }
    }

    /// Get an agent who has at least one of the aspects specified, and who is not the same as for_agent_id.
    /// `agent_pool` is expected to be randomly shuffled, to ensure that no hotspots are created.
    fn get_agent_not_missing_aspects(
        &self,
        entry_hash: &EntryHash,
        aspects: &Vec<AspectHash>,
        for_agent_id: &AgentId,
        agent_pool: &[AgentId],
        space_address: &SpaceHash,
    ) -> Option<AgentId> {
        let space_lock = self.spaces.get(space_address)?.read();
        agent_pool
            .into_iter()
            // We ignore all agents that are missing all of the same aspects as well since
            // they can't help us.
            .find(|a| {
                **a != *for_agent_id
                    && !space_lock.agent_is_missing_all_aspects(*a, entry_hash, aspects)
            })
            .cloned()
    }

    fn handle_new_entry_data(
        &mut self,
        entry_data: EntryData,
        space_address: SpaceHash,
        provider: AgentPubKey,
    ) {
        // Calculate list of agents that should store new data:
        let dht_agents = match self.dht_algorithm {
            DhtAlgorithm::FullSync => {
                self.all_agents_except_one(space_address.clone(), Some(&provider))
            }
            DhtAlgorithm::NaiveSharding { redundant_count } => {
                let entry_loc = entry_location(&self.crypto, &entry_data.entry_address);
                self.agents_in_neighbourhood(space_address.clone(), entry_loc, redundant_count)
            }
        };

        let aspect_addresses = entry_data
            .aspect_list
            .iter()
            .cloned()
            .map(|aspect_data| aspect_data.aspect_address)
            .collect::<Vec<_>>();
        let mut map = HashMap::new();
        map.insert(entry_data.entry_address.clone(), aspect_addresses);
        let aspect_list = AspectList::from(map);
        debug!("GOT NEW ASPECTS:\n{}", aspect_list.pretty_string());

        for aspect in entry_data.aspect_list {
            // 1. Add hashes to our global list of all aspects in this space:
            {
                let mut space = self.get_or_create_space(&space_address).write();
                space.add_aspect(
                    entry_data.entry_address.clone(),
                    aspect.aspect_address.clone(),
                );
                debug!(
                    "Space {} now knows about these aspects:\n{}",
                    &space_address,
                    space.all_aspects().pretty_string()
                );
            }

            // 2. Create store message
            let store_message = WireMessage::Lib3hToClient(Lib3hToClient::HandleStoreEntryAspect(
                StoreEntryAspectData {
                    request_id: "".into(),
                    space_address: space_address.clone(),
                    provider_agent_id: provider.clone(),
                    entry_address: entry_data.entry_address.clone(),
                    entry_aspect: aspect,
                },
            ));

            // 3. Send store message to selected nodes
            self.broadcast(&store_message, dht_agents.clone());
        }
    }

    fn broadcast(&mut self, msg: &WireMessage, agents: Vec<(AgentId, AgentInfo)>) {
        for (agent, info) in agents {
            debug!("Broadcast: Sending to {:?}", info.uri);
            self.send(agent, info.uri, msg);
        }
    }

    fn all_agents_except_one(
        &mut self,
        space: SpaceHash,
        except: Option<&AgentId>,
    ) -> Vec<(AgentId, AgentInfo)> {
        self.get_or_create_space(&space)
            .read()
            .all_agents()
            .clone()
            .into_iter()
            .filter(|(a, _)| {
                if let Some(exception) = except {
                    *a != *exception
                } else {
                    true
                }
            })
            .collect::<Vec<(AgentId, AgentInfo)>>()
    }

    fn agents_in_neighbourhood(
        &mut self,
        space: SpaceHash,
        entry_loc: Location,
        redundant_count: u64,
    ) -> Vec<(AgentId, AgentInfo)> {
        self.get_or_create_space(&space)
            .read()
            .agents_supposed_to_hold_entry(entry_loc, redundant_count)
            .into_iter()
            .collect::<Vec<(AgentId, AgentInfo)>>()
    }

    fn send(&mut self, agent: AgentId, uri: Lib3hUri, msg: &WireMessage) {
        match msg {
            WireMessage::Ping | WireMessage::Pong => debug!("PingPong: {} at {}", agent, uri),
            _ => {
                debug!(">>OUT>> {} to {}", msg.message_type(), uri);
                MESSAGE_LOGGER
                    .lock()
                    .log_out(agent, uri.clone(), msg.clone());
            }
        }

        let payload: Opaque = msg.clone().into();

        match self.open_connections.get_mut(&uri) {
            None => {
                error!("FAILED TO SEND, NO ROUTE: {}", uri);
                return;
            }
            Some((_con, outgoing_send)) => {
                if let Err(_) = outgoing_send.send(payload.as_bytes().into()) {
                    self.disconnect(&uri);
                }
            }
        }

        match msg {
            WireMessage::Ping | WireMessage::Pong => {}
            _ => debug!("sent."),
        }
    }

    fn retry_sync_missing_aspects(&mut self) {
        with_latency_publishing!(
            "sim2h-retry_sync_missing_aspects",
            self.metric_publisher,
            || {
                debug!("Checking for nodes with missing aspects to retry sync...");
                // Extract all needed info for the call to self.request_gossiping_list() below
                // as copies so we don't have to keep a reference to self.
                let spaces_with_agents_and_uris = self
                    .spaces
                    .iter()
                    .filter_map(|(space_hash, space_lock)| {
                        let space = space_lock.read();
                        let agents = space.agents_with_missing_aspects();
                        // If this space doesn't have any agents with missing aspects,
                        // ignore it:
                        if agents.is_empty() {
                            None
                        } else {
                            // For spaces with agents with missing aspects,
                            // annotate all agent IDs with their corresponding URI:
                            let agent_ids_with_uris: Vec<(AgentId, Lib3hUri)> = agents
                                .iter()
                                .filter_map(|agent_id| {
                                    space
                                        .agent_id_to_uri(agent_id)
                                        .map(|uri| (agent_id.clone(), uri))
                                })
                                .collect();

                            Some((space_hash.clone(), agent_ids_with_uris))
                        }
                    })
                    .collect::<HashMap<SpaceHash, Vec<_>>>();

                for (space_hash, agents) in spaces_with_agents_and_uris {
                    for (agent_id, uri) in agents {
                        debug!("Re-requesting gossip list from {} at {}", agent_id, uri);
                        self.request_gossiping_list(uri, space_hash.clone(), agent_id);
                    }
                }
            }
        )
    }
}
