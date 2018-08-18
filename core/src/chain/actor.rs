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
// use riker::kernel::Dispatcher;
// use hash_table::actor::HASH_TABLE_SYS;
use actor::SYS;
use actor::Protocol;

// struct ChainModel;
//
// // @see https://github.com/riker-rs/riker/blob/master/riker-default/riker-dispatcher/src/lib.rs
// pub struct ChainDispatcher {
//     inner: ThreadPool,
// }
//
// impl Dispatcher for ChainDispatcher {
//     fn new(_config: &Config, _: bool) -> ChainDispatcher {
//         ChainDispatcher {
//             inner: ThreadPoolBuilder::new()
//                                         .pool_size(4)
//                                         .name_prefix("pool-thread-chain-#")
//                                         .create()
//                                         .unwrap()
//         }
//     }
//
//     fn execute<F>(&mut self, f: F)
//         where F: Future<Item=(), Error=Never> + Send + 'static
//     {
//         self.inner.run(spawn(f)).unwrap();
//     }
// }
//
// impl Model for ChainModel {
//     type Msg = Protocol;
//     type Dis = ChainDispatcher;
//     type Ded = DeadLettersActor<Self::Msg>;
//     type Tmr = BasicTimer<Self::Msg>;
//     type Evs = MapVec<Self::Msg>;
//     type Tcp = NoIo<Self::Msg>;
//     type Udp = NoIo<Self::Msg>;
//     type Log = SimpleLogger<Self::Msg>;
// }

// lazy_static! {
//     pub static ref CHAIN_SYS: ActorSystem<Protocol> = {
//         // let chain_model: DefaultModel<Protocol> = DefaultModel::new();
//         let chain_model = ChainModel{};
//         ActorSystem::new(&chain_model).unwrap()
//     };
// }

// #[derive(Debug, Clone)]
// pub enum Protocol {
//     TopPair,
//     TopPairResult(Option<Pair>),
//
//     TopPairType(String),
//     TopPairTypeResult(Option<Pair>),
//
//     PushEntry(Entry),
//     PushEntryResult(Result<Pair, HolochainError>),
//
//     PushPair(Pair),
//     PushPairResult(Result<Pair, HolochainError>),
//
//     GetEntry(String),
//     GetEntryResult(Result<Option<Pair>, HolochainError>),
//
//     GetPair(String),
//     GetPairResult(Result<Option<Pair>, HolochainError>),
// }

// impl Into<ActorMsg<Protocol>> for Protocol {
//     fn into(self) -> ActorMsg<Protocol> {
//         ActorMsg::User(self)
//     }
// }

/// anything that can be asked Protocol and block on responses
/// needed to support implementing ask on upstream ActorRef from riker
/// convenience wrappers around chain struct methods
pub trait AskChain {
    fn ask(&self, message: Protocol) -> Protocol;
}

impl AskChain for ActorRef<Protocol> {
    fn ask(&self, message: Protocol) -> Protocol {
        let a = ask(
            &(*SYS),
            self,
            message,
        );
        println!("asking chain");
        block_on(a).unwrap()
    }
}

impl SourceChain for ActorRef<Protocol> {

    fn top_pair(&self) -> Option<Pair> {
        let response = self.ask(Protocol::ChainTopPair);
        unwrap_to!(response => Protocol::ChainTopPairResult).clone()
    }

    fn top_pair_type(&self, t: &str) -> Option<Pair> {
        let response = self.ask(Protocol::ChainTopPairType(t.to_string()));
        unwrap_to!(response => Protocol::ChainTopPairTypeResult).clone()
    }

    fn push_entry(&mut self, entry: &Entry) -> Result<Pair, HolochainError> {
        let response = self.ask(Protocol::ChainPushEntry(entry.clone()));
        unwrap_to!(response => Protocol::ChainPushEntryResult).clone()
    }

    fn get_entry(&self, entry_hash: &str) -> Result<Option<Pair>, HolochainError> {
        let response = self.ask(Protocol::ChainGetEntry(entry_hash.to_string()));
        unwrap_to!(response => Protocol::ChainGetEntryResult).clone()
    }

    fn push_pair(&mut self, pair: &Pair) -> Result<Pair, HolochainError> {
        let response = self.ask(Protocol::ChainPushPair(pair.clone()));
        unwrap_to!(response => Protocol::ChainPushPairResult).clone()
    }

    fn get_pair(&self, k: &str) -> Result<Option<Pair>, HolochainError> {
        let response = self.ask(Protocol::ChainGetPair(k.to_string()));
        unwrap_to!(response => Protocol::ChainGetPairResult).clone()
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

    pub fn actor(chain: Chain) -> BoxActor<Protocol> {
        Box::new(ChainActor::new(chain))
    }

    pub fn props(chain: Chain) -> BoxActorProd<Protocol> {
        Props::new_args(Box::new(ChainActor::actor), chain)
    }

    pub fn new_ref(chain: Chain) -> ActorRef<Protocol> {
        SYS.actor_of(
            ChainActor::props(chain),
            &snowflake::ProcessUniqueId::new().to_string(),
        ).unwrap()
    }

}

impl Actor for ChainActor {
    type Msg = Protocol;

    fn receive(
        &mut self,
        context: &Context<Self::Msg>,
        message: Self::Msg,
        sender: Option<ActorRef<Self::Msg>>,
    ) {
        println!("received {:?}", message);
        sender.try_tell(
            match message {
                Protocol::ChainTopPair => Protocol::ChainTopPairResult(self.chain.top_pair()),
                Protocol::ChainTopPairResult(_) => unreachable!(),

                Protocol::ChainTopPairType(t) => Protocol::ChainTopPairTypeResult(self.chain.top_pair_type(&t)),
                Protocol::ChainTopPairTypeResult(_) => unreachable!(),

                Protocol::ChainPushPair(pair) => Protocol::ChainPushPairResult(self.chain.push_pair(&pair)),
                Protocol::ChainPushPairResult(_) => unreachable!(),

                Protocol::ChainPushEntry(entry) => Protocol::ChainPushEntryResult(self.chain.push_entry(&entry)),
                Protocol::ChainPushEntryResult(_) => unreachable!(),

                Protocol::ChainGetEntry(key) => Protocol::ChainGetEntryResult(self.chain.get_entry(&key)),
                Protocol::ChainGetEntryResult(_) => unreachable!(),

                Protocol::ChainGetPair(key) => Protocol::ChainGetPairResult(self.chain.get_pair(&key)),
                Protocol::ChainGetPairResult(_) => unreachable!(),
                _ => unreachable!(),
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
    use chain::actor::Protocol;

    pub fn test_chain_actor() -> ActorRef<Protocol> {
        ChainActor::new_ref(test_chain())
    }

}
