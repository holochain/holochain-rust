#![feature(vec_remove_item)]

extern crate backtrace;
extern crate env_logger;
extern crate lib3h_crypto_api;
extern crate log;
extern crate nanoid;
extern crate num_cpus;
extern crate threadpool;
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
        EntryData, EntryListData, FetchEntryData, GetListData, Opaque, QueryEntryData, SpaceData,
        StoreEntryAspectData,
    },
    protocol::*,
    types::SpaceHash,
    uri::Lib3hUri,
};
use url2::prelude::*;

pub use wire_message::{
    HelloData, StatusData, WireError, WireMessage, WireMessageVersion, WIRE_VERSION,
};

use im::{HashMap, HashSet};
use in_stream::*;
use log::*;
use rand::{seq::SliceRandom, thread_rng};
use std::{convert::TryFrom, sync::Arc};

use holochain_locksmith::Mutex;
use holochain_metrics::{
    config::MetricPublisherConfig, with_latency_publishing, Metric, MetricPublisher,
};
use threadpool::ThreadPool;

mod sim2h_context;
use sim2h_context::*;

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

const RETRY_FETCH_MISSING_ASPECTS_INTERVAL_MS: u64 = 30000; // 30 seconds

fn conn_lifecycle(desc: &str, uuid: &str, obj: &ConnectionState, uri: &Lib3hUri) {
    debug!(
        "connection event conn: {} for {}@{} {:?}",
        desc, uuid, uri, obj
    );
}

fn open_lifecycle(desc: &str, uuid: &str, uri: &Lib3hUri) {
    debug!("connection event open_conns: {} for {}@{}", desc, uuid, uri);
}

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

enum PoolTask {
    //    VerifyPayload(Result<(Lib3hUri, WireMessage, AgentPubKey), ()>),
    Disconnect(Vec<Lib3hUri>),
}

pub struct Sim2h {
    sim2h_context: Sim2hContextRef,
    pub bound_uri: Option<Lib3hUri>,
    pool: Pool,
    wss_recv: crossbeam_channel::Receiver<TcpWss>,
    msg_send: crossbeam_channel::Sender<(Url2, FrameResult)>,
    msg_recv: crossbeam_channel::Receiver<(Url2, FrameResult)>,
    num_ticks: u64,
    /// when should we try to resync nodes that are still missing aspect data
    missing_aspects_resync: std::time::Instant,
    dht_algorithm: DhtAlgorithm,
    threadpool: ThreadPool,
    tp_send: crossbeam_channel::Sender<PoolTask>,
    tp_recv: crossbeam_channel::Receiver<PoolTask>,
    metric_publisher: std::sync::Arc<holochain_locksmith::RwLock<dyn MetricPublisher>>,
}

impl Sim2h {
    pub fn new(crypto: Box<dyn CryptoSystem>, bind_spec: Lib3hUri) -> Self {
        let pool = Pool::new();
        pool.push_job(Box::new(Arc::new(Mutex::new(Tick::new()))));

        let (wss_send, wss_recv) = crossbeam_channel::unbounded();
        let (msg_send, msg_recv) = crossbeam_channel::unbounded();
        let (tp_send, tp_recv) = crossbeam_channel::unbounded();
        let metric_publisher = MetricPublisherConfig::default().create_metric_publisher();

        let state = Sim2hState {
            crypto: crypto.box_clone(),
            connection_states: std::collections::HashMap::new(),
            open_connections: std::collections::HashMap::new(),
            spaces: HashMap::new(),
            metric_publisher: metric_publisher.clone(),
        };
        let sim2h_context = sim2h_context_thread_pool(crypto.box_clone(), state);
        let mut sim2h = Sim2h {
            sim2h_context,
            bound_uri: None,
            pool,
            wss_recv,
            msg_send,
            msg_recv,
            num_ticks: 0,
            missing_aspects_resync: std::time::Instant::now(),
            dht_algorithm: DhtAlgorithm::FullSync,
            threadpool: ThreadPool::new(num_cpus::get()),
            tp_send,
            tp_recv,
            metric_publisher,
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
    fn priv_check_incoming_connections(&mut self) -> bool {
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
                        return true; //did work despite error.
                    }
                    let uuid = nanoid::simple();
                    open_lifecycle("adding conn job", &uuid, &url);
                    let mut state = self.sim2h_context.delete_me();
                    state.write().open_connections.insert(
                        url,
                        OpenConnectionItem {
                            version: 1, // assume version 1 until we get a Hello
                            uuid,
                            job: job.clone(),
                            sender: outgoing_send,
                        },
                    );
                    self.pool.push_job(Box::new(job));
                    true
                } else {
                    false
                }
            }
        )
    }

    /// we received some kind of error related to a stream/socket
    /// print some debugging and disconnect it
    fn priv_drop_connection_for_error(&mut self, uri: Lib3hUri, error: Sim2hError) {
        debug!(
            "dropping connection to {} because of error: {:?}",
            uri, error,
        );
        self.disconnect(&uri);
    }

    /// if our connections sent us any data, process it
    fn priv_check_incoming_messages(&mut self) -> bool {
        with_latency_publishing!(
            "sim2h-priv_check_incoming_messages",
            self.metric_publisher,
            || {
                let len = self.msg_recv.len();
                if len > 0 {
                    debug!("Handling {} incoming messages", len);
                    debug!("threadpool len {}", self.threadpool.queued_count());
                }
                let v: Vec<_> = self.msg_recv.try_iter().collect();
                for (url, msg) in v {
                    let url: Lib3hUri = url::Url::from(url).into();
                    match msg {
                        Ok(frame) => match frame {
                            WsFrame::Text(s) => self.priv_drop_connection_for_error(
                                url,
                                format!("unexpected text message: {:?}", s).into(),
                            ),
                            WsFrame::Binary(b) => {
                                trace!(
                                    "priv_check_incoming_messages: received a frame from {}",
                                    url
                                );
                                let payload: Opaque = b.into();
                                match Sim2h::verify_payload(payload.clone()) {
                                    Ok((source, wire_message)) => {
                                        if let Err(error) =
                                            self.handle_message(&url, wire_message, &source)
                                        {
                                            error!("Error handling message: {:?}", error);
                                        }
                                    }
                                    Err(error) => {
                                        error!(
                                            "Could not verify payload from {}!\nError: {:?}\nPayload was: {:?}",
                                            url,
                                            error, payload
                                        );
                                    }
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
                false
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

    // adds an agent to a space
    fn join(&mut self, uri: &Lib3hUri, data: &SpaceData) -> Sim2hResult<()> {
        with_latency_publishing!("sim2h-join", self.metric_publisher, || {
            debug!("join entered for {} with {:?}", uri, data);
            let result = if let Some((uuid, conn)) = self.get_connection(uri) {
                if let ConnectionState::Limbo(pending_messages) = conn {
                    let conn = ConnectionState::new_joined(
                        data.space_address.clone(),
                        data.agent_id.clone(),
                    )?;
                    {
                        let mut state = self.sim2h_context.delete_me();
                        let _ = state.write().connection_states.insert(
                            uri.clone(),
                            // MDD: we are overwriting the existing connection state here, so we keep the same uuid.
                            // (This could be done more directly with a Hashmap entry update)
                            (uuid, conn),
                        );

                        state.write().join_agent(
                            &data.space_address,
                            data.agent_id.clone(),
                            uri.clone(),
                        )?;
                    }
                    info!(
                        "Agent {:?} joined space {:?}",
                        data.agent_id, data.space_address
                    );
                    self.request_authoring_list(
                        uri.clone(),
                        data.space_address.clone(),
                        data.agent_id.clone(),
                    );
                    // MDD: why is request_gossiping_list in Sim2hState but not request_authoring_list?
                    self.sim2h_context
                        .delete_me()
                        .write()
                        .request_gossiping_list(
                            uri.clone(),
                            data.space_address.clone(),
                            data.agent_id.clone(),
                        );
                    // MDD: maybe the pending messages shouldn't be handled immediately, but pushed into the queue?
                    debug!("pending messages in join: {}", pending_messages.len());
                    for message in *pending_messages {
                        if let Err(err) = self.handle_message(uri, message.clone(), &data.agent_id)
                        {
                            error!(
                                "Error while handling limbo pending message {:?} for {}: {}",
                                message, uri, err
                            );
                        }
                    }
                    Ok(())
                } else {
                    Err(format!("no agent found in limbo at {} ", uri).into())
                }
            } else {
                Err(format!("no agent found at {} ", uri).into())
            };
            trace!("join done");
            result
        })
    }

    // get the connection status of an agent
    fn get_connection(&self, uri: &Lib3hUri) -> Option<ConnectionStateItem> {
        with_latency_publishing!("sim2h-get_connection", self.metric_publisher, || {
            self.sim2h_context.delete_me().read().get_connection(uri)
        })
    }

    // find out if an agent is in a space or not and return its URI
    fn lookup_joined(&self, space_address: &SpaceHash, agent_id: &AgentId) -> Option<Lib3hUri> {
        with_latency_publishing!("sim2h-lookup_joined", self.metric_publisher, || {
            self.sim2h_context
                .delete_me_lock_space(space_address)?
                .agent_id_to_uri(agent_id)
        })
    }

    // handler for incoming connections
    fn handle_incoming_connect(&self, uri: Lib3hUri) -> Sim2hResult<bool> {
        with_latency_publishing!(
            "sim2h-handle_incoming_connect",
            self.metric_publisher,
            || {
                trace!("handle_incoming_connect entered");
                debug!("New connection from {:?}", uri);
                if let Some(_old) = self
                    .sim2h_context
                    .delete_me()
                    .write()
                    .connection_states
                    .insert(uri.clone(), (nanoid::simple(), ConnectionState::new()))
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
            trace!("handle_message entered for {}", uri);

            MESSAGE_LOGGER
                .lock()
                .log_in(signer.clone(), uri.clone(), message.clone());
            let (uuid, mut agent) = self
                .get_connection(uri)
                .ok_or_else(|| format!("no connection for {}", uri))?;

            conn_lifecycle("handle_message", &uuid, &agent, uri);

            // TODO: anyway, but especially with this Ping/Pong, mitigate DoS attacks.
            if message == WireMessage::Ping {
                debug!("Sending Pong in response to Ping");
                self.send(signer.clone(), uri.clone(), &WireMessage::Pong);
                return Ok(());
            }
            if let WireMessage::Status = message {
                debug!("Sending StatusResponse in response to Status");
                let (spaces_len, connection_count) = {
                    let state = self.sim2h_context.delete_me();
                    let state = state.read();
                    (state.spaces.len(), state.open_connections.len())
                };
                self.send(
                    signer.clone(),
                    uri.clone(),
                    &WireMessage::StatusResponse(StatusData {
                        spaces: spaces_len,
                        connections: connection_count,
                        redundant_count: match self.dht_algorithm {
                            DhtAlgorithm::FullSync => 0,
                            DhtAlgorithm::NaiveSharding { redundant_count } => redundant_count,
                        },
                        version: WIRE_VERSION,
                    }),
                );
                return Ok(());
            }
            if let WireMessage::Hello(version) = message {
                debug!("Sending HelloResponse in response to Hello({})", version);
                {
                    let mut state = self.sim2h_context.delete_me();
                    let state = state.write();
                    if let Some(conn) = state.open_connections.get_mut(uri) {
                        conn.version = version;
                    }
                }
                self.send(
                    signer.clone(),
                    uri.clone(),
                    &WireMessage::HelloResponse(HelloData {
                        redundant_count: match self.dht_algorithm {
                            DhtAlgorithm::FullSync => 0,
                            DhtAlgorithm::NaiveSharding { redundant_count } => redundant_count,
                        },
                        version: WIRE_VERSION,
                        extra: None,
                    }),
                );
                return Ok(());
            }

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
                        debug!("inserting into pending message while in limbo.");
                        // TODO: maybe have some upper limit on the number of messages
                        // we allow to queue before dropping the connections
                        pending_messages.push(message);
                        // MDD: TODO: is it necessary to re-insert the data at the same uri?
                        // didn't we just mutate it in-place?
                        let _ = self
                            .sim2h_context
                            .delete_me()
                            .write()
                            .connection_states
                            .insert(uri.clone(), (uuid, agent));
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

    fn verify_payload(payload: Opaque) -> Sim2hResult<(AgentId, WireMessage)> {
        let signed_message = SignedWireMessage::try_from(payload)?;
        let result = signed_message.verify().unwrap();
        if !result {
            return Err(VERIFY_FAILED_ERR_STR.into());
        }
        let wire_message = WireMessage::try_from(signed_message.payload)?;
        Ok((signed_message.provenance.source().into(), wire_message))
    }

    // process transport and  incoming messages from it
    pub fn process(&mut self) -> Sim2hResult<bool> {
        with_latency_publishing!("sim2h-process", self.metric_publisher, || {
            self.num_ticks += 1;
            if self.num_ticks % 60000 == 0 {
                debug!(".");
                self.num_ticks = 0;
            }

            match self.tp_recv.try_recv() {
                Ok(PoolTask::Disconnect(disconnects)) => {
                    for url in disconnects {
                        self.sim2h_context.delete_me().write().disconnect(&url)
                    }
                }
                //            Ok(PoolTask::VerifyPayload(Ok(_))) => {
                /*/                let debug = url.host().unwrap().to_string() == "68.237.138.100";//  "127.0.0.1";
                    if debug {
                    println!("payload verified from from zippy ({}) message is {:?}", url, wire_message);
                }
                    if let Err(error) = self.handle_message(&url, wire_message, &source) {
                    error!("Error handling message: {:?}", error);
                }*/
                    //      }
                _ => (),
            };

            let did_work_1 = self.priv_check_incoming_connections();
            let did_work_2 = self.priv_check_incoming_messages();
            let did_work = did_work_1 || did_work_2;

            if std::time::Instant::now() >= self.missing_aspects_resync {
                self.missing_aspects_resync = std::time::Instant::now()
                    .checked_add(std::time::Duration::from_millis(
                        RETRY_FETCH_MISSING_ASPECTS_INTERVAL_MS,
                    ))
                    .expect("can add interval ms");

                self.retry_sync_missing_aspects();
            }
            Ok(did_work)
        })
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
        with_latency_publishing!("sim2h-joined", self.metric_publisher, || {
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
                self.sim2h_context.delete_me().write().leave(uri, &data)
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
                self.sim2h_context.delete_me().write().handle_new_entry_data(data.entry, space_address.clone(), agent_id.clone(), self.dht_algorithm.clone());
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
                        .sim2h_context
                        .delete_me_lock_space(&space_address)
                        .expect("space should exists")
                        .all_aspects()
                        .diff(&AspectList::from(HashMap::from(list_data.address_map))),
                    DhtAlgorithm::NaiveSharding {redundant_count} => self
                        .sim2h_context
                        .delete_me_lock_space(&space_address)
                        .expect("space should exist")
                        .aspects_in_shard_for_agent(agent_id, redundant_count)
                        .diff(&AspectList::from(HashMap::from(list_data.address_map)))
                };

                if aspects_missing_at_node.entry_addresses().count() > 0 {
                    warn!("MISSING ASPECTS at {}:\n{}", agent_id, aspects_missing_at_node.pretty_string());

                    // Cache info about what this agent is missing so we can make sure it got it
                    let missing_hashes: HashSet<(EntryHash, AspectHash)> = (&aspects_missing_at_node).into();
                    if missing_hashes.len() > 0 {
                        self.sim2h_context.delete_me().write().add_missing_aspects(space_address, &agent_id, missing_hashes);
                    }

                    match dht_algorithm {

                        DhtAlgorithm::FullSync => {
                            let all_agents_in_space = self
                                .sim2h_context
                                .delete_me_lock_space(&space_address)
                                .expect("space should exist")
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
                                let entry_loc = entry_location(self.sim2h_context.box_crypto(), entry_address);
                                let agent_pool = self
                                    .sim2h_context
                                    .delete_me_lock_space(&space_address)
                                    .expect("space should exist")
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
                    self.sim2h_context.delete_me().write().handle_new_entry_data(fetch_result.entry, space_address.clone(), agent_id.clone(),self.dht_algorithm.clone());
                } else {
                    debug!("Got FetchEntry result with request id {} - this is for gossiping to agent with incomplete data", fetch_result.request_id);
                    let to_agent_id = AgentPubKey::from(fetch_result.request_id);
                    let maybe_url = self.lookup_joined(space_address, &to_agent_id);
                    if maybe_url.is_none() {
                        error!("Got FetchEntryResult with request id that is not a known agent id. I guess we lost that agent before we could deliver missing aspects.");
                        return Ok(())
                    }
                    let url = maybe_url.unwrap();
                    let mut multi_messages = Vec::new();
                    for aspect in fetch_result.entry.aspect_list {
                        self
                            .sim2h_context
                            .delete_me()
                            .write()
                            .remove_missing_aspect(space_address, &to_agent_id, &fetch_result.entry.entry_address, &aspect.aspect_address);
                        multi_messages.push(Lib3hToClient::HandleStoreEntryAspect(
                            StoreEntryAspectData {
                                request_id: "".into(),
                                space_address: space_address.clone(),
                                provider_agent_id: agent_id.clone(),
                                entry_address: fetch_result.entry.entry_address.clone(),
                                entry_aspect: aspect,
                            },
                        ));
                    }
                    if multi_messages.len() > 0 {
                        let store_message = WireMessage::MultiSend(multi_messages);
                        self.send(to_agent_id, url, &store_message);
                    }
                }

                Ok(())
            }
            WireMessage::ClientToLib3h(ClientToLib3h::QueryEntry(query_data)) => {
                if let DhtAlgorithm::NaiveSharding {redundant_count} = self.dht_algorithm {
                    let ctx = self.sim2h_context.clone();
                    let tx = self.tp_send.clone();
                    let space_address = space_address.clone();
                    self.threadpool.execute(move || {
                        if let Some((agent_id, uri, wire_message)) = ctx
                            .delete_me_lock_space(&space_address)
                            .expect("space should exist")
                            .build_query(query_data, redundant_count)
                        {
                            let disconnects = ctx
                                .delete_me()
                                .read()
                                .send(agent_id, uri, &wire_message);
                            tx.send(PoolTask::Disconnect(disconnects))
                                .expect("should send");
                        }
                    });
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
        })
    }

    fn handle_unseen_aspects(
        &self,
        uri: &Lib3hUri,
        space_address: &SpaceHash,
        agent_id: &AgentId,
        list_data: &EntryListData,
    ) {
        let ctx = self.sim2h_context.clone();
        let tx = self.tp_send.clone();
        let uri = uri.clone();
        let space_address = space_address.clone();
        let agent_id = agent_id.clone();
        let list_data = list_data.clone();
        self.threadpool.execute(move || {
            if let Some((agent_id, uri, wire_message)) = ctx
                .delete_me_lock_space(&space_address)
                .expect("space should exist")
                .build_handle_unseen_aspects(uri, agent_id, list_data)
            {
                let disconnects = ctx.delete_me().read().send(agent_id, uri, &wire_message);
                tx.send(PoolTask::Disconnect(disconnects))
                    .expect("should send");
            }
        });
    }

    fn fetch_aspects_from_arbitrary_agent(
        &self,
        aspects_to_fetch: AspectList,
        for_agent_id: AgentId,
        agent_pool: Vec<AgentId>,
        space_address: SpaceHash,
    ) {
        with_latency_publishing!(
            "sim2h-fetch_aspects_from_arbitrary_agent",
            self.metric_publisher,
            || {
                let ctx = self.sim2h_context.clone();
                let tx = self.tp_send.clone();
                self.threadpool.execute(move || {
                    let sends = ctx
                        .delete_me_lock_space(&space_address)
                        .expect("space should exist")
                        .build_aspects_from_arbitrary_agent(
                            aspects_to_fetch,
                            for_agent_id,
                            agent_pool,
                        );
                    for (agent_id, uri, wire_message) in sends {
                        let disconnects = ctx.delete_me().read().send(agent_id, uri, &wire_message);
                        if !disconnects.is_empty() {
                            tx.send(PoolTask::Disconnect(disconnects))
                                .expect("should send");
                        }
                    }
                });
            }
        )
    }

    fn disconnect(&self, uri: &Lib3hUri) {
        self.sim2h_context.delete_me().write().disconnect(uri);
    }

    fn send(&mut self, agent: AgentId, uri: Lib3hUri, msg: &WireMessage) {
        self.sim2h_context.delete_me().write().send(agent, uri, msg);
    }

    fn retry_sync_missing_aspects(&mut self) {
        self.sim2h_context
            .delete_me()
            .write()
            .retry_sync_missing_aspects();
    }
}
