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
use futures::{
    executor::{ThreadPool, ThreadPoolBuilder},
    future::Future,
};
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

use im::{HashMap, HashSet};
use in_stream::*;
use log::*;
use rand::{seq::SliceRandom, thread_rng};
use std::convert::TryFrom;

use holochain_locksmith::Mutex;
use holochain_metrics::{config::MetricPublisherConfig, with_latency_publishing, MetricPublisher};

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
    fn i_send(&self, v: T) -> bool;
}

impl<T> SendExt<T> for crossbeam_channel::Sender<T> {
    fn f_send(&self, v: T) {
        self.send(v).expect("failed to send on crossbeam_channel");
    }

    fn i_send(&self, v: T) -> bool {
        if let Ok(_) = self.send(v) {
            true
        } else {
            false
        }
    }
}

const RETRY_FETCH_MISSING_ASPECTS_INTERVAL_MS: u64 = 30000; // 30 seconds

fn conn_lifecycle(desc: &str, uuid: &str, obj: &ConnectionState, uri: &Lib3hUri) {
    debug!(
        "connection event conn: {} for {}@{} {:?}",
        desc, uuid, uri, obj
    );
}

lazy_static! {
    /// the global futures thread pool reference
    /// the mutex should only be locked once per thread
    static ref GLB_SIM2H_POOL: Mutex<ThreadPool> = {
        let mut builder = ThreadPoolBuilder::new();
        Mutex::new(builder
            .name_prefix("sim2h-thread-pool-")
            .create()
            .expect("error creating futures thread pool")
        )
    };
}

thread_local! {
    /// the thread local futures thread pool reference
    /// clone on ThreadPool creates a cheap reference to the pool
    /// this way each thread has singleton-ish access without Mutex overhead
    static THRD_SIM2H_POOL: ThreadPool = GLB_SIM2H_POOL.f_lock().clone();
}

/// I haven't found a better way to ensure a long-running future
/// actually yields to other tasks...
/// future::ready() / future::lazy() do not, as `poll` is called immeiately
/// as an optimization.
pub(crate) struct TaskYield(bool);

impl TaskYield {
    pub fn new() -> Self {
        Self(false)
    }
}

impl Future for TaskYield {
    type Output = ();
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut futures::task::Context<'_>,
    ) -> futures::task::Poll<Self::Output> {
        // make sure we return Pending once, so the task yields
        if self.0 {
            futures::task::Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            futures::task::Poll::Pending
        }
    }
}

/// spawn an <Output = ()> future into the sigleton Sim2h futures ThreadPool
fn sim2h_spawn_ok<Fut>(future: Fut)
where
    Fut: Future<Output = ()> + Send + 'static,
{
    THRD_SIM2H_POOL.with(move |pool| pool.spawn_ok(future))
}

/// infinite loop writing a trace!() once per second as verification
/// that our ThreadPool executor is still processing jobs
async fn one_second_tick() {
    loop {
        trace!("sim2h futures thread_pool - one second tick");
        futures_timer::Delay::new(std::time::Duration::from_secs(1)).await;
    }
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

mod state;
use state::*;

pub struct Sim2h {
    crypto: Box<dyn CryptoSystem>,
    pub bound_uri: Option<Lib3hUri>,
    state: Sim2hStateHandle,
    recv_wss_event: crossbeam_channel::Receiver<WssEvent>,
    send_wss_command: crossbeam_channel::Sender<WssCommand>,
    num_ticks: u64,
    /// when should we try to resync nodes that are still missing aspect data
    missing_aspects_resync: std::time::Instant,
    dht_algorithm: DhtAlgorithm,
    metric_publisher: std::sync::Arc<holochain_locksmith::RwLock<dyn MetricPublisher>>,
}

impl Sim2h {
    pub fn new(crypto: Box<dyn CryptoSystem>, bind_spec: Lib3hUri) -> Self {
        sim2h_spawn_ok(one_second_tick());

        let (send_wss_event, recv_wss_event) = crossbeam_channel::unbounded();
        let (send_wss_command, recv_wss_command) = crossbeam_channel::unbounded();

        let mut sim2h = Sim2h {
            crypto: crypto.box_clone(),
            bound_uri: None,
            state: Sim2hStateHandle::new(crypto),
            recv_wss_event,
            send_wss_command,
            num_ticks: 0,
            missing_aspects_resync: std::time::Instant::now(),
            dht_algorithm: DhtAlgorithm::FullSync,
            metric_publisher: MetricPublisherConfig::default().create_metric_publisher(),
        };

        sim2h.priv_bind_listening_socket(
            url::Url::from(bind_spec).into(),
            send_wss_event,
            recv_wss_command,
        );

        sim2h
    }

    pub fn set_dht_algorithm(&mut self, new_algo: DhtAlgorithm) {
        self.dht_algorithm = new_algo;
    }

    /// bind a listening socket, and set up the polling job to accept connections
    fn priv_bind_listening_socket(
        &mut self,
        url: Url2,
        send_wss_event: crossbeam_channel::Sender<WssEvent>,
        recv_wss_command: crossbeam_channel::Receiver<WssCommand>,
    ) {
        let config = TcpBindConfig::default();
        //        let config = TlsBindConfig::new(config).dev_certificate();
        let config = WssBindConfig::new(config);
        let listen: TcpWssServer = InStreamListenerWss::bind(&url, config).unwrap();
        self.bound_uri = Some(url::Url::from(listen.binding()).into());

        let (send_wss, recv_wss) = crossbeam_channel::unbounded();
        let (send_pending, recv_pending) = crossbeam_channel::unbounded();

        // job to accept incoming connections
        sim2h_spawn_ok(listen_job(listen, send_pending));

        // job to drive new connections through handshaking
        sim2h_spawn_ok(pending_job(recv_pending, send_wss));

        // job to process data in and out of connections
        sim2h_spawn_ok(connection_job(recv_wss, send_wss_event, recv_wss_command));
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

    /// handle a batch of incoming wss events
    fn priv_check_wss_events(&mut self) {
        with_latency_publishing!("sim2h-priv_check_wss_events", self.metric_publisher, || {
            let len = self.recv_wss_event.len();
            if len > 0 {
                debug!("Handling {} incoming messages", len);
            }
            for _ in 0..100 {
                match self.recv_wss_event.try_recv() {
                    Ok(event) => match event {
                        WssEvent::IncomingConnection(url) => {
                            let url: Lib3hUri = url::Url::from(url).into();
                            if let Err(error) = self.handle_incoming_connect(url.clone()) {
                                error!("Error handling incoming connection: {:?}", error);
                            }
                        }
                        WssEvent::ReceivedData(url, frame) => {
                            self.priv_handle_received_data(url, frame);
                        }
                        WssEvent::Error(url, e) => {
                            let url: Lib3hUri = url::Url::from(url).into();
                            self.priv_drop_connection_for_error(url, e);
                        }
                    },
                    Err(crossbeam_channel::TryRecvError::Disconnected) => {
                        panic!("broken recv_wss_event channel");
                    }
                    Err(crossbeam_channel::TryRecvError::Empty) => break,
                }
            }
        })
    }

    /// if our connections sent us any data, process it
    fn priv_handle_received_data(&mut self, url: Url2, frame: WsFrame) {
        with_latency_publishing!(
            "sim2h-priv_handle_received_data",
            self.metric_publisher,
            || {
                let url: Lib3hUri = url::Url::from(url).into();
                match frame {
                    WsFrame::Text(s) => self.priv_drop_connection_for_error(
                        url,
                        format!("unexpected text message: {:?}", s).into(),
                    ),
                    WsFrame::Binary(b) => {
                        trace!("received a frame from {}", url);
                        let payload: Opaque = b.into();
                        match self.verify_payload(payload.clone()) {
                            Ok((source, wire_message)) => {
                                trace!(
                                    "frame from from {} verified and decoded to {:?}",
                                    url,
                                    wire_message
                                );
                                if let Err(error) = self.handle_message(&url, wire_message, &source)
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

    fn danger_get_or_create_space(&mut self, space_address: &SpaceHash) -> DangerGuard<'_> {
        self.state.danger_get_or_create_space(space_address)
        /*
        let clock = std::time::SystemTime::now();

        if !self.state.danger_lock().spaces.contains_key(space_address) {
            self.state
                .danger_lock()
                .spaces
                .insert(space_address.clone(), Space::new(self.crypto.box_clone()));
            info!(
                "\n\n+++++++++++++++\nNew Space: {}\n+++++++++++++++\n",
                space_address
            );
        }

        let out = self.state.danger_lock().spaces.get_mut(space_address).unwrap();
        self.metric_publisher
            .write()
            .unwrap()
            .publish(&Metric::new_timestamped_now(
                "sim2h-danger_get_or_create_space.latency",
                None,
                clock.elapsed().unwrap().as_millis() as f64,
            ));
        out
        */
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
                    let _ = self.state.danger_lock().connections.insert(
                        uri.clone(),
                        // MDD: we are overwriting the existing connection state here, so we keep the same uuid.
                        // (This could be done more directly with a Hashmap entry update)
                        (uuid, conn),
                    );

                    self.danger_get_or_create_space(&data.space_address)
                        .get()
                        .join_agent(data.agent_id.clone(), uri.clone())?;
                    info!(
                        "Agent {:?} @ {} joined space {:?}",
                        data.agent_id, uri, data.space_address
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

    // removes an agent from a space
    fn leave(&mut self, uri: &Lib3hUri, data: &SpaceData) -> Sim2hResult<()> {
        with_latency_publishing!("sim2h-disconnnect", self.metric_publisher, || {
            if let Some((uuid, state)) = self.get_connection(uri) {
                conn_lifecycle("leave -> disconnect", &uuid, &state, uri);
                if let ConnectionState::Joined(space_address, agent_id) = state {
                    if (data.agent_id != agent_id) || (data.space_address != space_address) {
                        Err(SPACE_MISMATCH_ERR_STR.into())
                    } else {
                        self.disconnect(uri);
                        Ok(())
                    }
                } else {
                    Err(format!("no joined agent found at {} ", &uri).into())
                }
            } else {
                Err(format!("no agent found at {} ", &uri).into())
            }
        })
    }
    // removes a uri from connection and from spaces
    fn disconnect(&mut self, uri: &Lib3hUri) {
        with_latency_publishing!("sim2h-disconnnect", self.metric_publisher, || {
            trace!("disconnect entered");

            self.send_wss_command.f_send(WssCommand::CloseConnection(
                url::Url::from(uri.clone()).into(),
            ));

            if let Some((uuid, conn)) = self.state.danger_lock().connections.remove(uri) {
                conn_lifecycle("disconnect", &uuid, &conn, uri);
                if let ConnectionState::Joined(space_address, agent_id) = conn {
                    if let Some(space) = self.state.danger_lock().spaces.get_mut(&space_address) {
                        if space.remove_agent(&agent_id) == 0 {
                            self.state.danger_lock().spaces.remove(&space_address);
                        }
                    }
                }
            }
            trace!("disconnect done");
        })
    }

    // get the connection status of an agent
    fn get_connection(&self, uri: &Lib3hUri) -> Option<ConnectionStateItem> {
        with_latency_publishing!("sim2h-get_connection", self.metric_publisher, || {
            self.state
                .danger_lock()
                .connections
                .get(uri)
                .map(|ca| (*ca).clone())
        })
    }

    // find out if an agent is in a space or not and return its URI
    fn lookup_joined(&self, space_address: &SpaceHash, agent_id: &AgentId) -> Option<Lib3hUri> {
        with_latency_publishing!(
            "sim2h-handle_incoming_connect",
            self.metric_publisher,
            || {
                self.state
                    .danger_lock()
                    .spaces
                    .get(space_address)?
                    .agent_id_to_uri(agent_id)
            }
        )
    }

    // handler for incoming connections
    fn handle_incoming_connect(&mut self, uri: Lib3hUri) -> Sim2hResult<bool> {
        with_latency_publishing!(
            "sim2h-handle_incoming_connect",
            self.metric_publisher,
            || {
                trace!("handle_incoming_connect entered");
                debug!("New connection from {:?}", uri);
                if let Some(_old) = self
                    .state
                    .danger_lock()
                    .connections
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
            if message == WireMessage::Status {
                debug!("Sending StatusResponse in response to Status");
                let (spaces, connections) = {
                    let l = self.state.danger_lock();
                    (l.spaces.len(), l.connections.len())
                };
                self.send(
                    signer.clone(),
                    uri.clone(),
                    &WireMessage::StatusResponse(StatusData {
                        spaces,
                        connections,
                        redundant_count: match self.dht_algorithm {
                            DhtAlgorithm::FullSync => 0,
                            DhtAlgorithm::NaiveSharding { redundant_count } => redundant_count,
                        },
                        version: WIRE_VERSION,
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
                        // TODO: maybe have some upper limit on the number of messages
                        // we allow to queue before dropping the connections
                        pending_messages.push(message);
                        let _ = self
                            .state
                            .danger_lock()
                            .connections
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
        with_latency_publishing!("sim2h-process", self.metric_publisher, || {
            self.num_ticks += 1;
            if self.num_ticks % 60000 == 0 {
                debug!(".");
                self.num_ticks = 0;
            }

            self.priv_check_wss_events();

            if std::time::Instant::now() >= self.missing_aspects_resync {
                self.missing_aspects_resync = std::time::Instant::now()
                    .checked_add(std::time::Duration::from_millis(
                        RETRY_FETCH_MISSING_ASPECTS_INTERVAL_MS,
                    ))
                    .expect("can add interval ms");

                self.retry_sync_missing_aspects();
            }

            Ok(())
        })
    }

    fn handle_unseen_aspects(
        &mut self,
        uri: &Lib3hUri,
        space_address: &SpaceHash,
        agent_id: &AgentId,
        list_data: &EntryListData,
    ) {
        with_latency_publishing!("sim2h-handle-unseen_aspects", self.metric_publisher, || {
            let unseen_aspects = AspectList::from(HashMap::from(&list_data.address_map)).diff(
                self.danger_get_or_create_space(space_address)
                    .get()
                    .all_aspects(),
            );
            debug!("UNSEEN ASPECTS:\n{}", unseen_aspects.pretty_string());
            for entry_address in unseen_aspects.entry_addresses() {
                if let Some(aspect_address_list) = unseen_aspects.per_entry(entry_address) {
                    let wire_message = WireMessage::Lib3hToClient(Lib3hToClient::HandleFetchEntry(
                        FetchEntryData {
                            request_id: "".into(),
                            space_address: space_address.clone(),
                            provider_agent_id: agent_id.clone(),
                            entry_address: entry_address.clone(),
                            aspect_address_list: Some(aspect_address_list.clone()),
                        },
                    ));
                    self.send(agent_id.clone(), uri.clone(), &wire_message);
                }
            }
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
                        .danger_get_or_create_space(&space_address)
                        .get()
                        .all_aspects()
                        .diff(&AspectList::from(HashMap::from(list_data.address_map))),
                    DhtAlgorithm::NaiveSharding {redundant_count} => self
                        .danger_get_or_create_space(&space_address)
                        .get()
                        .aspects_in_shard_for_agent(agent_id, redundant_count)
                        .diff(&AspectList::from(HashMap::from(list_data.address_map)))
                };

                if aspects_missing_at_node.entry_addresses().count() > 0 {
                    warn!("MISSING ASPECTS at {}:\n{}", agent_id, aspects_missing_at_node.pretty_string());

                    // Cache info about what this agent is missing so we can make sure it got it
                    let missing_hashes: HashSet<(EntryHash, AspectHash)> = (&aspects_missing_at_node).into();
                    if missing_hashes.len() > 0 {
                        let mut space = self
                            .danger_get_or_create_space(&space_address);
                        for (entry_hash, aspect_hash) in missing_hashes {
                            space.get().add_missing_aspect(agent_id.clone(), entry_hash, aspect_hash);
                        }
                    }

                    match dht_algorithm {

                        DhtAlgorithm::FullSync => {
                            let all_agents_in_space = self
                                .danger_get_or_create_space(&space_address)
                                .get()
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
                                    .danger_get_or_create_space(&space_address)
                                    .get()
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
                            .danger_get_or_create_space(&space_address)
                            .get()
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
                        .danger_get_or_create_space(&space_address)
                        .get()
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
                                    .danger_get_or_create_space(&space_address)
                                    .get()
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
        })
    }

    fn fetch_aspects_from_arbitrary_agent(
        &mut self,
        aspects_to_fetch: AspectList,
        for_agent_id: AgentId,
        mut agent_pool: Vec<AgentId>,
        space_address: SpaceHash,
    ) {
        with_latency_publishing!(
            "sim2h-fetch_aspects_from_arbitrary_agent",
            self.metric_publisher,
            || {
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

                            let wire_message = WireMessage::Lib3hToClient(
                                Lib3hToClient::HandleFetchEntry(FetchEntryData {
                                    request_id: for_agent_id.clone().into(),
                                    space_address: space_address.clone(),
                                    provider_agent_id: arbitrary_agent.clone(),
                                    entry_address: entry_address.clone(),
                                    aspect_address_list: Some(aspect_address_list.clone()),
                                }),
                            );
                            debug!("SENDING fetch with request ID: {:?}", wire_message);
                            self.send(arbitrary_agent.clone(), random_url.clone(), &wire_message);
                        } else {
                            warn!("Could not find an agent that has any of the missing aspects. Trying again later...")
                        }
                    }
                }
            }
        )
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
        with_latency_publishing!(
            "sim2h-get_agent_not_missing_aspects",
            self.metric_publisher,
            || {
                let space = self.state.danger_lock();
                let space = space.spaces.get(space_address)?;
                agent_pool
                    .into_iter()
                    // We ignore all agents that are missing all of the same aspects as well since
                    // they can't help us.
                    .find(|a| {
                        **a != *for_agent_id
                            && !space.agent_is_missing_all_aspects(*a, entry_hash, aspects)
                    })
                    .cloned()
            }
        )
    }

    fn handle_new_entry_data(
        &mut self,
        entry_data: EntryData,
        space_address: SpaceHash,
        provider: AgentPubKey,
    ) {
        with_latency_publishing!("sim2h-handle_new_entry_data", self.metric_publisher, || {
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
                    let mut space = self.danger_get_or_create_space(&space_address);
                    space.get().add_aspect(
                        entry_data.entry_address.clone(),
                        aspect.aspect_address.clone(),
                    );
                    debug!(
                        "Space {} now knows about these aspects:\n{}",
                        &space_address,
                        space.get().all_aspects().pretty_string()
                    );
                }

                // 2. Create store message
                let store_message = WireMessage::Lib3hToClient(
                    Lib3hToClient::HandleStoreEntryAspect(StoreEntryAspectData {
                        request_id: "".into(),
                        space_address: space_address.clone(),
                        provider_agent_id: provider.clone(),
                        entry_address: entry_data.entry_address.clone(),
                        entry_aspect: aspect,
                    }),
                );

                // 3. Send store message to selected nodes
                self.broadcast(&store_message, dht_agents.clone());
            }
        })
    }

    fn broadcast(&mut self, msg: &WireMessage, agents: Vec<(AgentId, AgentInfo)>) {
        with_latency_publishing!("sim2h-broadcast", self.metric_publisher, || {
            for (agent, info) in agents {
                debug!("Broadcast: Sending to {:?}", info.uri);
                self.send(agent, info.uri, msg);
            }
        })
    }

    fn all_agents_except_one(
        &mut self,
        space: SpaceHash,
        except: Option<&AgentId>,
    ) -> Vec<(AgentId, AgentInfo)> {
        with_latency_publishing!("sim2h-all_agents_except_one", self.metric_publisher, || {
            self.danger_get_or_create_space(&space)
                .get()
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
        })
    }

    fn agents_in_neighbourhood(
        &mut self,
        space: SpaceHash,
        entry_loc: Location,
        redundant_count: u64,
    ) -> Vec<(AgentId, AgentInfo)> {
        with_latency_publishing!(
            "sim2h-agents_in_neighbourhood",
            self.metric_publisher,
            || {
                self.danger_get_or_create_space(&space)
                    .get()
                    .agents_supposed_to_hold_entry(entry_loc, redundant_count)
                    .into_iter()
                    .collect::<Vec<(AgentId, AgentInfo)>>()
            }
        )
    }

    fn send(&mut self, agent: AgentId, uri: Lib3hUri, msg: &WireMessage) {
        with_latency_publishing!("sim2h-send", self.metric_publisher, || {
            match msg {
                _ => {
                    debug!(">>OUT>> {} to {}", msg.message_type(), uri);
                    MESSAGE_LOGGER
                        .lock()
                        .log_out(agent, uri.clone(), msg.clone());
                }
            }

            let payload: Opaque = msg.clone().into();

            if !self.state.danger_lock().connections.contains_key(&uri) {
                error!("FAILED TO SEND, NO ROUTE: {}", uri);
                return;
            }

            self.send_wss_command.f_send(WssCommand::SendMessage(
                url::Url::from(uri).into(),
                WsFrame::Binary(payload.as_bytes()),
            ));

            match msg {
                WireMessage::Ping | WireMessage::Pong => {}
                _ => debug!("sent."),
            }
        })
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
                    .state
                    .danger_lock()
                    .spaces
                    .iter()
                    .filter_map(|(space_hash, space)| {
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
