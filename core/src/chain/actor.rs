use riker_default::DefaultModel;
use riker::actors::*;
use std::{fmt};
use futures::executor::block_on;
use riker_patterns::ask::ask;
use hash_table::entry::Entry;
use hash_table::pair::Pair;
use error::HolochainError;
use chain::Chain;

lazy_static! {
    pub static ref CHAIN_SYS: ActorSystem<ChainProtocol> = {
        let chain_model: DefaultModel<ChainProtocol> = DefaultModel::new();
        ActorSystem::new(&chain_model).unwrap()
    };
}

/// anything that can be asked ChainProtocol and block on responses
/// needed to support implementing ask on upstream ActorRef from riker
pub trait AskChain {
    fn ask(&self, message: ChainProtocol) -> ChainProtocol;
}

impl AskChain for ActorRef<ChainProtocol> {
    fn ask(&self, message: ChainProtocol) -> ChainProtocol {
        block_on(
            ask(
                &(*CHAIN_SYS),
                self,
                message,
            )
        ).unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum ChainProtocol {
    Push(Entry),
    PushPair(Pair),
    PushResult(Result<Pair, HolochainError>),

    GetEntry(String),
    GetEntryResult(Result<Option<Pair>, HolochainError>),
}

impl fmt::Display for ChainProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<ActorMsg<ChainProtocol>> for ChainProtocol {

    fn into(self) -> ActorMsg<ChainProtocol> {
        ActorMsg::User(self)
    }

}

pub struct ChainActor {

    chain: Chain,

}

impl ChainActor {

    pub fn new(chain: Chain) -> ChainActor {
        ChainActor{
            chain,
        }
    }

    pub fn actor(chain: Chain) -> BoxActor<ChainProtocol> {
        Box::new(ChainActor::new(chain))
    }

    pub fn props(chain: Chain) -> BoxActorProd<ChainProtocol> {
        Props::new_args(Box::new(ChainActor::actor), chain)
    }

    pub fn new_ref(chain: Chain) -> ActorRef<ChainProtocol> {
        CHAIN_SYS.actor_of(
            ChainActor::props(chain),
            "chain",
        ).unwrap()
    }

}

impl Actor for ChainActor {
    type Msg = ChainProtocol;

    fn receive(
        &mut self,
        context: &Context<Self::Msg>,
        message: Self::Msg,
        sender: Option<ActorRef<Self::Msg>>,
    ) {
        println!("received {}", message);
        sender.try_tell(
            match message {
                ChainProtocol::Push(entry) => ChainProtocol::PushResult(self.chain.push(&entry)),
                ChainProtocol::PushPair(pair) => ChainProtocol::PushResult(self.chain.push_pair(&pair)),
                ChainProtocol::PushResult(_) => unreachable!(),

                ChainProtocol::GetEntry(key) => ChainProtocol::GetEntryResult(self.chain.get_entry(&key)),
                ChainProtocol::GetEntryResult(_) => unreachable!(),
            },
            Some(context.myself()),
        ).unwrap();
    }
}

#[cfg(test)]
pub mod tests {
    use super::ChainActor;
    use riker::actors::*;
    use chain::tests::test_chain;
    use chain::actor::ChainProtocol;

    pub fn test_chain_actor() -> ActorRef<ChainProtocol> {
        ChainActor::new_ref(test_chain())
    }

}
