#![feature(vec_remove_item)]
#![feature(label_break_value)]
#![allow(clippy::redundant_clone)]

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
use rand::{seq::SliceRandom, thread_rng};
use std::{
    convert::TryFrom,
    fs::File,
    hash::{Hash, Hasher},
    io::prelude::*,
};
use url2::prelude::*;

use holochain_common::new_relic_setup;
use holochain_locksmith::Mutex;
use holochain_metrics::{config::MetricPublisherConfig, Metric};
use holochain_tracing as ht;
use holochain_tracing_macros::{autotrace, newrelic_autotrace};
use lazy_static::lazy_static;
use sim2h_im_state::{MonoAspectHash, MonoEntryHash, StoreRef};
use tracing::*;
use tracing_futures::Instrument;

/// If we don't receive any messages from the remote end of a websocket
/// within 30 seconds, we assume the connection is dead, and clean it up.
/// Note, this setting also causes in_stream to send out a Ping every
/// 15 seconds (half the message timeout) just incase we don't have anything
/// else to say.
const NO_MESSAGE_CONNECTION_TIMEOUT_MS: u64 = 30000;

/// use the default 0 seed for xxHash
pub const RECEIPT_HASH_SEED: u64 = 0;

/// Generates a u64 hash response for an `Ack` message given input bytes
pub fn generate_ack_receipt_hash(payload: &Opaque) -> u64 {
    let mut hasher = XxHash64::with_seed(RECEIPT_HASH_SEED);
    payload.hash(&mut hasher);
    hasher.finish()
}

/// internal generate the full `Ack` message.
fn gen_receipt(payload: &Opaque) -> WireMessage {
    WireMessage::Ack(generate_ack_receipt_hash(payload))
}

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
use std::collections::BTreeMap;
use twox_hash::XxHash64;

 use std::collections::HashMap;

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
}

impl Sim2hHandle {
    pub fn new(
        crypto: Box<dyn CryptoSystem>,
        dht_algorithm: DhtAlgorithm,
        metric_gen: MetricsTimerGenerator,
        connection_mgr: ConnectionMgrHandle,
        connection_count: ConnectionCount,
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

    /// Notify core/sim2h_worker that we have processed the current message
    /// sufficiently, and are ready to receive another message.
    pub fn send_receipt(&self, receipt: &WireMessage, source: &AgentId, url: &Lib3hUri) {
        self.send(source.clone(), url.clone(), receipt);
    }

    /// forward a message to be handled
    pub fn handle_message(
        &self,
        uri: Lib3hUri,
        message: WireMessage,
        signer: AgentId,
        receipt: WireMessage,
    ) {
        let context = message
            .try_get_span()
            // Not using multi messages in this function so first is fine.
            .and_then(|spans| spans.first().cloned())
            .and_then(|context| ht::SpanContext::decode(context.clone()).ok());
        let follow = ht::follow_span!(Level::INFO, context);
        let _g = follow.enter();
        // The above follow span will not be reported to jaeger so it's helpful to create an inner follow
        let span = debug_span!("inner_message_follow");
        let _g = span.enter();
        debug!(received = ?message);

        // dispatch to correct handler
        let sim2h_handle = self.clone();

        // these message types are allowed before joining
        let message = match message {
            WireMessage::Lib3hToClient(_) | WireMessage::ClientToLib3hResponse(_) => {
                error!("This is soo wrong. Clients should never send a message that only servers can send.");
                return;
            }
            WireMessage::Ping => {
                return spawn_handle_message_ping(sim2h_handle, uri, signer, receipt)
            }
            WireMessage::Status => {
                return spawn_handle_message_status(sim2h_handle, uri, signer, receipt)
            }
            WireMessage::Debug => {
                return spawn_handle_message_debug(sim2h_handle, uri, signer, receipt)
            }
            WireMessage::Hello(version) => {
                return spawn_handle_message_hello(sim2h_handle, uri, signer, version, receipt)
            }
            WireMessage::ClientToLib3h(ht::EncodedSpanWrap {
                data: ClientToLib3h::JoinSpace(data),
                ..
            }) => {
                let _ = tokio::task::spawn(handle_message_join_space(
                    sim2h_handle,
                    uri,
                    signer,
                    data,
                    receipt,
                ));
                return;
            }
            message @ _ => message,
        };

        // you have to be in a space to proceed further
        tokio::task::spawn(async move {
            // -- right now each agent can only be part of a single space :/ --

            let (agent_id, space_hash) = {
                let state = sim2h_handle.state().get_clone().await;
                if let Some(info) = state.get_space_info_from_uri(&uri) {
                    info
                } else {
                    error!(
                        "uri has not joined space, cannot proceed {} {}",
                        uri,
                        message.message_type()
                    );
                    sim2h_handle.disconnect(vec![uri.clone()]);
                    return;
                }
            };

            if *agent_id != signer {
                error!(
                    "signer {} does not match joined agent {:?}",
                    signer, agent_id
                );
                return;
            }

            sim2h_handle.send_receipt(&receipt, &signer, &uri);

            match message {
                WireMessage::ClientToLib3h(ht::EncodedSpanWrap { data, .. }) => {
                    return client_to_lib3h(data, uri, sim2h_handle, signer, space_hash);
                }
                WireMessage::Lib3hToClientResponse(ht::EncodedSpanWrap { data, .. }) => {
                    return lib3h_to_client_response(data, uri, sim2h_handle, signer, space_hash);
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

#[instrument(skip(data, sim2h_handle))]
fn client_to_lib3h(
    data: ClientToLib3h,
    uri: Lib3hUri,
    sim2h_handle: Sim2hHandle,
    signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
) {
    match data {
        ClientToLib3h::LeaveSpace(_data) => {
            // for now, just disconnect on LeaveSpace
            sim2h_handle.disconnect(vec![uri.clone()]);
        }
        ClientToLib3h::SendDirectMessage(dm_data) => {
            return spawn_handle_message_send_dm(sim2h_handle, uri, signer, space_hash, dm_data);
        }
        ClientToLib3h::PublishEntry(data) => {
            return spawn_handle_message_publish_entry(sim2h_handle, uri, signer, space_hash, data);
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
        }
    }
}

#[instrument(skip(data, sim2h_handle))]
fn lib3h_to_client_response(
    data: Lib3hToClientResponse,
    uri: Lib3hUri,
    sim2h_handle: Sim2hHandle,
    signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
) {
    let span = debug_span!("Lib3hToClientResponse");
    let _g = span.enter();
    match data {
        Lib3hToClientResponse::HandleSendDirectMessageResult(dm_data) => {
            return spawn_handle_message_send_dm_result(
                sim2h_handle,
                uri,
                signer,
                space_hash,
                dm_data,
            );
        }
        Lib3hToClientResponse::HandleGetAuthoringEntryListResult(list_data) => {
            trace!("AUTHORING: list_data {:?}",list_data);
            // the author should always be holding it's own agent id so lets construct a holding
            // list for that and mark it as held
            let mut list_data1 = EntryListData {
                space_address: list_data.space_address.clone(),
                provider_agent_id: list_data.provider_agent_id.clone(),
                request_id: list_data.request_id.clone(),
                address_map: HashMap::new(),
            };
            let agent_hash = signer.to_string().into();
            if let Some(aspects) = list_data.address_map.get(&agent_hash) {
                list_data1.address_map.insert(agent_hash, aspects.clone());
            }
            spawn_handle_message_list_data(
                sim2h_handle.clone(),
                uri.clone(),
                signer.clone(),
                space_hash.clone(),
                list_data1,
            );
            spawn_handle_message_authoring_entry_list(
                sim2h_handle,
                uri,
                signer,
                space_hash,
                list_data,
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
                fetch_result,
            );
        }
        Lib3hToClientResponse::HandleQueryEntryResult(query_result) => {
            return spawn_handle_message_query_entry_result(
                sim2h_handle,
                uri,
                signer,
                space_hash,
                query_result,
            );
        }
        message @ _ => {
            error!("unhandled message type {:?}", message);
            return;
        }
    }
}

fn spawn_handle_message_ping(
    sim2h_handle: Sim2hHandle,
    uri: Lib3hUri,
    signer: AgentId,
    receipt: WireMessage,
) {
    /*
    tokio::task::spawn(async move {
    });
    */
    // no processing here, don't bother actually spawning
    debug!("Sending Pong in response to Ping");
    sim2h_handle.send(signer.clone(), uri.clone(), &WireMessage::Pong);
    sim2h_handle.send_receipt(&receipt, &signer, &uri);
}

fn spawn_handle_message_status(
    sim2h_handle: Sim2hHandle,
    uri: Lib3hUri,
    signer: AgentId,
    receipt: WireMessage,
) {
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
        sim2h_handle.send_receipt(&receipt, &signer, &uri);
    });
}

fn spawn_handle_message_debug(
    sim2h_handle: Sim2hHandle,
    uri: Lib3hUri,
    signer: AgentId,
    receipt: WireMessage,
) {
    tokio::task::spawn(async move {
        debug!("Sending DebugResponse in response to Debug");
        let state = sim2h_handle.state().get_clone().await;
        let mut response_map: BTreeMap<SpaceHash, String> = BTreeMap::new();
        for (hash, space) in state.spaces.iter() {
            let json = serde_json::to_string(&space).expect("Space must be serializable");
            response_map.insert((**hash).clone(), json.clone());
            let filename = format!("{}.json", **hash);
            if let Ok(mut file) = File::create(filename.clone()) {
                file.write_all(json.into_bytes().as_slice())
                    .unwrap_or_else(|_| error!("Could not write to file {}!", filename))
            } else {
                error!("Could not create file {}!", filename)
            }
        }
        let connection_list = sim2h_handle.connection_mgr().list_connections().await;
        let extra_data = format!("LIST_CONNECTIONS: {:#?}", connection_list);
        sim2h_handle.send(
            signer.clone(),
            uri.clone(),
            &WireMessage::DebugResponse((response_map, extra_data)),
        );
        sim2h_handle.send_receipt(&receipt, &signer, &uri);
    });
}

fn spawn_handle_message_hello(
    sim2h_handle: Sim2hHandle,
    uri: Lib3hUri,
    signer: AgentId,
    version: u32,
    receipt: WireMessage,
) {
    /*
    tokio::task::spawn(async move {
    });
    */
    // no processing here, don't bother actually spawning
    debug!("Sending HelloResponse in response to Hello({})", version);
    sim2h_handle.send(
        signer.clone(),
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
    sim2h_handle.send_receipt(&receipt, &signer, &uri);
    // versions do not match - disconnect them
    if version != WIRE_VERSION {
        warn!(
            "Disconnecting client for bad version this WIRE_VERSION = {}, client WIRE_VERSION = {}",
            WIRE_VERSION, version
        );
        sim2h_handle.disconnect(vec![uri]);
    }
}

#[tracing::instrument(level = "info", skip(sim2h_handle))]
async fn handle_message_join_space(
    sim2h_handle: Sim2hHandle,
    uri: Lib3hUri,
    signer: AgentId,
    data: SpaceData,
    receipt: WireMessage,
) {
    sim2h_handle
        .state()
        .new_connection(
            data.space_address.clone(),
            data.agent_id.clone(),
            uri.clone(),
        )
        .await;

    sim2h_handle.send_receipt(&receipt, &signer, &uri);

    sim2h_handle.send(
        data.agent_id.clone(),
        uri.clone(),
        &WireMessage::Lib3hToClient(
            ht::span_wrap_encode!(
                Level::INFO,
                Lib3hToClient::HandleGetGossipingEntryList(GetListData {
                    request_id: "".into(),
                    space_address: data.space_address.clone(),
                    provider_agent_id: data.agent_id.clone(),
                })
            )
            .into(),
        ),
    );

    sim2h_handle.send(
        data.agent_id.clone(),
        uri,
        &WireMessage::Lib3hToClient(
            ht::span_wrap_encode!(
                Level::INFO,
                Lib3hToClient::HandleGetAuthoringEntryList(GetListData {
                    request_id: "".into(),
                    space_address: data.space_address.clone(),
                    provider_agent_id: data.agent_id,
                })
            )
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
    data: DirectMessageData,
) {
    let to_agent_id = data.to_agent_id.clone();
    let data_space_hash = data.space_address.clone();
    inner_spawn_handle_message_send_dmx(
        sim2h_handle,
        to_agent_id,
        data_space_hash,
        space_hash,
        WireMessage::Lib3hToClient(
            ht::span_wrap_encode!(
                tracing::Level::INFO,
                Lib3hToClient::HandleSendDirectMessage(data)
            )
            .into(),
        ),
    );
}

#[instrument(level = "info", skip(sim2h_handle))]
fn spawn_handle_message_send_dm_result(
    sim2h_handle: Sim2hHandle,
    _uri: Lib3hUri,
    _signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    data: DirectMessageData,
) {
    let to_agent_id = data.to_agent_id.clone();
    let data_space_hash = data.space_address.clone();
    inner_spawn_handle_message_send_dmx(
        sim2h_handle,
        to_agent_id,
        data_space_hash,
        space_hash,
        WireMessage::Lib3hToClient(
            ht::span_wrap_encode!(Level::INFO, Lib3hToClient::SendDirectMessageResult(data)).into(),
        ),
    );
}
#[instrument(level = "info", skip(sim2h_handle))]
fn spawn_handle_message_publish_entry(
    sim2h_handle: Sim2hHandle,
    _uri: Lib3hUri,
    signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    data: ProvidedEntryData,
) {
    if data.space_address != *space_hash {
        error!(
            "space mismatch - agent is in {}, message is for {}",
            *space_hash, data.space_address
        );
        return;
    }

    tokio::task::spawn(async move {
        /*        let aspect_list: im::HashSet<AspectHash> = data
        .entry
        .aspect_list
        .iter()
        .map(|a| a.aspect_address.clone())
        .collect();*/
        let mut multi_message = Vec::new();
        for aspect in data.entry.aspect_list {
            let data = Lib3hToClient::HandleStoreEntryAspect(StoreEntryAspectData {
                request_id: "".into(),
                space_address: (&*space_hash).clone(),
                provider_agent_id: signer.clone(),
                entry_address: data.entry.entry_address.clone(),
                entry_aspect: aspect,
            });
            multi_message.push(ht::span_wrap_encode!(Level::INFO, data).into());
        }

        if multi_message.is_empty() {
            return;
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
            /* send not guaranteed to work so we can't mark as held.
            sim2h_handle.state().spawn_agent_holds_aspects(
                (&*space_hash).clone(),
                (&*agent_id).clone(),
                data.entry.entry_address.clone(),
                aspect_list.clone(),
            );*/
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
        if aspects.is_empty() {
            continue;
        }

        sim2h_handle.state().spawn_agent_holds_aspects(
            (&*space_hash).clone(),
            signer.clone(),
            entry_hash,
            aspects.into(),
        );
    }
}

#[instrument(level = "info", skip(sim2h_handle))]
fn spawn_handle_message_authoring_entry_list(
    sim2h_handle: Sim2hHandle,
    uri: Lib3hUri,
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

    tokio::task::spawn(
        async move {
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
                    multi_message.push(
                        ht::span_wrap_encode!(
                            Level::INFO,
                            Lib3hToClient::HandleFetchEntry(FetchEntryData {
                                request_id: "".to_string(),
                                space_address: (&*space_hash).clone(),
                                provider_agent_id: signer.clone(),
                                entry_address: entry_hash.clone(),
                                aspect_address_list: Some(aspect_list),
                            },)
                        )
                        .into(),
                    );
                }
            }
            trace!("AUTHORING multi-message: {:?}", multi_message);
            if !multi_message.is_empty() {
                let multi_send = WireMessage::MultiSend(multi_message);
                sim2h_handle.send(signer, uri, &multi_send);
            }
        }
        .instrument(debug_span!("authoring_entry")),
    );
}

#[instrument(level = "info", skip(sim2h_handle))]
fn spawn_handle_message_fetch_entry_result(
    sim2h_handle: Sim2hHandle,
    _uri: Lib3hUri,
    signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    fetch_result: FetchEntryResultData,
) {
    if signer != fetch_result.provider_agent_id || fetch_result.space_address != *space_hash {
        error!(
            "space mismatch - agent is in {}, message is for {}",
            *space_hash, fetch_result.space_address
        );
        return;
    }

    tokio::task::spawn(
        async move {
            let state = sim2h_handle.state().get_clone().await;

            #[allow(clippy::type_complexity)]
            let mut to_agent: std::collections::HashMap<
                MonoRef<AgentId>,
                //(
                Vec<ht::EncodedSpanWrap<Lib3hToClient>>,
                //   std::collections::HashMap<EntryHash, im::HashSet<AspectHash>>,
                //),
            > = std::collections::HashMap::new();

            for aspect in fetch_result.entry.aspect_list {
                let agents_that_need_aspect = state.get_agents_that_need_aspect(
                    &space_hash,
                    &fetch_result.entry.entry_address,
                    &aspect.aspect_address.clone(),
                );

                for agent_id in agents_that_need_aspect {
                    let m = to_agent.entry(agent_id.clone()).or_default();
                    let data = Lib3hToClient::HandleStoreEntryAspect(StoreEntryAspectData {
                        request_id: "".into(),
                        space_address: (&*space_hash).clone(),
                        provider_agent_id: (&*agent_id).clone(),
                        entry_address: fetch_result.entry.entry_address.clone(),
                        entry_aspect: aspect.clone(),
                    });
                    m./*0.*/push(ht::span_wrap_encode!(Level::INFO, data).into());

                    /*let e =
                        m.1.entry(fetch_result.entry.entry_address.clone())
                            .or_default();
                    e.insert(aspect.aspect_address.clone());*/
                }
            }

            for (agent_id, /*(*/ multi_message /*, mut holding)*/) in to_agent.drain() {
                let uri = match state.lookup_joined(&space_hash, &agent_id) {
                    None => continue,
                    Some(uri) => uri,
                };

                let multi_send = WireMessage::MultiSend(multi_message);

                sim2h_handle.send((&*agent_id).clone(), (&*uri).clone(), &multi_send);

                /* Conductor may not actually hold when told because of a consistency error so
                   we can't mark this as held.  Conductor will send back a partial gossip list
                   after a hold request to tell us that it's held the aspects.
                for (entry_hash, aspects) in holding.drain() {
                    sim2h_handle.state().spawn_agent_holds_aspects(
                        (&*space_hash).clone(),
                        (&*agent_id).clone(),
                        entry_hash,
                        aspects,
                    );
                }
                 */
            }
        }
        .instrument(debug_span!("spawn_handle_message_fetch_entry_result")),
    );
}

#[instrument(level = "info", skip(sim2h_handle))]
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

    tokio::task::spawn(
        async move {
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
            let query_message = WireMessage::Lib3hToClient(
                ht::span_wrap_encode!(Level::INFO, Lib3hToClient::HandleQueryEntry(query_data))
                    .into(),
            );
            sim2h_handle.send((&*query_target).clone(), url.clone(), &query_message);
        }
        .instrument(debug_span!("message_query")),
    );
}

#[instrument(level = "info", skip(sim2h_handle))]
fn spawn_handle_message_query_entry_result(
    sim2h_handle: Sim2hHandle,
    _uri: Lib3hUri,
    signer: AgentId,
    space_hash: MonoRef<SpaceHash>,
    query_result: QueryEntryResultData,
) {
    if signer != query_result.responder_agent_id || query_result.space_address != *space_hash {
        error!(
            "space mismatch - agent is in {}, message is for {}",
            *space_hash, query_result.space_address
        );
        return;
    }

    tokio::task::spawn(
        async move {
            let req_agent_id = query_result.requester_agent_id.clone();
            let msg_out = WireMessage::ClientToLib3hResponse(
                ht::span_wrap_encode!(
                    Level::INFO,
                    ClientToLib3hResponse::QueryEntryResult(query_result)
                )
                .into(),
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
        }
        .instrument(debug_span!("handle_message_query_entry_result")),
    );
}

/// creates a tokio runtime and executes the Sim2h instance within it
/// returns the runtime so the user can choose how to manage the main loop
pub fn run_sim2h(
    crypto: Box<dyn CryptoSystem>,
    bind_spec: Lib3hUri,
    dht_algorithm: DhtAlgorithm,
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
        let sim2h = Sim2h::new(crypto, bind_spec, dht_algorithm);
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
#[newrelic_autotrace(SIM2H)]
impl Sim2h {
    /// create a new Sim2h server instance
    pub fn new(
        crypto: Box<dyn CryptoSystem>,
        bind_spec: Lib3hUri,
        dht_algorithm: DhtAlgorithm,
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
        );

        let config = TcpBindConfig::default();
        //        let config = TlsBindConfig::new(config).dev_certificate();

        // if we don't get any messages within a timeframe from a connection,
        // the connection will throw a timeout error and disconnect.
        let config = WssBindConfig::new(config)
            .disconnect_on_slow_pong_ms(Some(NO_MESSAGE_CONNECTION_TIMEOUT_MS));
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
                    let uuid = nanoid::simple();

                    let url: Lib3hUri =
                        url::Url::from(url2!("{}#{}", wss.remote_url(), uuid)).into();

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
            match (|| -> Sim2hResult<(AgentId, WireMessage, WireMessage)> {
                let signed_message = SignedWireMessage::try_from(payload.clone())?;
                let result = signed_message.verify().unwrap();
                if !result {
                    return Err(VERIFY_FAILED_ERR_STR.into());
                }
                let agent_id: AgentId = signed_message.provenance.source().into();
                let receipt = gen_receipt(&signed_message.payload);

                let wire_message = WireMessage::try_from(signed_message.payload)?;
                Ok((agent_id, wire_message, receipt))
            })() {
                Ok((source, wire_message, receipt)) => {
                    sim2h_handle.handle_message(url.clone(), wire_message, source.clone(), receipt);
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

            // spawn a task to periodically check for disconnects
            // due to connections being replaced in the sim2h_im_state
            let sim2h_handle = self.sim2h_handle.clone();
            tokio::task::spawn(async move {
                loop {
                    tokio::time::delay_for(std::time::Duration::from_millis(500)).await;
                    let disconnect_uri = sim2h_handle.state().check_disconnected().await;
                    sim2h_handle.disconnect(disconnect_uri);
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
            let span = debug_span!("missing aspect root", root = true);
            let _g = span.enter();
            let schedule_guard = self.missing_aspects_resync_schedule.get_guard();
            let sim2h_handle = self.sim2h_handle.clone();
            tokio::task::spawn(
                missing_aspects_resync(sim2h_handle, schedule_guard)
                    .instrument(debug_span!("missing aspect future")),
            );
        }

        Ok(did_work)
    }
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

            let gossip_aspects =
                match state.get_gossip_aspects_needed_for_agent(&space_hash, &agent_id) {
                    None => continue,
                    Some(r) => r,
                };

            fetch_entry_data(gossip_aspects, space_hash, &sim2h_handle, state);

            trace!(
                "sim2h gossip agent in {} ms",
                gossip_agent_start.elapsed().as_millis()
            );
        }
    }
    trace!("sim2h gossip full loop in {} ms (ok to be long, this task is broken into multiple sub-loops)", gossip_full_start.elapsed().as_millis());
}

struct EntriesAlreadyFetched {
    entries: std::collections::HashMap<MonoEntryHash, std::time::Instant>,
}

impl EntriesAlreadyFetched {
    pub fn check(&mut self, entry: &MonoEntryHash) -> bool {
        // first - prune
        self.entries.retain(|_, t| t.elapsed().as_millis() < 1000);

        // next - check
        if self.entries.contains_key(entry) {
            return true;
        }

        // finally - set
        self.entries
            .insert(entry.clone(), std::time::Instant::now());

        false
    }
}

lazy_static! {
    static ref ENTRIES_ALREADY_FETCHED: std::sync::Mutex<EntriesAlreadyFetched> = {
        std::sync::Mutex::new(EntriesAlreadyFetched {
            entries: std::collections::HashMap::new(),
        })
    };
}

fn check_already_fetched(entry: &MonoEntryHash) -> bool {
    ENTRIES_ALREADY_FETCHED
        .lock()
        .expect("failed mutex lock")
        .check(entry)
}

fn fetch_entry_data(
    gossip_aspects: im::HashMap<MonoEntryHash, im::HashSet<MonoAspectHash>>,
    space_hash: &MonoRef<SpaceHash>,
    sim2h_handle: &Sim2hHandle,
    state: StoreRef,
) {
    for (entry_hash, aspects) in gossip_aspects.iter() {
        if aspects.is_empty() {
            continue;
        }

        if check_already_fetched(&entry_hash) {
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

        let wire_message = WireMessage::Lib3hToClient({
            let s = FetchEntryData {
                request_id: "".to_string(),
                space_address: (&**space_hash).clone(),
                provider_agent_id: (&*query_agent).clone(),
                entry_address: (&**entry_hash).clone(),
                //aspect_address_list: Some(aspects.iter().map(|a| (&**a).clone()).collect()),
                // david.b - We have more breadth than depth to worry about here
                //           i.e. all aspects for a given entry_address are
                //           not that numerous compared to how many entry
                //           addresses we have to deal with.
                //           Doing `None` here makes our "AlreadyFetched" logic
                //           work better.
                aspect_address_list: None,
            };
            debug!(message = "wire_message", ?s.request_id, ?s.space_address);
            ht::span_wrap_encode!(tracing::Level::INFO, Lib3hToClient::HandleFetchEntry(s)).into()
        });

        sim2h_handle.send((&*query_agent).clone(), (&*uri).clone(), &wire_message);
    }
}
