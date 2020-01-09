use crate::*;
use futures::{channel::mpsc, stream::StreamExt};
use holochain_locksmith::MutexGuard;
use std::sync::Arc;

pub(crate) type ConnectionStateItem = (String, ConnectionState);

enum Sim2hStateProtocol {}

pub struct Sim2hState {
    pub crypto: Box<dyn CryptoSystem>,
    pub connections: HashMap<Lib3hUri, ConnectionStateItem>,
    pub spaces: HashMap<SpaceHash, Space>,
}

impl Sim2hState {
    pub fn new(crypto: Box<dyn CryptoSystem>) -> Self {
        Self {
            crypto,
            connections: HashMap::new(),
            spaces: HashMap::new(),
        }
    }
}

pub struct DangerGuard<'lt> {
    mutex_guard: MutexGuard<'lt, Sim2hState>,
    space_address: SpaceHash,
}

impl<'lt> DangerGuard<'lt> {
    pub fn get(&mut self) -> &mut Space {
        self.mutex_guard
            .spaces
            .get_mut(&self.space_address)
            .unwrap()
    }
}

#[derive(Clone)]
pub struct Sim2hStateHandle {
    inner: Arc<Mutex<Sim2hState>>,
    send_proto: mpsc::UnboundedSender<Sim2hStateProtocol>,
}

impl Sim2hStateHandle {
    pub fn new(crypto: Box<dyn CryptoSystem>) -> Self {
        let (send_proto, mut recv_proto) = mpsc::unbounded();
        let out = Self {
            inner: Arc::new(Mutex::new(Sim2hState::new(crypto))),
            send_proto,
        };
        sim2h_spawn_ok(async move {
            loop {
                let _msg = match recv_proto.next().await {
                    None => return,
                    Some(msg) => msg,
                };
            }
        });
        out
    }

    pub fn danger_lock(&self) -> MutexGuard<'_, Sim2hState> {
        self.inner.f_lock()
    }

    pub fn danger_get_or_create_space(&mut self, space_address: &SpaceHash) -> DangerGuard<'_> {
        //let clock = std::time::SystemTime::now();

        let mut out = DangerGuard {
            mutex_guard: self.danger_lock(),
            space_address: space_address.clone(),
        };

        let crypto = out.mutex_guard.crypto.box_clone();
        if !out.mutex_guard.spaces.contains_key(space_address) {
            out.mutex_guard
                .spaces
                .insert(space_address.clone(), Space::new(crypto));
            info!(
                "\n\n+++++++++++++++\nNew Space: {}\n+++++++++++++++\n",
                space_address
            );
        }

        /*
        self.metric_publisher
            .write()
            .unwrap()
            .publish(&Metric::new_timestamped_now(
                "sim2h-danger_get_or_create_space.latency",
                None,
                clock.elapsed().unwrap().as_millis() as f64,
            ));
        */

        out
    }
}
