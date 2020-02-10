#![feature(vec_remove_item)]
#![feature(proc_macro_hygiene)]

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
    collections::{HashMap, HashSet},
    convert::TryFrom,
    sync::Arc,
};

use holochain_locksmith::Mutex;
use holochain_metrics::{config::MetricPublisherConfig, Metric};

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

#[derive(Clone)]
struct MetricsTimerGenerator {
    sender: tokio::sync::mpsc::UnboundedSender<(&'static str, f64)>,
}

impl MetricsTimerGenerator {
    pub fn new() -> (Self, BoxFuture<'static, ()>) {
        let (sender, mut recv) = tokio::sync::mpsc::unbounded_channel::<(&'static str, f64)>();
        let out = async move {
            let metric_publisher = MetricPublisherConfig::default().create_metric_publisher();
            loop {
                let msg = match recv.next().await {
                    None => return,
                    Some(msg) => msg,
                };
                // TODO - this write is technically blocking
                //        move to spawn_blocking?? use tokio::sync::Mutex??
                metric_publisher
                    .write()
                    .unwrap()
                    .publish(&Metric::new_timestamped_now(msg.0, None, msg.1));
            }
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

mod sim2h_state;
pub(crate) use sim2h_state::*;

#[derive(Debug)]
struct Sim2hComHandleMessage {
    uri: Lib3hUri,
    message: WireMessage,
    signer: AgentId,
}

#[derive(Debug)]
struct Sim2hComHandleJoined {
    uri: Lib3hUri,
    space_address: SpaceHash,
    agent_id: AgentId,
    message: WireMessage,
}

#[derive(Debug)]
enum Sim2hCom {
    HandleMessage(Box<Sim2hComHandleMessage>),
    HandleJoined(Box<Sim2hComHandleJoined>),
    Disconnect(Vec<Lib3hUri>),
}

#[derive(Clone)]
/// A clonable reference to our Sim2h instance that can be passed
/// into `'static` async blocks && still be able to make sim2h calls
struct Sim2hHandle {
    state: Arc<tokio::sync::Mutex<Sim2hState>>,
    send_com: tokio::sync::mpsc::UnboundedSender<Sim2hCom>,
    dht_algorithm: DhtAlgorithm,
    metric_gen: MetricsTimerGenerator,
    connection_mgr: ConnectionMgrHandle,
}

impl Sim2hHandle {
    pub fn new(
        state: Arc<tokio::sync::Mutex<Sim2hState>>,
        send_com: tokio::sync::mpsc::UnboundedSender<Sim2hCom>,
        dht_algorithm: DhtAlgorithm,
        metric_gen: MetricsTimerGenerator,
        connection_mgr: ConnectionMgrHandle,
    ) -> Self {
        Self {
            state,
            send_com,
            dht_algorithm,
            metric_gen,
            connection_mgr,
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

    /// acquire a mutex lock to our state data
    pub async fn lock_state(&self) -> tokio::sync::MutexGuard<'_, Sim2hState> {
        self.state.lock().await
    }

    /// forward a message to be handled
    pub fn handle_message(&self, uri: Lib3hUri, message: WireMessage, signer: AgentId) {
        if let Err(e) =
            self.send_com
                .send(Sim2hCom::HandleMessage(Box::new(Sim2hComHandleMessage {
                    uri,
                    message,
                    signer,
                })))
        {
            error!("error sending message to sim2h - shutting down? {:?}", e);
        }
    }

    /// forward a message to an already joined connection to be handled
    pub fn handle_joined(
        &self,
        uri: Lib3hUri,
        space_address: SpaceHash,
        agent_id: AgentId,
        message: WireMessage,
    ) {
        self.send_com
            .send(Sim2hCom::HandleJoined(Box::new(Sim2hComHandleJoined {
                uri,
                space_address,
                agent_id,
                message,
            })))
            .expect("can send");
    }

    /// disconnect an active connection
    pub fn disconnect(&self, disconnect: Vec<Lib3hUri>) {
        self.send_com
            .send(Sim2hCom::Disconnect(disconnect))
            .expect("can send");
    }
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
        loop {
            // NOTE - once we move everything in sim2h to futures
            //        we can get rid of the `process()` function
            //        and remove this spawn_blocking code
            let sim2h = match tokio::task::spawn_blocking(blocking_fn.take().unwrap()).await {
                Err(e) => {
                    // sometimes we get errors on shutdown...
                    // we can't recover because the sim2h instance is lost
                    // but don't panic... just exit
                    error!("sim2h process failed: {:?}", e);
                    return;
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
    missing_aspects_resync: std::time::Instant,
    dht_algorithm: DhtAlgorithm,
    recv_com: tokio::sync::mpsc::UnboundedReceiver<Sim2hCom>,
    sim2h_handle: Sim2hHandle,
    connection_count: usize,
    metric_gen: MetricsTimerGenerator,
    tracer: ht::Tracer,
    // TODO: This is just a hack to tell if tracing is on because sometimes we need to know to avoid doing extra work
    // Maybe the above tracer can just be in an option?
    tracing_on: Option<()>,
}

#[autotrace]
//#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
impl Sim2h {
    /// create a new Sim2h server instance
    pub fn new(
        crypto: Box<dyn CryptoSystem>,
        bind_spec: Lib3hUri,
        dht_algorithm: DhtAlgorithm,
        tracer: Option<ht::Tracer>,
    ) -> Self {
        let (metric_gen, metric_task) = MetricsTimerGenerator::new();

        let (connection_mgr, connection_mgr_evt_recv) = ConnectionMgr::new(tracer.clone());

        let (wss_send, wss_recv) = crossbeam_channel::unbounded();
        let state = Arc::new(tokio::sync::Mutex::new(Sim2hState {
            crypto: crypto.box_clone(),
            connection_states: HashMap::new(),
            spaces: HashMap::new(),
            metric_gen: metric_gen.clone(),
            connection_mgr: connection_mgr.clone(),
        }));
        let (send_com, recv_com) = tokio::sync::mpsc::unbounded_channel();
        let sim2h_handle = Sim2hHandle::new(
            state,
            send_com,
            dht_algorithm.clone(),
            metric_gen.clone(),
            connection_mgr,
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
            missing_aspects_resync: std::time::Instant::now(),
            dht_algorithm,
            recv_com,
            sim2h_handle,
            connection_count: 0,
            metric_gen,
            tracing_on: tracer.as_ref().map(|_| ()),
            tracer: tracer.unwrap_or_else(|| ht::null_tracer()),
        };

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
                let mut state = sim2h_handle.lock_state().await;

                for wss in wss_list.drain(..) {
                    let url: Lib3hUri = url::Url::from(wss.remote_url()).into();
                    let uuid = nanoid::simple();
                    open_lifecycle("adding conn job", &uuid, &url);

                    state
                        .connection_states
                        .insert(url.clone(), (nanoid::simple(), ConnectionState::new()));

                    state.connection_mgr.connect(url, wss);
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

        let mut disconnect = Vec::new();
        for _ in 0..100 {
            match self.connection_mgr_evt_recv.try_recv() {
                Ok(evt) => {
                    did_work = true;
                    match evt {
                        ConMgrEvent::Disconnect(uri, maybe_err) => {
                            debug!("disconnect {} {:?}", uri, maybe_err);
                            disconnect.push(uri);
                        }
                        ConMgrEvent::ReceiveData(uri, data) => {
                            self.priv_handle_recv_data(uri, data);
                        }
                        ConMgrEvent::ConnectionCount(count) => {
                            self.connection_count = count;
                        }
                    }
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Closed) => {
                    panic!("connection mgr channel broken");
                }
            }
        }

        if !disconnect.is_empty() {
            self.sim2h_handle.disconnect(disconnect);
        }

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
                let tracer = self.tracing_on.map(|_| self.tracer.clone());
                Sim2h::handle_payload(self.sim2h_handle.clone(), uri, payload, tracer);
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

    // adds an agent to a space
    async fn join(sim2h_handle: Sim2hHandle, uri: Lib3hUri, data: SpaceData) {
        let _m = sim2h_handle.metric_timer("sim2h-join");
        debug!("join entered for {} with {:?}", uri, data);
        let mut pending_messages = {
            let mut state = sim2h_handle.lock_state().await;

            let (_uuid, conn) = match state.connection_states.get_mut(&uri) {
                Some((uuid, conn)) => (uuid, conn),
                None => {
                    error!("no agent found at {} ", uri);
                    sim2h_handle.disconnect(vec![uri]);
                    return;
                }
            };

            let pending_messages = match conn {
                ConnectionState::Limbo(pending_messages) => {
                    pending_messages.drain(..).collect::<Vec<_>>()
                }
                _ => {
                    error!("no agent found in limbo at {} ", uri);
                    sim2h_handle.disconnect(vec![uri]);
                    return;
                }
            };

            let new_conn = match ConnectionState::new_joined(
                data.space_address.clone(),
                data.agent_id.clone(),
            ) {
                Err(e) => {
                    error!("error creating new connection state: {:?}", e);
                    sim2h_handle.disconnect(vec![uri]);
                    return;
                }
                Ok(new_conn) => new_conn,
            };

            *conn = new_conn;

            if let Err(e) =
                state.join_agent(&data.space_address, data.agent_id.clone(), uri.clone())
            {
                error!("error joining agent {} - {:?}", uri, e);
                sim2h_handle.disconnect(vec![uri]);
                return;
            }
            info!(
                "Agent {:?} joined space {:?}",
                data.agent_id, data.space_address
            );
            state.request_gossiping_list(
                uri.clone(),
                data.space_address.clone(),
                data.agent_id.clone(),
            );
            state.request_authoring_list(
                uri.clone(),
                data.space_address.clone(),
                data.agent_id.clone(),
            );

            pending_messages
        };

        debug!("pending messages in join: {}", pending_messages.len());
        for message in pending_messages.drain(..) {
            sim2h_handle.handle_message(uri.clone(), message.clone(), data.agent_id.clone());
        }
        trace!("join done");
    }

    // handler for messages sent to sim2h
    fn handle_message(
        &mut self,
        uri: Lib3hUri,
        message: WireMessage,
        signer: AgentId,
    ) -> Sim2hResult<()> {
        let _m = self.metric_gen.timer("sim2h-state-handle_message");
        trace!("handle_message entered for {}", uri);

        MESSAGE_LOGGER
            .lock()
            .log_in(signer.clone(), uri.clone(), message.clone());

        // TODO: anyway, but especially with this Ping/Pong, mitigate DoS attacks.
        if message == WireMessage::Ping {
            debug!("Sending Pong in response to Ping");
            let sim2h_handle = self.sim2h_handle.clone();
            tokio::task::spawn(async move {
                sim2h_handle
                    .lock_state()
                    .await
                    .send(signer, uri, &WireMessage::Pong);
            });
            return Ok(());
        }
        if let WireMessage::Status = message {
            debug!("Sending StatusResponse in response to Status");
            let sim2h_handle = self.sim2h_handle.clone();
            let connection_count = self.connection_count;
            tokio::task::spawn(async move {
                let state = sim2h_handle.lock_state().await;
                let spaces_len = state.spaces.len();
                state.send(
                    signer.clone(),
                    uri.clone(),
                    &WireMessage::StatusResponse(StatusData {
                        spaces: spaces_len,
                        connections: connection_count,
                        redundant_count: match sim2h_handle.dht_algorithm() {
                            DhtAlgorithm::FullSync => 0,
                            DhtAlgorithm::NaiveSharding { redundant_count } => *redundant_count,
                        },
                        version: WIRE_VERSION,
                    }),
                );
            });
            return Ok(());
        }
        if let WireMessage::Hello(version) = message {
            debug!("Sending HelloResponse in response to Hello({})", version);
            let sim2h_handle = self.sim2h_handle.clone();
            tokio::task::spawn(async move {
                let state = sim2h_handle.lock_state().await;
                state.send(
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
                // versions do not match - disconnect them
                if version != WIRE_VERSION {
                    warn!("Disconnecting client for bad version this WIRE_VERSIO = {}, client WIRE_VERSION = {}", WIRE_VERSION, version);
                    sim2h_handle.disconnect(vec![uri]);
                }
            });
            return Ok(());
        }

        tokio::task::spawn(Sim2h::handle_connection_msg(
            self.sim2h_handle.clone(),
            uri,
            message,
            signer,
            self.tracing_on.map(|_| self.tracer.clone()),
        ));
        Ok(())
    }

    async fn handle_connection_msg(
        sim2h_handle: Sim2hHandle,
        uri: Lib3hUri,
        message: WireMessage,
        signer: AgentId,
        tracer: Option<ht::Tracer>,
    ) {
        let _spanguard = tracer.map(|t| match message.clone() {
            WireMessage::ClientToLib3h(span_wrap) => {
                let span =
                    ht::SpanWrap::from(span_wrap).follower(&t, format!("{}:{}", file!(), line!()));
                span.map(|span| ht::push_span(span))
            }
            _ => None,
        });
        autotrace_deep_block!({
            let _m = sim2h_handle.metric_timer("sim2h-handle_connection_msg");
            let state = sim2h_handle.clone();
            let mut state = state.lock_state().await;
            let (uuid, agent) = match state.connection_states.get_mut(&uri) {
                Some((uuid, agent)) => (uuid, agent),
                None => {
                    error!("handle message for disconnected agent: {}", uri);
                    return;
                }
            };
            conn_lifecycle("handle_message", &uuid, &agent, &uri);

            match agent {
                ConnectionState::Limbo(ref mut pending_messages) => {
                    if let WireMessage::ClientToLib3h(ht::EncodedSpanWrap {
                        data: ClientToLib3h::JoinSpace(data),
                        ..
                    }) = message
                    {
                        if data.agent_id != signer {
                            error!("{}", SIGNER_MISMATCH_ERR_STR);
                            return;
                        }
                        tokio::task::spawn(Sim2h::join(sim2h_handle, uri, data));
                    } else {
                        debug!("inserting into pending message while in limbo.");
                        // TODO: maybe have some upper limit on the number of messages
                        // we allow to queue before dropping the connections
                        pending_messages.push(message);

                        // commenting this out...
                        // I don't think we want core to have to deal with this
                        // we just haven't finished processing the join yet
                        /*
                        state.send(
                            signer.clone(),
                            uri.clone(),
                            &WireMessage::Err(WireError::MessageWhileInLimbo),
                        );
                        */
                    }
                }
                ConnectionState::Joined(space_address, agent_id) => {
                    if *agent_id != signer {
                        error!("{}", SIGNER_MISMATCH_ERR_STR);
                        return;
                    }
                    sim2h_handle.handle_joined(
                        uri,
                        space_address.clone(),
                        agent_id.clone(),
                        message,
                    );
                }
            }
        })
    }

    fn handle_payload(
        sim2h_handle: Sim2hHandle,
        url: Lib3hUri,
        payload: Opaque,
        tracer: Option<ht::Tracer>,
    ) {
        tokio::task::spawn(async move {
            let _m = sim2h_handle.metric_timer("sim2h-handle_payload");
            match (|| -> Sim2hResult<(AgentId, WireMessage)> {
                let signed_message = SignedWireMessage::try_from(payload.clone())?;
                let result = signed_message.verify().unwrap();
                if !result {
                    return Err(VERIFY_FAILED_ERR_STR.into());
                }
                let wire_message = WireMessage::try_from(signed_message.payload)?;
                Ok((signed_message.provenance.source().into(), wire_message))
            })() {
                Ok((source, wire_message)) => {
                    let _spanguard = tracer.map(|t| match wire_message.clone() {
                        WireMessage::ClientToLib3h(span_wrap) => {
                            if let ClientToLib3h::SendDirectMessage(_) = span_wrap.data {
                                let span = ht::SpanWrap::from(span_wrap.clone()).follower_(
                                    &t,
                                    format!("handle_payload-ClientToLib3h"),
                                    |options| {
                                        options
                                            .tag(ht::debug_tag(
                                                "ClientToLib3h",
                                                span_wrap.data.clone(),
                                            ))
                                            .start()
                                            .into()
                                    },
                                );
                                span.map(|span| ht::push_span(span))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    });
                    sim2h_handle.handle_message(url, wire_message, source)
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

        let mut d_list = Vec::new();
        for _ in 0..100 {
            match self.recv_com.try_recv() {
                Ok(Sim2hCom::Disconnect(mut disconnects)) => {
                    did_work = true;
                    d_list.append(&mut disconnects);
                }
                Ok(Sim2hCom::HandleMessage(m)) => {
                    did_work = true;
                    self.handle_message(m.uri, m.message, m.signer)?;
                }
                Ok(Sim2hCom::HandleJoined(m)) => {
                    let _spanguard = self.tracing_on.map(|_| match m.message.clone() {
                        WireMessage::ClientToLib3h(span_wrap) => {
                            let span = ht::SpanWrap::from(span_wrap)
                                .follower(&self.tracer, format!("HandleJoined ClientToLib3h"));
                            span.map(|span| ht::push_span(span))
                        }
                        _ => None,
                    });
                    did_work = true;
                    self.handle_joined(m.uri, m.space_address, m.agent_id, m.message)?;
                }
                _ => (),
            }
        }
        if !d_list.is_empty() {
            let sim2h_handle = self.sim2h_handle.clone();
            tokio::task::spawn(async move {
                let mut state = sim2h_handle.lock_state().await;
                for url in d_list {
                    state.disconnect(&url);
                }
            });
        }

        if self.priv_check_incoming_connections() {
            did_work = true;
        }

        if self.priv_check_incoming_messages() {
            did_work = true;
        }

        if std::time::Instant::now() >= self.missing_aspects_resync {
            self.missing_aspects_resync = std::time::Instant::now()
                .checked_add(std::time::Duration::from_millis(
                    RETRY_FETCH_MISSING_ASPECTS_INTERVAL_MS,
                ))
                .expect("can add interval ms");

            let sim2h_handle = self.sim2h_handle.clone();
            tokio::task::spawn(async move {
                sim2h_handle.lock_state().await.retry_sync_missing_aspects();
            });
        }

        Ok(did_work)
    }

    /// given an incoming messages, prepare a proxy message and whether it's an publish or request
    #[allow(clippy::cognitive_complexity)]
    fn handle_joined(
        &mut self,
        uri: Lib3hUri,
        space_address: SpaceHash,
        agent_id: AgentId,
        message: WireMessage,
    ) -> Sim2hResult<()> {
        let _m = self.metric_gen.timer("sim2h-handle_joined");
        trace!("handle_joined entered");
        debug!(
            "<<IN<< {} from {}",
            message.message_type(),
            agent_id.to_string()
        );
        match message.clone() {
            // First make sure we are not receiving a message in the wrong direction.
            // Panic for now so we can easily spot a mistake.
            // Should maybe break up WireMessage into two different structs so we get the
            // error already when parsing an incoming payload.
            WireMessage::Lib3hToClient(_) | WireMessage::ClientToLib3hResponse(_) =>
                panic!("This is soo wrong. Clients should never send a message that only servers can send."),
            // -- Space -- //
            WireMessage::ClientToLib3h(span_wrap) => {
                let span = ht::SpanWrap::from(span_wrap.clone()).follower(&self.tracer, "handle_joined - ClientToLib3h");
                let _spanguard = span.map(|span| ht::push_span(span));
                match span_wrap.data.clone() {
                    ClientToLib3h::JoinSpace(_) => {
                        Err("join message should have been processed elsewhere and can't be proxied".into())
                    }
                    ClientToLib3h::LeaveSpace(data) => {
                        let sim2h_handle = self.sim2h_handle.clone();
                        tokio::task::spawn(async move {
                            let mut state = sim2h_handle.lock_state().await;
                            if let Err(e) = state.leave(&uri, &data) {
                                error!("failed to leave space: {:?}", e);
                                sim2h_handle.disconnect(vec![uri]);
                            }
                        });
                        Ok(())
                    }

                    // -- Direct Messaging -- //
                    // Send a message directly to another agent on the network
                    ClientToLib3h::SendDirectMessage(dm_data) => {
                        if (dm_data.from_agent_id != agent_id) || (dm_data.space_address != space_address) {
                            return Err(SPACE_MISMATCH_ERR_STR.into());
                        }
                        let sim2h_handle = self.sim2h_handle.clone();
                        let tracer_handle = self.tracer.clone();
                        tokio::task::spawn(async move {
                            let state = sim2h_handle.lock_state().await;
                            let to_url = match state
                                .lookup_joined(&space_address, &dm_data.to_agent_id)
                            {
                                Some(to_url) => to_url,
                                None => {
                                    error!("unvalidated proxy agent {}", &dm_data.to_agent_id);
                                    return;
                                }
                            };
                            let span = ht::SpanWrap::from(span_wrap.clone()).follower(&tracer_handle, format!("{}:{}", file!(), line!()));
                            let _spanguard = span.map(|span| ht::push_span(span));
                            state.send(
                                dm_data.to_agent_id.clone(),
                                to_url,
                                &WireMessage::Lib3hToClient(span_wrap.swapped(Lib3hToClient::HandleSendDirectMessage(dm_data.to_owned())))
                            );
                        });
                        Ok(())
                    }
                    ClientToLib3h::PublishEntry(data) => {
                        if (data.provider_agent_id != agent_id) || (data.space_address != space_address) {
                            return Err(SPACE_MISMATCH_ERR_STR.into());
                        }
                        let sim2h_handle = self.sim2h_handle.clone();
                        tokio::task::spawn(Sim2hState::handle_new_entry_data(
                            sim2h_handle,
                            data.entry,
                            space_address,
                            agent_id,
                        ));
                        Ok(())
                    }
                    ClientToLib3h::QueryEntry(query_data) => {
                        if let DhtAlgorithm::NaiveSharding {redundant_count} = self.dht_algorithm {
                            let sim2h_handle = self.sim2h_handle.clone();
                            tokio::task::spawn(async move {
                                let disconnects = sim2h_handle
                                    .lock_state().await
                                    .build_query(
                                        space_address,
                                        query_data,
                                        redundant_count
                                    );
                                sim2h_handle.disconnect(disconnects);
                            });
                            Ok(())
                        } else {
                            Err("Got ClientToLib3h::QueryEntry in full-sync mode".into())
                        }
                    }
                    _ => {
                        warn!("Ignoring unimplemented message: {:?}", message );
                        Err(format!("Message not implemented: {:?}", message.clone()).into())
                    }
                }
            }
            WireMessage::Lib3hToClientResponse(span_wrap) => {
                let span = ht::SpanWrap::from(span_wrap.clone()).follower(&self.tracer, "handle_joined - Lib3hToClientResponse");
                let _spanguard = span.map(|span| ht::push_span(span));
                match span_wrap.data.clone() {
                    // Direct message response
                    Lib3hToClientResponse::HandleSendDirectMessageResult(
                        dm_data,
                    ) => {
                        if (dm_data.from_agent_id != agent_id) || (dm_data.space_address != space_address) {
                            return Err(SPACE_MISMATCH_ERR_STR.into());
                        }
                        let sim2h_handle = self.sim2h_handle.clone();
                        tokio::task::spawn({
                            let tracer = self.tracer.clone();
                            async move {
                            let state = sim2h_handle.lock_state().await;
                            let to_url = match state
                                .lookup_joined(&space_address, &dm_data.to_agent_id)
                            {
                                Some(to_url) => to_url,
                                None => {
                                    error!("unvalidated proxy agent {}", &dm_data.to_agent_id);
                                    return;
                                }
                            };
                            let span = ht::SpanWrap::from(span_wrap.clone()).follower(&tracer, format!("{}:{}", file!(), line!()));
                            let _spanguard = span.map(|span| ht::push_span(span));
                            state.send(
                                dm_data.to_agent_id.clone(),
                                to_url,
                                &WireMessage::Lib3hToClient(span_wrap.swapped(Lib3hToClient::SendDirectMessageResult(dm_data)))
                            );
                        }});
                        Ok(())
                    }
                    Lib3hToClientResponse::HandleGetAuthoringEntryListResult(list_data) => {
                        debug!("GOT AUTHORING LIST from {}", agent_id);
                        if (list_data.provider_agent_id != agent_id) || (list_data.space_address != space_address) {
                            return Err(SPACE_MISMATCH_ERR_STR.into());
                        }
                        self.handle_unseen_aspects(&uri, &space_address, &agent_id, &list_data);
                        Ok(())
                    }
                    Lib3hToClientResponse::HandleGetGossipingEntryListResult(list_data) => {
                        debug!("GOT GOSSIPING LIST from {}", agent_id);
                        if (list_data.provider_agent_id != agent_id) || (list_data.space_address != space_address) {
                            return Err(SPACE_MISMATCH_ERR_STR.into());
                        }
                        self.handle_unseen_aspects(&uri, &space_address, &agent_id, &list_data);

                        let sim2h_handle = self.sim2h_handle.clone();

                        tokio::task::spawn(async move {
                            let l_state = sim2h_handle.clone();
                            let mut l_state = l_state.lock_state().await;

                            // Check if the node is missing any aspects
                            let aspects_missing_at_node = match sim2h_handle.dht_algorithm() {
                                DhtAlgorithm::FullSync => l_state
                                    .get_space(&space_address)
                                    .all_aspects()
                                    .diff(&AspectList::from(list_data.address_map)),
                                DhtAlgorithm::NaiveSharding {redundant_count} => l_state
                                    .get_space(&space_address)
                                    .aspects_in_shard_for_agent(&agent_id, *redundant_count)
                                    .diff(&AspectList::from(list_data.address_map))
                            };

                            if aspects_missing_at_node.entry_addresses().count() > 0 {
                                warn!("MISSING ASPECTS at {}:\n{}", agent_id, aspects_missing_at_node.pretty_string());

                                // Cache info about what this agent is missing so we can make sure it got it
                                let missing_hashes: HashSet<(EntryHash, AspectHash)> = (&aspects_missing_at_node).into();
                                if missing_hashes.len() > 0 {
                                    l_state.add_missing_aspects(&space_address, &agent_id, missing_hashes);
                                }

                                match sim2h_handle.dht_algorithm() {

                                    DhtAlgorithm::FullSync => {
                                        let all_agents_in_space = l_state
                                            .get_space(&space_address)
                                            .all_agents()
                                            .keys()
                                            .cloned()
                                            .collect::<Vec<AgentPubKey>>();
                                        if all_agents_in_space.len() == 1 {
                                            error!("MISSING ASPECTS and no way to get them. Agent is alone in space..");
                                        } else {
                                            Sim2h::fetch_aspects_from_arbitrary_agent(
                                                sim2h_handle,
                                                aspects_missing_at_node,
                                                agent_id.clone(),
                                                all_agents_in_space,
                                                space_address.clone()
                                            );
                                        }
                                    },

                                    DhtAlgorithm::NaiveSharding {redundant_count} => {
                                        for entry_address in aspects_missing_at_node.entry_addresses() {
                                            let entry_loc = entry_location(&l_state.crypto, entry_address);
                                            let agent_pool = l_state
                                                .get_space(&space_address)
                                                .agents_supposed_to_hold_entry(entry_loc, *redundant_count)
                                                .keys()
                                                .cloned()
                                                .collect::<Vec<AgentPubKey>>();
                                            Sim2h::fetch_aspects_from_arbitrary_agent(
                                                sim2h_handle.clone(),
                                                aspects_missing_at_node.filtered_by_entry_hash(|e| e == entry_address),
                                                agent_id.clone(),
                                                agent_pool,
                                                space_address.clone()
                                            );
                                        }
                                    }
                                }

                            }
                        });

                        Ok(())
                    }
                    Lib3hToClientResponse::HandleFetchEntryResult(fetch_result) => {
                        if (fetch_result.provider_agent_id != agent_id) || (fetch_result.space_address != space_address) {
                            return Err(SPACE_MISMATCH_ERR_STR.into());
                        }
                        debug!("HANDLE FETCH ENTRY RESULT: {:?}", fetch_result);
                        if fetch_result.request_id == "" {
                            debug!("Got FetchEntry result from {} without request id - must be from authoring list", agent_id);
                            let sim2h_handle = self.sim2h_handle.clone();
                            tokio::task::spawn(Sim2hState::handle_new_entry_data(
                                sim2h_handle,
                                fetch_result.entry,
                                space_address,
                                agent_id,
                            ));
                        } else {
                            debug!("Got FetchEntry result with request id {} - this is for gossiping to agent with incomplete data", fetch_result.request_id);
                            let sim2h_handle = self.sim2h_handle.clone();
                            tokio::task::spawn(async move {
                                let to_agent_id = AgentPubKey::from(fetch_result.request_id);
                                let mut multi_messages = Vec::new();
                                let mut to_remove = Vec::new();
                                for aspect in fetch_result.entry.aspect_list {
                                    to_remove.push((
                                        fetch_result.entry.entry_address.clone(),
                                        aspect.aspect_address.clone(),
                                    ));
                                    let msg = Lib3hToClient::HandleStoreEntryAspect(
                                        StoreEntryAspectData {
                                            request_id: "".into(),
                                            space_address: space_address.clone(),
                                            provider_agent_id: agent_id.clone(),
                                            entry_address: fetch_result.entry.entry_address.clone(),
                                            entry_aspect: aspect,
                                        },
                                    );
                                    multi_messages.push(span_wrap.swapped(msg));
                                }

                                let store_message = WireMessage::MultiSend(multi_messages);

                                let mut state = sim2h_handle.lock_state().await;
                                let maybe_url = state.lookup_joined(&space_address, &to_agent_id);
                                if maybe_url.is_none() {
                                    error!("Got FetchEntryResult with request id that is not a known agent id. I guess we lost that agent before we could deliver missing aspects.");
                                    return;
                                }
                                let url = maybe_url.unwrap();
                                for (entry_address, aspect_address) in to_remove.drain(..) {
                                    state.remove_missing_aspect(
                                        &space_address,
                                        &to_agent_id,
                                        &entry_address,
                                        &aspect_address,
                                    );
                                }
                                state.send(to_agent_id, url, &store_message);
                            });
                        }

                        Ok(())
                    }
                    Lib3hToClientResponse::HandleQueryEntryResult(query_result) => {
                        if (query_result.responder_agent_id != agent_id) || (query_result.space_address != space_address)
                        {
                            return Err(SPACE_MISMATCH_ERR_STR.into());
                        }
                        let sim2h_handle = self.sim2h_handle.clone();
                        tokio::task::spawn(async move {
                            let req_agent_id = query_result.requester_agent_id.clone();
                            let msg_out = WireMessage::ClientToLib3hResponse(
                                span_wrap.swapped(
                                    ClientToLib3hResponse::QueryEntryResult(query_result)
                                )
                            );
                            let state = sim2h_handle.lock_state().await;
                            let to_url = match state
                                .lookup_joined(&space_address, &req_agent_id)
                            {
                                Some(to_url) => to_url,
                                None => {
                                    error!("unvalidated proxy agent {}", &req_agent_id);
                                    return;
                                }
                            };
                            state.send(
                                req_agent_id,
                                to_url,
                                &msg_out,
                            );
                        });
                        Ok(())
                    }
                    _ => {
                        warn!("Ignoring unimplemented message: {:?}", message );
                        Err(format!("Message not implemented: {:?}", message).into())
                    }
                }
            }

            _ => {
                warn!("Ignoring unimplemented message: {:?}", message );
                Err(format!("Message not implemented: {:?}", message).into())
            }
        }
    }

    #[autotrace]
    fn handle_unseen_aspects(
        &self,
        uri: &Lib3hUri,
        space_address: &SpaceHash,
        agent_id: &AgentId,
        list_data: &EntryListData,
    ) {
        let sim2h_handle = self.sim2h_handle.clone();
        let uri = uri.clone();
        let space_address = space_address.clone();
        let agent_id = agent_id.clone();
        let list_data = list_data.clone();
        tokio::task::spawn(async move {
            let disconnects = sim2h_handle.lock_state().await.build_handle_unseen_aspects(
                uri,
                space_address,
                agent_id,
                list_data,
            );
            sim2h_handle.disconnect(disconnects);
        });
    }

    fn fetch_aspects_from_arbitrary_agent(
        sim2h_handle: Sim2hHandle,
        aspects_to_fetch: AspectList,
        for_agent_id: AgentId,
        agent_pool: Vec<AgentId>,
        space_address: SpaceHash,
    ) {
        tokio::task::spawn(async move {
            let state = sim2h_handle.lock_state().await;
            let disconnects = state.build_aspects_from_arbitrary_agent(
                aspects_to_fetch,
                for_agent_id,
                agent_pool,
                space_address,
            );
            sim2h_handle.disconnect(disconnects);
        });
    }
}
