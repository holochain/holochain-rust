use crate::{cache::*, error::Sim2hResult, sim2h_context::Sim2hContextSpawnFn};
use futures::{
    channel::{mpsc, oneshot},
    future::{BoxFuture, FutureExt},
    lock::Mutex,
    stream::StreamExt,
};
use im::HashMap;
use lib3h_crypto_api::CryptoSystem;
use lib3h_protocol::{
    types::{AgentPubKey, SpaceHash},
    uri::Lib3hUri,
};
use log::*;
use std::sync::Arc;

enum StateWriteProtocol {
    CloneSpace {
        result: oneshot::Sender<Option<Space>>,
        space_address: SpaceHash,
    },
    JoinAgent {
        result: oneshot::Sender<Sim2hResult<()>>,
        space_address: SpaceHash,
        agent_id: AgentPubKey,
        uri: Lib3hUri,
    },
}

pub struct Sim2hStateInner {
    pub spaces: HashMap<SpaceHash, Space>,
    pub crypto: Box<dyn CryptoSystem>,
}

impl Sim2hStateInner {
    pub fn new(crypto: Box<dyn CryptoSystem>) -> Self {
        Self {
            spaces: HashMap::new(),
            crypto,
        }
    }

    fn mutate(&mut self, msg: StateWriteProtocol) {
        match msg {
            StateWriteProtocol::CloneSpace {
                result,
                space_address,
            } => {
                result.send(self.clone_space(&space_address)).unwrap();
            }
            StateWriteProtocol::JoinAgent {
                result,
                space_address,
                agent_id,
                uri,
            } => {
                result
                    .send(self.join_agent(&space_address, agent_id, uri))
                    .unwrap();
            }
        }
    }

    fn clone_space(&mut self, space_address: &SpaceHash) -> Option<Space> {
        Some(self.spaces.get(space_address)?.clone())
    }

    fn get_or_create_space(&mut self, space_address: &SpaceHash) -> &mut Space {
        if !self.spaces.contains_key(space_address) {
            self.spaces
                .insert(space_address.clone(), Space::new(self.crypto.box_clone()));
            info!(
                "\n\n+++++++++++++++\nNew Space: {}\n+++++++++++++++\n",
                space_address
            );
        }
        self.spaces.get_mut(space_address).unwrap()
    }

    fn join_agent(
        &mut self,
        space_address: &SpaceHash,
        agent_id: AgentPubKey,
        uri: Lib3hUri,
    ) -> Sim2hResult<()> {
        let space = self.get_or_create_space(space_address);
        space.join_agent(agent_id, uri)
    }
}

pub struct Sim2hState {
    #[allow(dead_code)]
    inner: Arc<Mutex<Sim2hStateInner>>,
    send_mutate: mpsc::UnboundedSender<StateWriteProtocol>,
}

pub type Sim2hStateRef = Arc<Sim2hState>;

impl Sim2hState {
    pub fn new(spawn_fn: Sim2hContextSpawnFn, crypto: Box<dyn CryptoSystem>) -> Sim2hStateRef {
        let inner = Arc::new(Mutex::new(Sim2hStateInner::new(crypto)));
        let (send_mutate, mut recv_mutate) = mpsc::unbounded();
        let out = Arc::new(Sim2hState {
            inner: inner.clone(),
            send_mutate,
        });
        spawn_fn(
            async move {
                loop {
                    let msg = match recv_mutate.next().await {
                        None => return, // closed stream
                        Some(msg) => msg,
                    };
                    inner.lock().await.mutate(msg);
                }
            }
            .boxed(),
        );
        out
    }

    pub fn delete_me_block_lock(&self) -> futures::lock::MutexGuard<'_, Sim2hStateInner> {
        futures::executor::block_on(self.inner.lock())
    }

    pub fn join_agent(
        &self,
        space_address: SpaceHash,
        agent_id: AgentPubKey,
        uri: Lib3hUri,
    ) -> BoxFuture<'static, Sim2hResult<()>> {
        let (snd, rcv) = oneshot::channel();
        self.send_mutate
            .unbounded_send(StateWriteProtocol::JoinAgent {
                result: snd,
                space_address,
                agent_id,
                uri,
            })
            .unwrap();
        async { rcv.await? }.boxed()
    }

    pub fn lookup_joined(
        &self,
        space_address: SpaceHash,
        agent_id: AgentPubKey,
    ) -> BoxFuture<'static, Option<Lib3hUri>> {
        let (snd, rcv) = oneshot::channel();
        self.send_mutate
            .unbounded_send(StateWriteProtocol::CloneSpace {
                result: snd,
                space_address,
            })
            .unwrap();
        async move {
            match rcv.await {
                Ok(Some(space)) => space.agent_id_to_uri(&agent_id),
                _ => None,
            }
        }
        .boxed()
    }
}
