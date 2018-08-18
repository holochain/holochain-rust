// use riker_default::DefaultModel;
use riker::actors::*;
use futures::executor::block_on;
use riker_patterns::ask::ask;
use hash_table::entry::Entry;
use hash_table::pair::Pair;
use error::HolochainError;
use chain::Chain;
use chain::SourceChain;
use snowflake;
use riker_default::SimpleLogger;
use riker::kernel::Dispatcher;
use config::Config;
use futures::{Future, Never};
use riker::futures_util::spawn;
use riker_default::DeadLettersActor;
use riker_default::BasicTimer;
use riker_default::MapVec;
use riker::system::NoIo;
use futures::executor::{ThreadPool, ThreadPoolBuilder};

struct ChainModel;

// @see https://github.com/riker-rs/riker/blob/master/riker-default/riker-dispatcher/src/lib.rs
pub struct ChainDispatcher {
    inner: ThreadPool,
}

impl Dispatcher for ChainDispatcher {
    fn new(_config: &Config, _: bool) -> ChainDispatcher {
        ChainDispatcher {
            inner: ThreadPoolBuilder::new()
                                        .pool_size(4)
                                        .name_prefix("pool-thread-chain-#")
                                        .create()
                                        .unwrap()
        }
    }

    fn execute<F>(&mut self, f: F)
        where F: Future<Item=(), Error=Never> + Send + 'static
    {
        self.inner.run(spawn(f)).unwrap();
    }
}

impl Model for ChainModel {
    type Msg = ChainProtocol;
    type Dis = ChainDispatcher;
    type Ded = DeadLettersActor<Self::Msg>;
    type Tmr = BasicTimer<Self::Msg>;
    type Evs = MapVec<Self::Msg>;
    type Tcp = NoIo<Self::Msg>;
    type Udp = NoIo<Self::Msg>;
    type Log = SimpleLogger<Self::Msg>;
}

lazy_static! {
    pub static ref CHAIN_SYS: ActorSystem<ChainProtocol> = {
        // let chain_model: DefaultModel<ChainProtocol> = DefaultModel::new();
        let chain_model = ChainModel{};
        ActorSystem::new(&chain_model).unwrap()
    };
}

#[derive(Debug, Clone)]
pub enum ChainProtocol {
    TopPair,
    TopPairResult(Option<Pair>),

    TopPairType(String),
    TopPairTypeResult(Option<Pair>),

    PushEntry(Entry),
    PushEntryResult(Result<Pair, HolochainError>),

    PushPair(Pair),
    PushPairResult(Result<Pair, HolochainError>),

    GetEntry(String),
    GetEntryResult(Result<Option<Pair>, HolochainError>),

    GetPair(String),
    GetPairResult(Result<Option<Pair>, HolochainError>),
}

impl Into<ActorMsg<ChainProtocol>> for ChainProtocol {
    fn into(self) -> ActorMsg<ChainProtocol> {
        ActorMsg::User(self)
    }
}

/// anything that can be asked ChainProtocol and block on responses
/// needed to support implementing ask on upstream ActorRef from riker
/// convenience wrappers around chain struct methods
pub trait AskChain {
    fn ask(&self, message: ChainProtocol) -> ChainProtocol;
}

impl AskChain for ActorRef<ChainProtocol> {
    fn ask(&self, message: ChainProtocol) -> ChainProtocol {
        let a = ask(
            &(*CHAIN_SYS),
            self,
            message,
        );
        println!("asking chain");
        block_on(a).unwrap()
    }
}

impl SourceChain for ActorRef<ChainProtocol> {

    fn top_pair(&self) -> Option<Pair> {
        let response = self.ask(ChainProtocol::TopPair);
        unwrap_to!(response => ChainProtocol::TopPairResult).clone()
    }

    fn top_pair_type(&self, t: &str) -> Option<Pair> {
        let response = self.ask(ChainProtocol::TopPairType(t.to_string()));
        unwrap_to!(response => ChainProtocol::TopPairTypeResult).clone()
    }

    fn push_entry(&mut self, entry: &Entry) -> Result<Pair, HolochainError> {
        let response = self.ask(ChainProtocol::PushEntry(entry.clone()));
        unwrap_to!(response => ChainProtocol::PushEntryResult).clone()
    }

    fn get_entry(&self, entry_hash: &str) -> Result<Option<Pair>, HolochainError> {
        let response = self.ask(ChainProtocol::GetEntry(entry_hash.to_string()));
        unwrap_to!(response => ChainProtocol::GetEntryResult).clone()
    }

    fn push_pair(&mut self, pair: &Pair) -> Result<Pair, HolochainError> {
        let response = self.ask(ChainProtocol::PushPair(pair.clone()));
        unwrap_to!(response => ChainProtocol::PushPairResult).clone()
    }

    fn get_pair(&self, k: &str) -> Result<Option<Pair>, HolochainError> {
        let response = self.ask(ChainProtocol::GetPair(k.to_string()));
        unwrap_to!(response => ChainProtocol::GetPairResult).clone()
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
            &snowflake::ProcessUniqueId::new().to_string(),
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
        println!("received {:?}", message);
        sender.try_tell(
            match message {
                ChainProtocol::TopPair => ChainProtocol::TopPairResult(self.chain.top_pair()),
                ChainProtocol::TopPairResult(_) => unreachable!(),

                ChainProtocol::TopPairType(t) => ChainProtocol::TopPairTypeResult(self.chain.top_pair_type(&t)),
                ChainProtocol::TopPairTypeResult(_) => unreachable!(),

                ChainProtocol::PushPair(pair) => ChainProtocol::PushPairResult(self.chain.push_pair(&pair)),
                ChainProtocol::PushPairResult(_) => unreachable!(),

                ChainProtocol::PushEntry(entry) => ChainProtocol::PushEntryResult(self.chain.push_entry(&entry)),
                ChainProtocol::PushEntryResult(_) => unreachable!(),

                ChainProtocol::GetEntry(key) => ChainProtocol::GetEntryResult(self.chain.get_entry(&key)),
                ChainProtocol::GetEntryResult(_) => unreachable!(),

                ChainProtocol::GetPair(key) => ChainProtocol::GetPairResult(self.chain.get_pair(&key)),
                ChainProtocol::GetPairResult(_) => unreachable!(),
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
