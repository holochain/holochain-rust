//! `cargo run --bin sim2h_stress -- --help`

#[macro_use]
extern crate log;
#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate serde_derive;
extern crate holochain_tracing as ht;

use holochain_stress::*;
use in_stream::*;
use lib3h_crypto_api::CryptoSystem;
use lib3h_protocol::{data_types::*, protocol::*, uri::Lib3hUri};
use lib3h_sodium::SodiumCryptoSystem;
use sim2h::{
    crypto::{Provenance, SignedWireMessage},
    run_sim2h, DhtAlgorithm, WireMessage,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};
use structopt::StructOpt;
use url2::prelude::*;

/// options for configuring this specific stress run
#[derive(StructOpt, Serialize, Deserialize, Debug, Clone)]
struct OptStressRunConfig {
    #[structopt(short, long, env = "STRESS_THREAD_COUNT", default_value = "0")]
    /// how many threads to spin up in the job executor pool - 0 for cpu count
    thread_count: usize,

    #[structopt(short, long, env = "STRESS_JOB_COUNT", default_value = "100")]
    /// how many parallel jobs to execute
    job_count: usize,

    #[structopt(short, long, env = "STRESS_RUN_TIME_MS", default_value = "10000")]
    /// total runtime for the test
    run_time_ms: u64,

    #[structopt(short, long, env = "STRESS_WARM_TIME_MS", default_value = "5000")]
    /// total runtime for the test
    warm_time_ms: u64,

    #[structopt(
        short,
        long,
        env = "STRESS_PROGRESS_INTERVAL_MS",
        default_value = "1000"
    )]
    /// how often to output in-progress statistics
    progress_interval_ms: u64,

    #[structopt(long, env = "STRESS_PING_FREQ_MS", default_value = "1000")]
    /// how often each job should send a ping to sim2h
    ping_freq_ms: u64,

    #[structopt(long, env = "STRESS_PUBLISH_FREQ_MS", default_value = "1000")]
    /// how often each job should publish a new entry
    publish_freq_ms: u64,

    #[structopt(long, env = "STRESS_PUBLISH_BYTE_COUNT", default_value = "1024")]
    /// how many bytes should be published each time
    publish_byte_count: usize,

    #[structopt(long, env = "STRESS_DM_FREQ_MS", default_value = "1000")]
    /// how often each job should send a direct message to another agent
    dm_freq_ms: u64,

    #[structopt(long, env = "STRESS_DM_BYTE_COUNT", default_value = "1024")]
    /// how many bytes should be direct messaged each time
    dm_byte_count: usize,
}

impl Default for OptStressRunConfig {
    fn default() -> Self {
        OptStressRunConfig::from_iter(<Vec<&str>>::new().iter())
    }
}

/// options for setting up the sim2h server
#[derive(StructOpt, Serialize, Deserialize, Debug, Clone)]
struct OptSim2hConfig {
    #[structopt(long, env = "SIM2H_PORT", default_value = "0")]
    /// port on which to spin up the sim2h server
    sim2h_port: u16,

    #[structopt(long, env = "SIM2H_MESSAGE_LOG_FILE")]
    /// optional sim2h log file path
    sim2h_message_log_file: Option<std::path::PathBuf>,
}

/// pulling the sim2h stress test commandline options together
#[derive(StructOpt, Serialize, Deserialize, Debug, Clone)]
#[structopt(name = "sim2h_stress")]
struct Opt {
    #[structopt(
        short,
        long,
        env = "STRESS_CONFIG",
        default_value = "sim2h_stress.toml"
    )]
    /// specify a config file to load stress options
    config_file: std::path::PathBuf,

    #[structopt(long)]
    /// generate a demo stress config file and exit
    gen_config: bool,

    #[structopt(flatten)]
    stress: OptStressRunConfig,

    #[structopt(flatten)]
    sim2h: OptSim2hConfig,
}

impl Opt {
    /// do all the steps to resolve args
    /// will pick CLI args first, fallback to ENV, then fall back to config
    fn resolve() -> Self {
        let mut args = Opt::from_args();

        let def_stress = OptStressRunConfig::default();

        if args.gen_config {
            println!("{}", toml::to_string_pretty(&def_stress).unwrap());
            std::process::exit(0);
        }

        if let Ok(config) = std::fs::read_to_string(&args.config_file) {
            let cfg_stress: OptStressRunConfig = toml::from_str(&config).unwrap();
            macro_rules! cfg_def {
                ($i:ident) => {
                    if *$i == def_stress.$i {
                        *$i = cfg_stress.$i;
                    }
                };
            }
            match &mut args.stress {
                // destructure so we get a compile error here if more
                // fields are added to this struct
                OptStressRunConfig {
                    thread_count,
                    job_count,
                    run_time_ms,
                    warm_time_ms,
                    progress_interval_ms,
                    ping_freq_ms,
                    publish_freq_ms,
                    publish_byte_count,
                    dm_freq_ms,
                    dm_byte_count,
                } => {
                    cfg_def!(thread_count);
                    cfg_def!(job_count);
                    cfg_def!(run_time_ms);
                    cfg_def!(warm_time_ms);
                    cfg_def!(progress_interval_ms);
                    cfg_def!(ping_freq_ms);
                    cfg_def!(publish_freq_ms);
                    cfg_def!(publish_byte_count);
                    cfg_def!(dm_freq_ms);
                    cfg_def!(dm_byte_count);
                }
            }
        }

        args
    }

    /// private convert our cli options into a stress job config
    fn create_stress_run_config<S: StressSuite, J: StressJob>(
        &self,
        suite: S,
        job_factory: JobFactory<J>,
    ) -> StressRunConfig<S, J> {
        StressRunConfig {
            thread_pool_size: self.stress.thread_count,
            job_count: self.stress.job_count,
            run_time_ms: self.stress.run_time_ms,
            warm_time_ms: self.stress.warm_time_ms,
            progress_interval_ms: self.stress.progress_interval_ms,
            suite,
            job_factory,
        }
    }
}

//fn await_in_stream_connect(connect_uri: &Lib3hUri) -> InStreamWss<InStreamTls<InStreamTcp>> {
fn await_in_stream_connect(connect_uri: &Lib3hUri) -> InStreamWss<InStreamTcp> {
    let timeout = std::time::Instant::now()
        .checked_add(std::time::Duration::from_millis(10000))
        .unwrap();

    let mut read_frame = WsFrame::default();

    // keep trying to connect
    loop {
        //let config = WssConnectConfig::new(TlsConnectConfig::new(TcpConnectConfig::default()));
        let config = WssConnectConfig::new(TcpConnectConfig::default());
        let mut connection = InStreamWss::connect(&(**connect_uri).clone().into(), config).unwrap();
        connection.write(WsFrame::Ping(b"".to_vec())).unwrap();

        loop {
            let mut err = false;

            match connection.read(&mut read_frame) {
                Ok(_) => return connection,
                Err(e) if e.would_block() => (),
                Err(_) => {
                    err = true;
                }
            }

            if std::time::Instant::now() >= timeout {
                panic!("could not connect within timeout");
            }

            if err {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// inner struct for ActiveAgentIds
struct ActiveAgentIdsInner {
    agent_id_list: [Option<String>; 5],
    write_ptr: usize,
    read_ptr: usize,
}

/// small thread-safe ring buffer for tracking active agent ids
/// jobs use this to send direct messages amongst themselves
#[derive(Clone)]
struct ActiveAgentIds {
    inner: Arc<Mutex<ActiveAgentIdsInner>>,
}

impl ActiveAgentIds {
    /// create a new agent_id ring buffer
    pub fn new() -> Self {
        ActiveAgentIds {
            inner: Arc::new(Mutex::new(ActiveAgentIdsInner {
                agent_id_list: [None, None, None, None, None],
                write_ptr: 0,
                read_ptr: 0,
            })),
        }
    }

    /// write a new agent_id to the ring buffer
    pub fn write(&mut self, agent_id: &str) {
        let mut inner = self.inner.lock().unwrap();
        let idx = inner.write_ptr;
        inner.agent_id_list[idx] = Some(agent_id.to_string());
        inner.write_ptr += 1;
        if inner.write_ptr >= 5 {
            inner.write_ptr = 0;
        }
    }

    /// read an agent_id from the ring buffer
    pub fn read(&self) -> Option<String> {
        let mut inner = self.inner.lock().unwrap();
        let idx = inner.read_ptr;
        let out = inner.agent_id_list[idx].clone();
        inner.read_ptr += 1;
        if inner.read_ptr >= 5 {
            inner.read_ptr = 0;
        }
        out
    }
}

thread_local! {
    pub static CRYPTO: Box<dyn CryptoSystem> = Box::new(SodiumCryptoSystem::new());
}

/// our job is a websocket connection to sim2h immitating a holochain-rust core
struct Job {
    agent_id: String,
    agent_ids: ActiveAgentIds,
    #[allow(dead_code)]
    pub_key: Arc<Mutex<Box<dyn lib3h_crypto_api::Buffer>>>,
    sec_key: Arc<Mutex<Box<dyn lib3h_crypto_api::Buffer>>>,
    connection: InStreamWss<InStreamTcp>,
    //connection: InStreamWss<InStreamTls<InStreamTcp>>,
    stress_config: OptStressRunConfig,
    next_ping: std::time::Instant,
    next_publish: std::time::Instant,
    next_dm: std::time::Instant,
    ping_sent_stack: VecDeque<std::time::Instant>,
    pending_dms: HashMap<String, std::time::Instant>,
}

impl Job {
    /// create a new job - connected to sim2h
    pub fn new(
        connect_uri: &Lib3hUri,
        stress_config: OptStressRunConfig,
        agent_ids: ActiveAgentIds,
    ) -> Self {
        let (pub_key, sec_key) = CRYPTO.with(|crypto| {
            let mut pub_key = crypto.buf_new_insecure(crypto.sign_public_key_bytes());
            let mut sec_key = crypto.buf_new_secure(crypto.sign_secret_key_bytes());
            crypto.sign_keypair(&mut pub_key, &mut sec_key).unwrap();
            (pub_key, sec_key)
        });
        let enc = hcid::HcidEncoding::with_kind("hcs0").unwrap();
        let agent_id = enc.encode(&*pub_key).unwrap();
        info!("GENERATED AGENTID {}", agent_id);

        let connection = await_in_stream_connect(connect_uri);

        let mut out = Self {
            agent_id,
            agent_ids,
            pub_key: Arc::new(Mutex::new(pub_key)),
            sec_key: Arc::new(Mutex::new(sec_key)),
            connection,
            stress_config,
            next_ping: std::time::Instant::now(),
            next_publish: std::time::Instant::now(),
            next_dm: std::time::Instant::now(),
            ping_sent_stack: VecDeque::new(),
            pending_dms: HashMap::new(),
        };

        out.join_space();

        out
    }

    /// sign a message and send it to sim2h
    pub fn send_wire(&mut self, message: WireMessage) {
        let payload: Opaque = message.into();
        let payload_buf: Box<dyn lib3h_crypto_api::Buffer> = Box::new(payload.clone().as_bytes());
        let sig = base64::encode(
            &*CRYPTO
                .with(|crypto| {
                    let mut sig = crypto.buf_new_insecure(crypto.sign_bytes());
                    crypto
                        .sign(&mut sig, &payload_buf, &*self.sec_key.lock().unwrap())
                        .unwrap();
                    sig
                })
                .read_lock(),
        );
        let signed_message = SignedWireMessage {
            provenance: Provenance::new(self.agent_id.clone().into(), sig.into()),
            payload,
        };
        let to_send: Opaque = signed_message.into();
        self.connection.write(to_send.as_bytes().into()).unwrap();
    }

    /// join the space "abcd" : )
    pub fn join_space(&mut self) {
        self.send_wire(WireMessage::ClientToLib3h(
            ht::top_follower("join_space")
                .wrap(ClientToLib3h::JoinSpace(SpaceData {
                    agent_id: self.agent_id.clone().into(),
                    request_id: "".to_string(),
                    space_address: "abcd".to_string().into(),
                }))
                .into(),
        ));
    }

    /// send a ping message to sim2h
    pub fn ping(&mut self, logger: &mut StressJobMetricLogger) {
        self.ping_sent_stack.push_back(std::time::Instant::now());
        self.send_wire(WireMessage::Ping);
        logger.log("ping_send_count", 1.0);
    }

    pub fn dm(&mut self, logger: &mut StressJobMetricLogger) {
        if let Some(to_agent_id) = self.agent_ids.read() {
            let content = CRYPTO.with(|crypto| {
                let mut content = crypto.buf_new_insecure(self.stress_config.dm_byte_count);
                crypto.randombytes_buf(&mut content).unwrap();
                let content: Opaque = (*content.read_lock()).to_vec().into();
                content
            });

            let rid = nanoid::simple();
            self.pending_dms
                .insert(rid.clone(), std::time::Instant::now());

            self.send_wire(WireMessage::ClientToLib3h(
                ht::top_follower("dm")
                    .wrap(ClientToLib3h::SendDirectMessage(DirectMessageData {
                        space_address: "abcd".to_string().into(),
                        request_id: rid,
                        to_agent_id: to_agent_id.into(),
                        from_agent_id: self.agent_id.clone().into(),
                        content,
                    }))
                    .into(),
            ));

            logger.log("dm_send_count", 1.0);
        }
    }

    /// send a ping message to sim2h
    pub fn publish(&mut self, logger: &mut StressJobMetricLogger) {
        let (addr, aspect) = CRYPTO.with(|crypto| {
            let mut addr = crypto.buf_new_insecure(32);
            crypto.randombytes_buf(&mut addr).unwrap();
            let addr = base64::encode(&*addr.read_lock());

            let mut aspect_data = crypto.buf_new_insecure(self.stress_config.publish_byte_count);
            crypto.randombytes_buf(&mut aspect_data).unwrap();

            let mut aspect_hash = crypto.buf_new_insecure(crypto.hash_sha256_bytes());
            crypto.hash_sha256(&mut aspect_hash, &aspect_data).unwrap();

            let enc = hcid::HcidEncoding::with_kind("hca0").unwrap();
            let aspect_hash = enc.encode(&*aspect_hash).unwrap();

            let aspect_data: Opaque = (*aspect_data.read_lock()).to_vec().into();

            let sent_epoch_millis = format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );

            let aspect = EntryAspectData {
                aspect_address: aspect_hash.into(),
                type_hint: sent_epoch_millis,
                aspect: aspect_data,
                publish_ts: 0,
            };

            (addr, aspect)
        });

        let msg = ClientToLib3h::PublishEntry(ProvidedEntryData {
            space_address: "abcd".to_string().into(),
            provider_agent_id: self.agent_id.clone().into(),
            entry: EntryData {
                entry_address: addr.into(),
                aspect_list: vec![aspect],
            },
        });
        self.send_wire(WireMessage::ClientToLib3h(
            ht::top_follower("publish").wrap(msg).into(),
        ));

        logger.log("publish_send_count", 1.0);
    }

    fn priv_handle_msg(&mut self, logger: &mut StressJobMetricLogger, msg: WireMessage) {
        match msg {
            WireMessage::Pong => {
                // with the current Ping/Pong structs
                // there's no way to correlate specific messages
                // if we switch to using the Websocket Ping/Pong
                // we could put a message id in them.
                let res = self.ping_sent_stack.pop_front();
                if res.is_none() {
                    return;
                }
                let res = res.unwrap();
                logger.log("ping_recv_pong_in_ms", res.elapsed().as_millis() as f64);
            }
            WireMessage::Lib3hToClient(span_wrap) => self.priv_handle_msg_inner(logger, span_wrap),
            WireMessage::MultiSend(msg_list) => {
                for span_wrap in msg_list {
                    self.priv_handle_msg_inner(logger, span_wrap)
                }
            }
            e @ _ => panic!("unexpected: {:?}", e),
        }
    }

    fn priv_handle_msg_inner(
        &mut self,
        logger: &mut StressJobMetricLogger,
        span_wrap: ht::EncodedSpanWrap<Lib3hToClient>,
    ) {
        match &span_wrap.data {
            Lib3hToClient::HandleGetAuthoringEntryList(_)
            | Lib3hToClient::HandleGetGossipingEntryList(_) => {}
            Lib3hToClient::HandleStoreEntryAspect(aspect) => {
                let epoch_millis = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                let published = aspect.entry_aspect.type_hint.parse::<u64>().unwrap();
                let elapsed = epoch_millis - published;
                logger.log("publish_received_aspect_in_ms", elapsed as f64);
            }
            Lib3hToClient::HandleSendDirectMessage(dm_data) => {
                let to_agent_id: String = dm_data.to_agent_id.clone().into();
                assert_eq!(self.agent_id, to_agent_id);
                let mut out_dm = dm_data.clone();
                out_dm.to_agent_id = dm_data.from_agent_id.clone();
                out_dm.from_agent_id = dm_data.to_agent_id.clone();
                self.send_wire(WireMessage::Lib3hToClientResponse(
                    span_wrap.swapped(Lib3hToClientResponse::HandleSendDirectMessageResult(out_dm)),
                ));
            }
            Lib3hToClient::SendDirectMessageResult(dm_data) => {
                let res = self.pending_dms.remove(&dm_data.request_id);
                if res.is_none() {
                    panic!("invalid dm.request_id")
                }
                let res = res.unwrap();
                logger.log("dm_result_in_ms", res.elapsed().as_millis() as f64);
            }
            e @ _ => panic!("unexpected: {:?}", e),
        }
    }
}

impl StressJob for Job {
    /// check for any messages from sim2h and also send a ping
    fn tick(&mut self, logger: &mut StressJobMetricLogger) -> StressJobTickResult {
        let mut frame = WsFrame::default();
        match self.connection.read(&mut frame) {
            Ok(_) => {
                if let WsFrame::Binary(b) = frame {
                    let msg: WireMessage = serde_json::from_slice(&b).unwrap();
                    self.priv_handle_msg(logger, msg);
                } else {
                    panic!("unexpected {:?}", frame);
                }
            }
            Err(e) if e.would_block() => (),
            Err(e) => panic!(e),
        }

        let now = std::time::Instant::now();

        if now >= self.next_ping {
            self.next_ping = now
                .checked_add(std::time::Duration::from_millis(
                    self.stress_config.ping_freq_ms,
                ))
                .unwrap();

            self.ping(logger);
        }

        if now >= self.next_publish {
            self.next_publish = now
                .checked_add(std::time::Duration::from_millis(
                    self.stress_config.publish_freq_ms,
                ))
                .unwrap();
            self.publish(logger);
        }

        if now >= self.next_dm {
            self.next_dm = now
                .checked_add(std::time::Duration::from_millis(
                    self.stress_config.dm_freq_ms,
                ))
                .unwrap();
            self.dm(logger);
        }

        self.agent_ids.write(&self.agent_id);

        StressJobTickResult::default()
    }
}

/// our suite creates a thread for sim2h and gives the code processor time
struct Suite {
    sim2h_cont: Arc<Mutex<bool>>,
    sim2h_join: Option<std::thread::JoinHandle<()>>,
    bound_uri: Lib3hUri,
    snd_thread_logger: crossbeam_channel::Sender<StressJobMetricLogger>,
}

impl Suite {
    /// create a new sim2h server instance on given port
    #[allow(clippy::mutex_atomic)]
    pub fn new(port: u16) -> Self {
        let (snd1, rcv1) = crossbeam_channel::unbounded();
        let (snd2, rcv2) = crossbeam_channel::unbounded::<StressJobMetricLogger>();

        let sim2h_cont = Arc::new(Mutex::new(true));
        let sim2h_cont_clone = sim2h_cont.clone();
        let sim2h_join = Some(std::thread::spawn(move || {
            // changed to ws until we reactive TLS
            let url = Url2::parse(&format!("ws://127.0.0.1:{}", port));

            let mut logger = None;

            let (mut rt, binding) = run_sim2h(
                Box::new(SodiumCryptoSystem::new()),
                Lib3hUri(url.into()),
                DhtAlgorithm::FullSync,
                None,
            );
            rt.block_on(async move {
                tokio::task::spawn(async move {
                    snd1.send(binding.await.unwrap()).unwrap();
                });
                while *sim2h_cont_clone.lock().unwrap() {
                    tokio::time::delay_for(std::time::Duration::from_millis(1)).await;

                    if let Ok(l) = rcv2.try_recv() {
                        logger.replace(l);
                    }

                    let start = std::time::Instant::now();

                    if let Some(logger) = &mut logger {
                        logger.log("tick_sim2h_elapsed_ms", start.elapsed().as_millis() as f64);
                    }
                }
            });
        }));

        let bound_uri = rcv1.recv().unwrap();
        println!("GOT BOUND: {:?}", bound_uri);

        info!("sim2h started, attempt test self connection");

        // wait 'till server is accepting connections.
        // let this one get dropped
        await_in_stream_connect(&bound_uri);

        Self {
            sim2h_cont,
            sim2h_join,
            bound_uri,
            snd_thread_logger: snd2,
        }
    }
}

fn print_stats(stats: StressStats) {
    println!("\n== RUN COMPLETE - Results ==");
    println!(" - master_tick_count: {}", stats.master_tick_count);

    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    table.set_titles(prettytable::row![
        r -> "STAT",
        l -> "COUNT",
        l -> "MIN",
        l -> "MAX",
        l -> "MEAN",
    ]);

    for (k, v) in stats.log_stats.iter() {
        table.add_row(prettytable::row![
            r -> k,
            l -> v.count,
            l -> v.min,
            l -> v.max,
            l -> v.avg,
        ]);
    }
    table.printstd();
}

impl StressSuite for Suite {
    fn start(&mut self, logger: StressJobMetricLogger) {
        self.snd_thread_logger.send(logger).unwrap();
    }

    fn warmup_complete(&mut self) {
        println!("WARMUP COMPLETE");
    }

    fn progress(&mut self, stats: &StressStats) {
        println!("PROGRESS - {}", stats.master_tick_count);
    }

    fn stop(&mut self, stats: StressStats) {
        *self.sim2h_cont.lock().unwrap() = false;
        self.sim2h_join.take().unwrap().join().unwrap();
        //println!("FINAL STATS: {:#?}", stats);
        print_stats(stats);
    }
}

/// main function executes the stress suite given the cli arguments
pub fn main() {
    env_logger::init();
    let opt = Opt::resolve();
    if opt.sim2h.sim2h_message_log_file.is_some() {
        unimplemented!();
    }
    let suite = Suite::new(opt.sim2h.sim2h_port);
    let bound_uri = suite.bound_uri.clone();
    println!(
        r#"== SIM2H STRESS CONFIG ==
{}
== SIM2H STRESS CONFIG =="#,
        toml::to_string_pretty(&opt.stress).unwrap()
    );
    let agent_ids = ActiveAgentIds::new();
    let stress_config = opt.stress.clone();
    let config = opt.create_stress_run_config(
        suite,
        Box::new(move |_| Job::new(&bound_uri, stress_config.clone(), agent_ids.clone())),
    );
    println!("WARMING UP...");
    stress_run(config);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_start_sim2h_and_connect() {
        env_logger::init();
        let suite = Suite::new(0);
        let mut stress_cfg = OptStressRunConfig::default();
        stress_cfg.publish_freq_ms = 500;
        let agent_ids = ActiveAgentIds::new();
        let mut job = Some(Job::new(&suite.bound_uri, stress_cfg, agent_ids));
        std::thread::sleep(std::time::Duration::from_millis(500));
        stress_run(StressRunConfig {
            thread_pool_size: 1,
            job_count: 1,
            run_time_ms: 1000,
            warm_time_ms: 100,
            progress_interval_ms: 2000,
            suite,
            job_factory: Box::new(move |_| std::mem::replace(&mut job, None).unwrap()),
        });
    }
}
