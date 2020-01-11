use std::sync::Arc;
use futures::{
    future::FutureExt,
    stream::StreamExt,
    lock::Mutex,
    channel::{mpsc, oneshot},
};
use lib3h_crypto_api::CryptoSystem;
use lib3h_protocol::{
    types::{AgentPubKey, SpaceHash},
    uri::Lib3hUri,
};
use crate::{
    cache::*,
    error::Sim2hResult,
    sim2h_context::Sim2hContextSpawnFn,
};
use im::HashMap;
use log::*;

enum StateWriteProtocol {
    JoinAgent {
        result: oneshot::Sender<Sim2hResult<()>>,
        space_address: SpaceHash,
        agent_id: AgentPubKey,
        uri: Lib3hUri,
    }
}

pub struct Sim2hStateInner {
    #[allow(dead_code)]
    spaces: HashMap<SpaceHash, Space>,
    crypto: Box<dyn CryptoSystem>,
}

impl Sim2hStateInner {
    pub fn new(
        crypto: Box<dyn CryptoSystem>,
    ) -> Self {
        Self {
            spaces: HashMap::new(),
            crypto,
        }
    }

    fn mutate(&mut self, msg: StateWriteProtocol) {
        match msg {
            StateWriteProtocol::JoinAgent {
                result, space_address, agent_id, uri
            } => {
                result.send(self.join_agent(&space_address, agent_id, uri)).unwrap();
            }
        }
    }

    fn get_or_create_space(&mut self, space_address: &SpaceHash) -> &mut Space {
        if !self.spaces.contains_key(space_address) {
            self.spaces.insert(
                space_address.clone(), Space::new(self.crypto.box_clone()));
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
    inner: Arc<Mutex<Sim2hStateInner>>,
    send_mutate: mpsc::UnboundedSender<StateWriteProtocol>,
}

pub type Sim2hStateRef = Arc<Sim2hState>;

impl Sim2hState {
    pub fn new(
        spawn_fn: Sim2hContextSpawnFn,
        crypto: Box<dyn CryptoSystem>,
    ) -> Sim2hStateRef {
        let inner = Arc::new(Mutex::new(Sim2hStateInner::new(crypto)));
        let (send_mutate, mut recv_mutate) = mpsc::unbounded();
        let out = Arc::new(Sim2hState {
            inner: inner.clone(),
            send_mutate,
        });
        spawn_fn(async move {
            loop {
                let msg = match recv_mutate.next().await {
                    None => return, // closed stream
                    Some(msg) => msg,
                };
                inner.lock().await.mutate(msg);
            }
        }.boxed());
        out
    }

    pub async fn join_agent(
        &mut self,
        space_address: SpaceHash,
        agent_id: AgentPubKey,
        uri: Lib3hUri,
    ) -> Sim2hResult<()> {
        let (snd, rcv) = oneshot::channel();
        self.send_mutate.unbounded_send(StateWriteProtocol::JoinAgent {
            result: snd,
            space_address,
            agent_id,
            uri,
        }).unwrap();
        rcv.await?
    }

    pub async fn lookup_joined(
        &self,
        space_address: SpaceHash,
        agent_id: AgentPubKey,
    ) -> Option<Lib3hUri> {
        let space = self.clone_space(space_address).await?;
        space.agent_id_to_uri(&agent_id)
    }

    async fn clone_space(&self, space_address: SpaceHash) -> Option<Space> {
        Some(self.inner.lock().await.spaces.get(&space_address)?.clone())
    }
}
