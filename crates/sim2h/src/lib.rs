#![feature(vec_remove_item)]
#![feature(label_break_value)]
#![feature(proc_macro_hygiene)]
#![allow(clippy::redundant_clone)]

extern crate backtrace;
extern crate env_logger;
extern crate lib3h_crypto_api;
extern crate log;
extern crate nanoid;
extern crate num_cpus;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate lazy_static;
extern crate holochain_tracing as ht;
#[macro_use]
extern crate holochain_tracing_macros;
extern crate newrelic;

#[macro_use]
extern crate holochain_common;

#[allow(dead_code)]
mod naive_sharding;
#[allow(dead_code)]
mod schedule;
#[allow(unused_imports)]
use schedule::*;
pub mod connection_state;
pub mod crypto;
pub mod error;
use lib3h_protocol::types::*;
mod message_log;
pub mod websocket;
pub mod wire_message;

pub use crate::message_log::MESSAGE_LOGGER;
use crate::{crypto::*, error::*, naive_sharding::entry_location};
use connection_state::*;
use lib3h_crypto_api::CryptoSystem;
use lib3h_protocol::{data_types::*, protocol::*, types::SpaceHash, uri::Lib3hUri};

pub use wire_message::{
    HelloData, StatusData, WireError, WireMessage, WireMessageVersion, WIRE_VERSION,
};

use futures::{
    future::{BoxFuture, FutureExt},
    stream::StreamExt,
};
use in_stream::*;
use log::*;
use rand::{seq::SliceRandom, thread_rng};
use std::{
    convert::TryFrom,
    hash::{Hash, Hasher},
};

use holochain_locksmith::Mutex;
use holochain_metrics::{config::MetricPublisherConfig, Metric};
use tracing_futures::Instrument;

lazy_static! {
    static ref SET_THREAD_PANIC_FATAL: bool = {
        let orig_handler = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            eprintln!("THREAD PANIC {:#?}", panic_info);
            // invoke the default handler and exit the process
            orig_handler(panic_info);
            std::process::exit(1);
        }));
        true
    };
}

/// if we can't acquire a lock in 20 seconds, panic!
const MAX_LOCK_TIMEOUT: u64 = 20000;
pub const RECEIPT_HASH_SEED: u64 = 0;

//set up license_key
new_relic_setup!("NEW_RELIC_LICENSE_KEY");

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
        if let Err(e) = self.send(v) {
            error!("failed to send on channel -- shutting down? {:?}", e);
        }
    }
}

//const RETRY_FETCH_MISSING_ASPECTS_INTERVAL_MS: u64 = 30000; // 30 seconds
/// actual timing is handled by sim2h_im_state
/// but it does cause a mutate, so we don't want to spam it too hard
const RETRY_FETCH_MISSING_ASPECTS_INTERVAL_MS: u64 = 500; // half second

fn open_lifecycle(desc: &str, uuid: &str, uri: &Lib3hUri) {
    debug!("connection event open_conns: {} for {}@{}", desc, uuid, uri);
}

#[derive(Clone)]
struct MetricsTimerGenerator {
    sender: tokio::sync::mpsc::UnboundedSender<(&'static str, f64)>,
}

impl MetricsTimerGenerator {
    pub fn new() -> (Self, BoxFuture<'static, ()>) {
        let (sender, mut recv) = tokio::sync::mpsc::unbounded_channel::<(&'static str, f64)>();
        let out = async move {
            let metric_publisher = MetricPublisherConfig::default().create_metric_publisher();
            'metric_loop: loop {
                let msg = match recv.next().await {
                    None => break 'metric_loop,
                    Some(msg) => msg,
                };
                // TODO - this write is technically blocking
                //        move to spawn_blocking?? use tokio::sync::Mutex??
                metric_publisher
                    .write()
                    .unwrap()
                    .publish(&Metric::new_timestamped_now(msg.0, None, msg.1));
            }
            warn!("metric loop ended");
        }
        .boxed();
        (Self { sender }, out)
    }

    pub fn timer(&self, tag: &'static str) -> MetricsTimer {
        MetricsTimer::new(tag, self.sender.clone())
    }
}

struct MetricsTimer {
    tag: &'static str,
    create_time: std::time::Instant,
    sender: tokio::sync::mpsc::UnboundedSender<(&'static str, f64)>,
}

impl MetricsTimer {
    pub fn new(
        tag: &'static str,
        sender: tokio::sync::mpsc::UnboundedSender<(&'static str, f64)>,
    ) -> Self {
        Self {
            tag,
            create_time: std::time::Instant::now(),
            sender,
        }
    }
}

impl Drop for MetricsTimer {
    fn drop(&mut self) {
        let elapsed = self.create_time.elapsed().as_millis() as f64;
        if elapsed >= 1000.0 {
            error!("VERY SLOW metric - {} - {} ms", self.tag, elapsed);
        } else if elapsed >= 100.0 {
            warn!("SLOW metric - {} - {} ms", self.tag, elapsed);
        } else if elapsed >= 10.0 {
            info!("metric - {} - {} ms", self.tag, elapsed);
        }
        if let Err(e) = self.sender.send((self.tag, elapsed)) {
            error!(
                "failed to send metric - shutting down? {} {:?}",
                self.tag, e
            );
        }
    }
}

//pub(crate) type TcpWssServer = InStreamListenerWss<InStreamListenerTls<InStreamListenerTcp>>;
//pub(crate) type TcpWss = InStreamWss<InStreamTls<InStreamTcp>>;
pub(crate) type TcpWssServer = InStreamListenerWss<InStreamListenerTcp>;
pub type TcpWss = InStreamWss<InStreamTcp>;

mod connection_mgr;
use connection_mgr::*;

#[derive(Clone)]
pub enum DhtAlgorithm {
    FullSync,
    NaiveSharding { redundant_count: u64 },
}

#[allow(dead_code)]
mod mono_ref;
use mono_ref::*;
use twox_hash::XxHash64;

#[allow(dead_code)]
mod sim2h_im_state;

#[derive(Clone)]
/// A clonable reference to our Sim2h instance that can be passed
/// into `'static` async blocks && still be able to make sim2h calls
struct Sim2hHandle {
    state: sim2h_im_state::StoreHandle,
    dht_algorithm: DhtAlgorithm,
    metric_gen: MetricsTimerGenerator,
    connection_mgr: ConnectionMgrHandle,
    connection_count: ConnectionCount,
    tracer: Option<ht::Tracer>,
}

impl Sim2hHandle {
    pub fn new(
        crypto: Box<dyn CryptoSystem>,
        dht_algorithm: DhtAlgorithm,
        metric_gen: MetricsTimerGenerator,
        connection_mgr: ConnectionMgrHandle,
        connection_count: ConnectionCount,
        tracer: Option<ht::Tracer>,
    ) -> Self {
        let redundancy = match dht_algorithm {
            DhtAlgorithm::FullSync => 0,
            DhtAlgorithm::NaiveSharding { redundant_count } => redundant_count,
        };
        Self {
            state: sim2h_im_state::Store::new(crypto, redundancy, None),
            dht_algorithm,
            metric_gen,
            connection_mgr,
            connection_count,
            tracer,
        }
    }

    /// generate a new metrics timer
    pub fn metric_timer(&self, tag: &'static str) -> MetricsTimer {
        self.metric_gen.timer(tag)
    }

    /// get our current dht algorithm
    pub fn dht_algorithm(&self) -> &DhtAlgorithm {
        &self.dht_algorithm
    }

    /// access our connection manager handle
    pub fn connection_mgr(&self) -> &ConnectionMgrHandle {
        &self.connection_mgr
    }

    /// send a message to another connected agent
    pub fn send(&self, agent: AgentId, uri: Lib3hUri, msg: &WireMessage) {
        debug!(">>OUT>> {} to {}", msg.message_type(), uri);
        MESSAGE_LOGGER
            .lock()
            .log_out(agent, uri.clone(), msg.clone());
        let payload: Opaque = msg.clone().into();
        self.connection_mgr
            .send_data(uri, payload.as_bytes().into());
    }

    /// get access to our im_state object
    pub fn state(&self) -> &sim2h_im_state::StoreHandle {
        &self.state
    }

    /// forward a message to be handled
    pub fn handle_message(&self, uri: Lib3hUri, message: WireMessage, signer: AgentId) {
        let context = message
            .try_get_span()
            .and_then(|spans| spans.get(0).map(|s| (*s).to_owned()))
            .and_then(|span| ht::SpanContext::decode(span).ok());
        let follow = ht::follow_span!("follower", context);
        let _g = follow.enter();

        tracing::info!("testing");
        let span = tracing::info_span!("inner span");
        let _guard = span.enter();
        tracing::info!("testing again");

        // dispatch to correct handler
        let sim2h_handle = self.clone();

        // these message types are allowed before joining
        let message = match message {
            WireMessage::Lib3hToClient(_) | WireMessage::ClientToLib3hResponse(_) => {
                error!("This is soo wrong. Clients should never send a message that only servers can send.");
                return;
            }
            WireMessage::Ping => return spawn_handle_message_ping(sim2h_handle, uri, signer),
            WireMessage::Status => return spawn_handle_message_status(sim2h_handle, uri, signer),
            WireMessage::Hello(version) => {
                return spawn_handle_message_hello(sim2h_handle, uri, signer, version)
            }
            WireMessage::ClientToLib3h(ht::EncodedSpanWrap {
                data: ClientToLib3h::JoinSpace(data),
                ..
            }) => {
                let _ = tokio::task::spawn(spawn_handle_message_join_space(
                    sim2h_handle,
                    uri,
                    signer,
                    data,
                ));
                return;
            }
            message @ _ => message,
        };

        // you have to be in a space to proceed further
        let tracer = self.tracer.clone().unwrap_or_else(|| ht::null_tracer());
        tokio::task::spawn(async move {
            // -- right now each agent can only be part of a single space :/ --

            let (agent_id, space_hash) = 'got_info: {
                for _ in 0_usize..600 {
                    // await consistency of new connection
                    let state = sim2h_handle.state().get_clone().await;
                    if let Some(info) = state.get_space_info_from_uri(&uri) {
                        break 'got_info info;
                    }
                    tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
                }
                let s = tracing::error_span!("uri_error");
                let _g = s.enter();
                tracing::error!(?message, ?uri, ?signer);
                error!("uri has not joined space, cannot proceed {}", uri);
                return;
            };

            if *agent_id != signer {
                error!(
                    "signer {} does not match joined agent {:?}",
                    signer, agent_id
                );
                return;
            }

            match message {
                WireMessage::ClientToLib3h(span_wrap) => {
                    let span = ht::SpanWrap::from(span_wrap.clone())
                        .follower(&tracer, "handle_joined - ClientToLib3h");
                    let _spanguard = span.map(|span| ht::push_span(span));
                    match span_wrap.data.clone() {
                        ClientToLib3h::LeaveSpace(_data) => {
                            // for now, just disconnect on LeaveSpace
                            sim2h_handle.disconnect(vec![uri.clone()]);
                            return;
                        }
                        ClientToLib3h::SendDirectMessage(dm_data) => {
                            return spawn_handle_message_send_dm(
                                sim2h_handle,
                                uri,
                                signer,
                                space_hash,
                                span_wrap.swapped(dm_data),
                            );
                        }
                        ClientToLib3h::PublishEntry(data) => {
                            return spawn_handle_message_publish_entry(
                                sim2h_handle,
                                uri,
                                signer,
                                space_hash,
                                span_wrap.swapped(data),
                            );
                        }
                        ClientToLib3h::QueryEntry(query_data) => {
                            return spawn_handle_message_query_entry(
                                sim2h_handle,
                                uri,
                                signer,
                                space_hash,
                                query_data,
                            );
                        }
                        message @ _ => {
                            error!("unhandled message type {:?}", message);
                            return;
                        }
                    }
                }
                WireMessage::Lib3hToClientResponse(span_wrap) => {
                    let span = ht::SpanWrap::from(span_wrap.clone())
                        .follower(&tracer, "handle_joined - Lib3hToClientResponse");
                    let _spanguard = span.map(|span| ht::push_span(span));
                    match span_wrap.data.clone() {
                        Lib3hToClientResponse::HandleSendDirectMessageResult(dm_data) => {
                            return spawn_handle_message_send_dm_result(
                                sim2h_handle,
                                uri,
                                signer,
                                space_hash,
                                span_wrap.swapped(dm_data),
                            );
                        }
                        Lib3hToClientResponse::HandleGetAuthoringEntryListResult(list_data) => {
                            spawn_handle_message_list_data(
                                sim2h_handle.clone(),
                                uri.clone(),
                                signer.clone(),
                                space_hash.clone(),
                                list_data.clone(),
                            );
                            spawn_handle_message_authoring_entry_list(
                                sim2h_handle,
                                uri,
                                signer,
                                space_hash,
                                span_wrap.swapped(list_data),
                            );
                            return;
                        }
                        Lib3hToClientResponse::HandleGetGossipingEntryListResult(list_data) => {
                            return spawn_handle_message_list_data(
                                sim2h_handle,
                                uri,
                                signer,
                                space_hash,
                                list_data,
                            );
                        }
                        Lib3hToClientResponse::HandleFetchEntryResult(fetch_result) => {
                            return spawn_handle_message_fetch_entry_result(
                                sim2h_handle,
                                uri,
                                signer,
                                space_hash,
                                span_wrap.swapped(fetch_result),
                            );
                        }
                        Lib3hToClientResponse::HandleQueryEntryResult(query_result) => {
                            return spawn_handle_message_query_entry_result(
                                sim2h_handle,
                                uri,
                                signer,
                                space_hash,
                                span_wrap.swapped(query_result),
                            );
                        }
                        message @ _ => {
                            error!("unhandled message type {:?}", message);
                            return;
                        }
                    }
                }
                message @ _ => {
                    error!("unhandled message type {:?}", message);
                    return;
                }
            }
        });
    }

    /// disconnect an active connection
    pub fn disconnect(&self, disconnect: Vec<Lib3hUri>) {
        for d in disconnect.iter() {
            self.state().spawn_drop_connection_by_uri(d.clone());
            self.connection_mgr.disconnect(d.clone());
        }
    }
}

fn spawn_handle_message_ping(sim2h_handle: Sim2hHandle, uri: Lib3hUri, signer: AgentId) {
    /*
    tokio::task::spawn(async move {
    });
    */
    // no processing here, don't bother actually spawning
    debug!("Sending Pong in response to Ping");
    sim2h_handle.send(signer, uri, &WireMessage::Pong);
}

fn spawn_handle_message_status(sim2h_handle: Sim2hHandle, uri: Lib3hUri, signer: AgentId) {
    tokio::task::spawn(async move {
        debug!("Sending StatusResponse in response to Status");
        let state = sim2h_handle.state().get_clone().await;
        let mut joined_connections = 0_usize;
        for (_, space) in state.spaces.iter() {
            joined_connections += space.connections.len();
        }
        sim2h_handle.send(
            signer.clone(),
            uri.clone(),
            &WireMessage::StatusResponse(StatusData {
                spaces: state.spaces_count(),
                connections: sim2h_handle.connection_count.get().await,
                joined_connections,
                redundant_count: match sim2h_handle.dht_algorithm() {
                    DhtAlgorithm::FullSync => 0,
                    DhtAlgorithm::NaiveSharding { redundant_count } => *redundant_count,
                },
                version: WIRE_VERSION,
            }),
        );
    });
}

fn spawn_handle_message_hello(
    sim2h_handle: Sim2hHandle,
    uri: Lib3hUri,
    signer: AgentId,
    version: u32,
) {
    /*
    tokio::task::spawn(async move {
    });
    */
    // no processing here, don't bother actually spawning
    debug!("Sending HelloResponse in response to Hello({})", version);
    sim2h_handle.send(
        signer,
        uri.clone(),
        &WireMessage::HelloResponse(HelloData {
            redundant_count: match sim2h_handle.dht_algorithm() {
                DhtAlgorithm::FullSync => 0,
                DhtAlgorithm::NaiveSharding { redundant_count } => *redundant_count,
            },
            version: WIRE_VERSION,
            extra: None,
        }),
    );
    // versions do not match - disconnect them
    if version != WIRE_VERSION {
        warn!(
            "Disconnecting client for bad version this WIRE_VERSION = {}, client WIRE_VERSION = {}",
            WIRE_VERSION, version
        );
        sim2h_handle.disconnect(vec![uri]);
    }
}

async fn spawn_handle_message_join_space(
    sim2h_handle: Sim2hHandle,
    uri: Lib3hUri,
    _signer: AgentId,
    data: SpaceData,
) {
    sim2h_handle
        .state()
        .spawn_new_connection(
            data.space_address.clone(),
            data.agent_id.clone(),
            uri.clone(),
        )
        .await;

    sim2h_handle.send(
        data.agent_id.clone(),
        uri.clone(),
        &WireMessage::Lib3hToClient(
            ht::top_follower("request_gossiping_list")
                .wrap(Lib3hToClient::HandleGetGossipingEntryList(GetListData {
                    request_id: "".into(),
                    space_address: data.space_address.clone(),
                    provider_agent_id: data.agent_id.clone(),
                }))
                .into(),
        ),
    );

    let span = tracing::info_span!("Out");
    let id = span.id();
    let _g = span.enter();
    let span_wrap: ht::EncodedSpanWrap<()> = id
        .and_then(|id| ht::tracing::span_context(&id).map(|context| context.wrap(())))
        .unwrap_or_else(|| ht::wrap((), "No context".into()))
        .into();
    sim2h_handle.send(
        data.agent_id.clone(),
        uri,
        &WireMessage::Lib3hToClient(
            span_wrap.swapped(Lib3hToClient::HandleGetAuthoringEntryList(GetListData {
                request_id: "".into(),
                space_address: data.space_address.clone(),
                provider_agent_id: data.agent_id,
            }))
            .into(),
        ),
    );
}

fn inner_spawn_handle_message_send_dmx(
    sim2h_handle: Sim2hHandle,
    to_agent_id: AgentId,
    data_space_hash: SpaceHash,
    space_hash: MonoRef<SpaceHash>,
    message: WireMessage,
) {
    if data_space_hash != *space_hash {
        error!(
            "space mismatch - agent is in {}, message is for {}",
            *space_hash, data_space_hash
        );
        return;
    }

    tokio::task::spawn(async move {
        let state = sim2h_handle.state().get_clone().await;
        let to_url = match state.lookup_joined(&space_hash, &to_agent_id) {
            Some(to_url) => to_url,
            None => {
                error!("unvalidated proxy agent {}", &to_agent_id);
                return;
            }
        };
        sim2h_handle.send(to_agent_id, to_url.clone(), &message);
    });
}

fn spawn_handle_message_send_dm(
    sim2h_handle: Sim2hHandle,
    _uri: Lib3hUri,
    _signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    span_wrap: ht::EncodedSpanWrap<DirectMessageData>,
) {
    // Avoid clone of data
    let (span_wrap, data) = {
        let s = span_wrap.swapped(());
        (s, span_wrap.data)
    };
    let to_agent_id = data.to_agent_id.clone();
    let data_space_hash = data.space_address.clone();
    inner_spawn_handle_message_send_dmx(
        sim2h_handle,
        to_agent_id,
        data_space_hash,
        space_hash,
        WireMessage::Lib3hToClient(span_wrap.swapped(Lib3hToClient::HandleSendDirectMessage(data))),
    );
}

fn spawn_handle_message_send_dm_result(
    sim2h_handle: Sim2hHandle,
    _uri: Lib3hUri,
    _signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    span_wrap: ht::EncodedSpanWrap<DirectMessageData>,
) {
    // Avoid clone of data
    let (span_wrap, data) = {
        let s = span_wrap.swapped(());
        (s, span_wrap.data)
    };
    let to_agent_id = data.to_agent_id.clone();
    let data_space_hash = data.space_address.clone();
    inner_spawn_handle_message_send_dmx(
        sim2h_handle,
        to_agent_id,
        data_space_hash,
        space_hash,
        WireMessage::Lib3hToClient(span_wrap.swapped(Lib3hToClient::SendDirectMessageResult(data))),
    );
}

fn spawn_handle_message_publish_entry(
    sim2h_handle: Sim2hHandle,
    _uri: Lib3hUri,
    signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    span_wrap: ht::EncodedSpanWrap<ProvidedEntryData>,
) {
    // Avoid clone of data
    let (span_wrap, data) = {
        let s = span_wrap.swapped(());
        (s, span_wrap.data)
    };
    if data.space_address != *space_hash {
        error!(
            "space mismatch - agent is in {}, message is for {}",
            *space_hash, data.space_address
        );
        return;
    }

    tokio::task::spawn(async move {
        let aspect_list: im::HashSet<AspectHash> = data
            .entry
            .aspect_list
            .iter()
            .map(|a| a.aspect_address.clone())
            .collect();
        let mut multi_message = Vec::new();
        for aspect in data.entry.aspect_list {
            multi_message.push(span_wrap.swapped(Lib3hToClient::HandleStoreEntryAspect(
                StoreEntryAspectData {
                    request_id: "".into(),
                    space_address: (&*space_hash).clone(),
                    provider_agent_id: signer.clone(),
                    entry_address: data.entry.entry_address.clone(),
                    entry_aspect: aspect,
                },
            )));
        }
        let multi_message = WireMessage::MultiSend(multi_message);

        let state = sim2h_handle.state().get_clone().await;
        let send_to =
            match state.get_agents_that_should_hold_entry(&space_hash, &data.entry.entry_address) {
                None => return,
                Some(send_to) => send_to,
            };

        for agent_id in send_to {
            if let Some(uri) = state.lookup_joined(&space_hash, &agent_id) {
                sim2h_handle.send((&*agent_id).clone(), uri.clone(), &multi_message);
            }
            sim2h_handle.state().spawn_agent_holds_aspects(
                (&*space_hash).clone(),
                (&*agent_id).clone(),
                data.entry.entry_address.clone(),
                aspect_list.clone(),
            );
        }
    });
}

fn spawn_handle_message_list_data(
    sim2h_handle: Sim2hHandle,
    _uri: Lib3hUri,
    signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    list_data: EntryListData,
) {
    if signer != list_data.provider_agent_id || list_data.space_address != *space_hash {
        error!(
            "space mismatch - agent is in {}, message is for {}",
            *space_hash, list_data.space_address
        );
        return;
    }

    //tokio::task::spawn(async move {
    //});
    // just iter/send we don't need a new task for this

    for (entry_hash, aspects) in list_data.address_map {
        sim2h_handle.state().spawn_agent_holds_aspects(
            (&*space_hash).clone(),
            signer.clone(),
            entry_hash,
            aspects.into(),
        );
    }
}

fn spawn_handle_message_authoring_entry_list(
    sim2h_handle: Sim2hHandle,
    uri: Lib3hUri,
    signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    span_wrap: ht::EncodedSpanWrap<EntryListData>,
) {
    // Avoid clone of data
    let (span_wrap, list_data) = {
        let s = span_wrap.swapped(());
        (s, span_wrap.data)
    };
    if signer != list_data.provider_agent_id || list_data.space_address != *space_hash {
        error!(
            "space mismatch - agent is in {}, message is for {}",
            *space_hash, list_data.space_address
        );
        return;
    }

    tokio::task::spawn(async move {
        let state = sim2h_handle.state().get_clone().await;

        let mut multi_message = Vec::new();

        for (entry_hash, aspects) in list_data.address_map {
            let mut aspect_list = Vec::new();

            for aspect in aspects {
                let agents_that_need_aspect =
                    state.get_agents_that_need_aspect(&space_hash, &entry_hash, &aspect);
                if !agents_that_need_aspect.is_empty() {
                    aspect_list.push(aspect.clone());
                }
            }

            if !aspect_list.is_empty() {
                multi_message.push(span_wrap.swapped(Lib3hToClient::HandleFetchEntry(
                    FetchEntryData {
                        request_id: "".to_string(),
                        space_address: (&*space_hash).clone(),
                        provider_agent_id: signer.clone(),
                        entry_address: entry_hash.clone(),
                        aspect_address_list: Some(aspect_list),
                    },
                )));
            }
        }

        if !multi_message.is_empty() {
            let multi_send = WireMessage::MultiSend(multi_message);
            sim2h_handle.send(signer, uri, &multi_send);
        }
    });
}

fn spawn_handle_message_fetch_entry_result(
    sim2h_handle: Sim2hHandle,
    _uri: Lib3hUri,
    signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    span_wrap: ht::EncodedSpanWrap<FetchEntryResultData>,
) {
    // Avoid cloning data
    let (span_wrap, fetch_result) = {
        let s = span_wrap.swapped(());
        (s, span_wrap.data)
    };
    if signer != fetch_result.provider_agent_id || fetch_result.space_address != *space_hash {
        error!(
            "space mismatch - agent is in {}, message is for {}",
            *space_hash, fetch_result.space_address
        );
        return;
    }

    tokio::task::spawn(async move {
        let state = sim2h_handle.state().get_clone().await;

        #[allow(clippy::type_complexity)]
        let mut to_agent: std::collections::HashMap<
            MonoRef<AgentId>,
            (
                Vec<ht::EncodedSpanWrap<Lib3hToClient>>,
                std::collections::HashMap<EntryHash, im::HashSet<AspectHash>>,
            ),
        > = std::collections::HashMap::new();

        for aspect in fetch_result.entry.aspect_list {
            let agents_that_need_aspect = state.get_agents_that_need_aspect(
                &space_hash,
                &fetch_result.entry.entry_address,
                &aspect.aspect_address.clone(),
            );

            for agent_id in agents_that_need_aspect {
                let m = to_agent.entry(agent_id.clone()).or_default();
                m.0.push(span_wrap.swapped(Lib3hToClient::HandleStoreEntryAspect(
                    StoreEntryAspectData {
                        request_id: "".into(),
                        space_address: (&*space_hash).clone(),
                        provider_agent_id: (&*agent_id).clone(),
                        entry_address: fetch_result.entry.entry_address.clone(),
                        entry_aspect: aspect.clone(),
                    },
                )));

                let e =
                    m.1.entry(fetch_result.entry.entry_address.clone())
                        .or_default();
                e.insert(aspect.aspect_address.clone());
            }
        }

        for (agent_id, (multi_message, mut holding)) in to_agent.drain() {
            let uri = match state.lookup_joined(&space_hash, &agent_id) {
                None => continue,
                Some(uri) => uri,
            };

            let multi_send = WireMessage::MultiSend(multi_message);

            sim2h_handle.send((&*agent_id).clone(), (&*uri).clone(), &multi_send);

            for (entry_hash, aspects) in holding.drain() {
                sim2h_handle.state().spawn_agent_holds_aspects(
                    (&*space_hash).clone(),
                    (&*agent_id).clone(),
                    entry_hash,
                    aspects,
                );
            }
        }
    });
}

fn spawn_handle_message_query_entry(
    sim2h_handle: Sim2hHandle,
    _uri: Lib3hUri,
    signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    query_data: QueryEntryData,
) {
    if signer != query_data.requester_agent_id || query_data.space_address != *space_hash {
        error!(
            "space mismatch - agent is in {}, message is for {}",
            *space_hash, query_data.space_address
        );
        return;
    }

    tokio::task::spawn(async move {
        let state = sim2h_handle.state().get_clone().await;

        let holding_agents = state.get_agents_for_query(
            &space_hash,
            &query_data.entry_address,
            Some(&query_data.requester_agent_id),
        );
        // TODO db - send it out to more than one node
        //           then give it some aggregation time
        let query_target = holding_agents[0].clone();

        let url = match state.lookup_joined(&space_hash, &query_target) {
            None => {
                error!("AHH - the query_target we found doesn't exist");
                return;
            }
            Some(url) => url,
        };

        let span = tracing::info_span!("Out qe", root = true);
        //let id = span.id();
        let _g = span.enter();
        let span = ht::top_follower("inner");
        let query_message = WireMessage::Lib3hToClient(
            span.wrap(Lib3hToClient::HandleQueryEntry(query_data))
                .into(),
        );
        sim2h_handle.send((&*query_target).clone(), url.clone(), &query_message);
    });
}

fn spawn_handle_message_query_entry_result(
    sim2h_handle: Sim2hHandle,
    _uri: Lib3hUri,
    signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    span_wrap: ht::EncodedSpanWrap<QueryEntryResultData>,
) {
    let (span_wrap, query_result) = {
        let s = span_wrap.swapped(());
        (s, span_wrap.data)
    };
    if signer != query_result.responder_agent_id || query_result.space_address != *space_hash {
        error!(
            "space mismatch - agent is in {}, message is for {}",
            *space_hash, query_result.space_address
        );
        return;
    }

    tokio::task::spawn(async move {
        let req_agent_id = query_result.requester_agent_id.clone();
        let msg_out = WireMessage::ClientToLib3hResponse(
            span_wrap.swapped(ClientToLib3hResponse::QueryEntryResult(query_result)),
        );
        let state = sim2h_handle.state().get_clone().await;
        let to_url = match state.lookup_joined(&space_hash, &req_agent_id) {
            Some(to_url) => to_url,
            None => {
                error!("unvalidated proxy agent {}", &req_agent_id);
                return;
            }
        };
        sim2h_handle.send(req_agent_id, to_url.clone(), &msg_out);
    });
}

/// creates a tokio runtime and executes the Sim2h instance within it
/// returns the runtime so the user can choose how to manage the main loop
pub fn run_sim2h(
    crypto: Box<dyn CryptoSystem>,
    bind_spec: Lib3hUri,
    dht_algorithm: DhtAlgorithm,
    tracer: Option<ht::Tracer>,
) -> (
    tokio::runtime::Runtime,
    tokio::sync::oneshot::Receiver<Lib3hUri>,
) {
    let rt = tokio::runtime::Builder::new()
        .enable_all()
        .threaded_scheduler()
        .core_threads(num_cpus::get())
        .thread_name("sim2h-tokio-thread")
        .build()
        .expect("can build tokio runtime");

    let (bind_send, bind_recv) = tokio::sync::oneshot::channel();

    rt.spawn(async move {
        let sim2h = Sim2h::new(crypto, bind_spec, dht_algorithm, tracer);
        let _ = bind_send.send(sim2h.bound_uri.clone().unwrap());

        /*
        tokio::task::spawn(async move {
            let mut listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("failed to bind");
            warn!("TT BOUND TO: {:?}", listener.local_addr());
            while let Ok((stream, addr)) = listener.accept().await {
                let stream: tokio::net::TcpStream = stream;
                tokio::task::spawn(async move {
                    warn!("GOT TT CONNECTION: {:?}", addr);
                    let ws_stream = tokio_tungstenite::accept_async(stream)
                        .await
                        .expect("failed to handshake websocket");
                    let (write, read) = ws_stream.split();
                    read.forward(write)
                        .await
                        .expect("failed to forward message")
                });
            }
        });
        */

        let gen_blocking_fn = move |mut sim2h: Sim2h| {
            move || {
                let res = sim2h.process();
                (sim2h, res)
            }
        };
        let mut blocking_fn = Some(gen_blocking_fn(sim2h));
        'sim2h_process_loop: loop {
            // NOTE - once we move everything in sim2h to futures
            //        we can get rid of the `process()` function
            //        and remove this spawn_blocking code
            let sim2h = match tokio::task::spawn_blocking(blocking_fn.take().unwrap()).await {
                Err(e) => {
                    // sometimes we get errors on shutdown...
                    // we can't recover because the sim2h instance is lost
                    // but don't panic... just exit
                    error!("sim2h process failed: {:?}", e);
                    break 'sim2h_process_loop;
                }
                Ok((sim2h, Err(e))) => {
                    if e.to_string().contains("Bind error:") {
                        println!("{:?}", e);
                        std::process::exit(1);
                    } else {
                        error!("{}", e.to_string())
                    }
                    sim2h
                }
                Ok((sim2h, Ok(did_work))) => {
                    if did_work {
                        tokio::task::yield_now().await;
                    } else {
                        tokio::time::delay_for(std::time::Duration::from_millis(1)).await;
                    }
                    sim2h
                }
            };
            blocking_fn = Some(gen_blocking_fn(sim2h));
        }
        warn!("sim2h process loop ended");
    });

    (rt, bind_recv)
}

/// a Sim2h server instance - manages connections between holochain instances
pub struct Sim2h {
    bound_listener: Option<TcpWssServer>,
    metric_task: Option<BoxFuture<'static, ()>>,
    pub bound_uri: Option<Lib3hUri>,
    wss_send: crossbeam_channel::Sender<TcpWss>,
    wss_recv: crossbeam_channel::Receiver<TcpWss>,
    connection_mgr_evt_recv: ConnectionMgrEventRecv,
    num_ticks: u64,
    /// when should we try to resync nodes that are still missing aspect data
    missing_aspects_resync_schedule: Schedule,
    sim2h_handle: Sim2hHandle,
    metric_gen: MetricsTimerGenerator,
}

#[autotrace]
#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
impl Sim2h {
    /// create a new Sim2h server instance
    pub fn new(
        crypto: Box<dyn CryptoSystem>,
        bind_spec: Lib3hUri,
        dht_algorithm: DhtAlgorithm,
        tracer: Option<ht::Tracer>,
    ) -> Self {
        // make sure if a thread panics, the whole process exits
        assert!(*SET_THREAD_PANIC_FATAL);

        let (metric_gen, metric_task) = MetricsTimerGenerator::new();

        let (connection_mgr, connection_mgr_evt_recv, connection_count) = ConnectionMgr::new();

        let (wss_send, wss_recv) = crossbeam_channel::unbounded();
        let sim2h_handle = Sim2hHandle::new(
            crypto.box_clone(),
            dht_algorithm,
            metric_gen.clone(),
            connection_mgr,
            connection_count,
            tracer,
        );

        let config = TcpBindConfig::default();
        //        let config = TlsBindConfig::new(config).dev_certificate();
        let config = WssBindConfig::new(config);
        let url = url::Url::from(bind_spec).into();
        let listen: TcpWssServer = InStreamListenerWss::bind(&url, config).unwrap();
        let bound_uri = Some(url::Url::from(listen.binding()).into());

        let sim2h = Sim2h {
            // TODO - (db) - Sim2h::new() is now called inside tokio runtime
            //               we can move these back into the constructor
            bound_listener: Some(listen),
            metric_task: Some(metric_task),
            bound_uri,
            wss_send,
            wss_recv,
            connection_mgr_evt_recv,
            num_ticks: 0,
            missing_aspects_resync_schedule: Schedule::new(std::time::Duration::from_millis(
                RETRY_FETCH_MISSING_ASPECTS_INTERVAL_MS,
            )),
            sim2h_handle,
            metric_gen,
        };

        // trigger an initial schedule ready event
        let _ = sim2h.missing_aspects_resync_schedule.get_guard();

        sim2h
    }

    /// if our listening socket has accepted any new connections, set them up
    fn priv_check_incoming_connections(&mut self) -> bool {
        let _m = self
            .metric_gen
            .timer("sim2h-priv_check_incoming_connections");

        let mut did_work = false;
        let mut wss_list = Vec::new();
        for _ in 0..100 {
            if let Ok(wss) = self.wss_recv.try_recv() {
                did_work = true;
                wss_list.push(wss);
            } else {
                break;
            }
        }
        if !wss_list.is_empty() {
            let sim2h_handle = self.sim2h_handle.clone();
            tokio::task::spawn(async move {
                let _m =
                    sim2h_handle.metric_timer("sim2h-priv_check_incoming_connections-async-add");

                for wss in wss_list.drain(..) {
                    let url: Lib3hUri = url::Url::from(wss.remote_url()).into();
                    let uuid = nanoid::simple();
                    open_lifecycle("adding conn job", &uuid, &url);

                    sim2h_handle.connection_mgr().connect(url, wss);
                }
            });
        }
        did_work
    }

    /// we received some kind of error related to a stream/socket
    /// print some debugging and disconnect it
    fn priv_drop_connection_for_error(&mut self, uri: Lib3hUri, error: Sim2hError) {
        debug!(
            "dropping connection to {} because of error: {:?}",
            uri, error,
        );
        self.sim2h_handle.disconnect(vec![uri]);
    }

    /// if our connections sent us any data, process it
    fn priv_check_incoming_messages(&mut self) -> bool {
        let _m = self.metric_gen.timer("sim2h-priv_check_incoming_messages");
        let mut did_work = false;

        let loop_start = std::time::Instant::now();
        let mut msg_count = 0;

        let mut disconnect = Vec::new();
        for _ in 0..100 {
            match self.connection_mgr_evt_recv.try_recv() {
                Ok(evt) => {
                    msg_count += 1;
                    did_work = true;
                    match evt {
                        ConMgrEvent::Disconnect(uri, maybe_err) => {
                            debug!("disconnect {} {:?}", uri, maybe_err);
                            disconnect.push(uri);
                        }
                        ConMgrEvent::ReceiveData(uri, data) => {
                            self.priv_handle_recv_data(uri, data);
                        }
                    }
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Closed) => {
                    error!("broken channel, shutting down?");
                    return true;
                }
            }
        }

        if !disconnect.is_empty() {
            self.sim2h_handle.disconnect(disconnect);
        }

        trace!(
            "sim2h lib processed {} incoming messages in {} ms",
            msg_count,
            loop_start.elapsed().as_millis(),
        );

        did_work
    }

    /// process an actual incoming message
    fn priv_handle_recv_data(&mut self, uri: Lib3hUri, data: WsFrame) {
        match data {
            WsFrame::Text(s) => self.priv_drop_connection_for_error(
                uri,
                format!("unexpected text message: {:?}", s).into(),
            ),
            WsFrame::Binary(b) => {
                trace!(
                    "priv_check_incoming_messages: received a frame from {}",
                    uri
                );
                let payload: Opaque = b.into();
                Sim2h::handle_payload(self.sim2h_handle.clone(), uri, payload);
            }
            // TODO - we should use websocket ping/pong
            //        instead of rolling our own on top of Binary
            WsFrame::Ping(_) => (),
            WsFrame::Pong(_) => (),
            WsFrame::Close(c) => {
                debug!("Disconnecting {} after connection reset {:?}", uri, c);
                self.sim2h_handle.disconnect(vec![uri]);
            }
        }
    }

    fn handle_payload(sim2h_handle: Sim2hHandle, url: Lib3hUri, payload: Opaque) {
        tokio::task::spawn(async move {
            let _m = sim2h_handle.metric_timer("sim2h-handle_payload");
            match (|| -> Sim2hResult<(AgentId, WireMessage)> {
                let signed_message = SignedWireMessage::try_from(payload.clone())?;
                let result = signed_message.verify().unwrap();
                if !result {
                    return Err(VERIFY_FAILED_ERR_STR.into());
                }
                let agent_id: AgentId = signed_message.provenance.source().into();
                send_receipt(
                    sim2h_handle.clone(),
                    &signed_message.payload,
                    agent_id.clone(),
                    url.clone(),
                );
                let wire_message = WireMessage::try_from(signed_message.payload)?;
                Ok((agent_id, wire_message))
            })() {
                Ok((source, wire_message)) => {
                    sim2h_handle.handle_message(url.clone(), wire_message, source.clone());
                }
                Err(error) => {
                    error!(
                        "Could not verify payload from {}!\nError: {:?}\nPayload was: {:?}",
                        url, error, payload
                    );
                    sim2h_handle.disconnect(vec![url]);
                }
            }
        });
    }

    /// process transport and incoming messages from it
    pub fn process(&mut self) -> Sim2hResult<bool> {
        let _m = self.metric_gen.timer("sim2h-process");
        if self.bound_listener.is_some() {
            let mut listen = self.bound_listener.take().unwrap();
            let wss_send = self.wss_send.clone();
            tokio::task::spawn(async move {
                loop {
                    let mut did_work = false;
                    for _ in 0..100 {
                        match listen.accept() {
                            Ok(wss) => {
                                wss_send.f_send(wss);
                                did_work = true;
                            }
                            Err(e) if e.would_block() => {
                                break;
                            }
                            Err(e) => {
                                error!(
                                    "LISTEN ACCEPT FAIL: {:?}\nbacktrace: {:?}",
                                    e,
                                    backtrace::Backtrace::new()
                                );
                                did_work = true;
                            }
                        }
                    }
                    if did_work {
                        tokio::task::yield_now().await;
                    } else {
                        tokio::time::delay_for(std::time::Duration::from_millis(10)).await;
                    }
                }
            });
        }
        if self.metric_task.is_some() {
            tokio::task::spawn(self.metric_task.take().unwrap());
        }

        let mut did_work = false;

        self.num_ticks += 1;
        if self.num_ticks % 60000 == 0 {
            debug!(".");
            self.num_ticks = 0;
        }

        if self.priv_check_incoming_connections() {
            did_work = true;
        }

        if self.priv_check_incoming_messages() {
            did_work = true;
        }

        if self.missing_aspects_resync_schedule.should_proceed() {
            let span = tracing::info_span!("missing aspect root", root = true);
            let _g = span.enter();
            let schedule_guard = self.missing_aspects_resync_schedule.get_guard();
            let sim2h_handle = self.sim2h_handle.clone();
            tokio::task::spawn(
                missing_aspects_resync(sim2h_handle, schedule_guard)
                    .instrument(tracing::info_span!("missing aspect future")),
            );
        }

        Ok(did_work)
    }
}

fn send_receipt(sim2h_handle: Sim2hHandle, payload: &Opaque, source: AgentId, url: Lib3hUri) {
    let mut hasher = XxHash64::with_seed(RECEIPT_HASH_SEED);
    payload.hash(&mut hasher);
    let hash = hasher.finish();
    let receipt = WireMessage::Ack(hash);
    sim2h_handle.send(source, url, &receipt);
}

async fn missing_aspects_resync(sim2h_handle: Sim2hHandle, _schedule_guard: ScheduleGuard) {
    let gossip_full_start = std::time::Instant::now();

    let agents_needing_gossip = sim2h_handle.state().check_gossip().await.spaces();

    if agents_needing_gossip.is_empty() {
        debug!("sim2h gossip no agents needing gossip");
    }

    for (space_hash, agents) in agents_needing_gossip.iter() {
        trace!(
            "sim2h gossip agent count: {} in space {:?}",
            agents.len(),
            space_hash
        );

        for agent_id in agents {
            // explicitly yield here as we don't want to hog the scheduler
            tokio::task::yield_now().await;
            let state = sim2h_handle.state().get_clone().await;

            let gossip_agent_start = std::time::Instant::now();

            let r = match state.get_gossip_aspects_needed_for_agent(&space_hash, &agent_id) {
                None => continue,
                Some(r) => r,
            };

            for (entry_hash, aspects) in r.iter() {
                if aspects.is_empty() {
                    continue;
                }

                let query_agents = state.get_agents_for_query(&space_hash, &entry_hash, None);

                if query_agents.is_empty() {
                    warn!(
                        "nobody online to service gossip request for aspects in entry hash {:?}",
                        entry_hash
                    );
                    continue;
                }

                // TODO - if we have multiple options,
                // do we want to fire off more than one?
                let query_agent = query_agents[0].clone();

                let uri = match state.lookup_joined(space_hash, &query_agent) {
                    None => continue,
                    Some(uri) => uri,
                };
                let span = tracing::info_span!("Out");
                let id = span.id();
                let _g = span.enter();
                let span_wrap: ht::EncodedSpanWrap<()> = id
                    .and_then(|id| ht::tracing::span_context(&id).map(|context| context.wrap(())))
                    .unwrap_or_else(|| ht::wrap((), "No context".into()))
                    .into();

                let wire_message = WireMessage::Lib3hToClient({
                    let s = FetchEntryData {
                        request_id: "".to_string(),
                        space_address: (&**space_hash).clone(),
                        provider_agent_id: (&*query_agent).clone(),
                        entry_address: (&**entry_hash).clone(),
                        aspect_address_list: Some(aspects.iter().map(|a| (&**a).clone()).collect()),
                    };
                    tracing::info!(message = "wire_message", ?s.request_id, ?s.space_address);
                    span_wrap.swapped(Lib3hToClient::HandleFetchEntry(s)).into()
                });

                sim2h_handle.send((&*query_agent).clone(), (&*uri).clone(), &wire_message);
            }
            trace!(
                "sim2h gossip agent in {} ms",
                gossip_agent_start.elapsed().as_millis()
            );
        }
    }
    trace!("sim2h gossip full loop in {} ms (ok to be long, this task is broken into multiple sub-loops)", gossip_full_start.elapsed().as_millis());
}
