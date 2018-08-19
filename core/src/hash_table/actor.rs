use agent::keys::Keys;
use error::HolochainError;
use futures::executor::block_on;
use hash_table::{pair::Pair, pair_meta::PairMeta, HashTable};
use riker::actors::*;
use riker_patterns::ask::ask;
use snowflake;
// use riker::kernel::Dispatcher;
// use futures::{Future};
// use riker::futures_util::spawn;
// use riker_default::DeadLettersActor;
// use riker_default::BasicTimer;
// use riker_default::MapVec;
// use riker_default::SimpleLogger;
// use riker::system::NoIo;
// use riker_patterns::ask::Ask;
// use futures::channel::oneshot::Canceled;
// use futures::Async::Ready;
use actor::{Protocol, SYS};
// use futures::executor::spawn_with_handle;
// use futures::executor::SpawnWithHandle;

// struct HashTableModel;
//
// // @see https://github.com/riker-rs/riker/blob/master/riker-default/riker-dispatcher/src/lib.rs
// pub struct HashTableDispatcher {
//     inner: ThreadPool,
// }
//
// impl Dispatcher for HashTableDispatcher {
//     fn new(_config: &Config, _: bool) -> HashTableDispatcher {
//         HashTableDispatcher {
//             inner: ThreadPoolBuilder::new()
//                                         .pool_size(4)
//                                         .name_prefix("pool-thread-hash-table-#")
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
// impl Model for HashTableModel {
//     type Msg = Protocol;
//     type Dis = HashTableDispatcher;
//     type Ded = DeadLettersActor<Self::Msg>;
//     type Tmr = BasicTimer<Self::Msg>;
//     type Evs = MapVec<Self::Msg>;
//     type Tcp = NoIo<Self::Msg>;
//     type Udp = NoIo<Self::Msg>;
//     type Log = SimpleLogger<Self::Msg>;
// }

// lazy_static! {
//     pub static ref HASH_TABLE_SYS: ActorSystem<Protocol> HashTable= {
//         let hash_table_model: DefaultModel<Protocol> HashTable= DefaultModel::new();
//         // let hash_table_model = HashTableModel{};
//         ActorSystem::new(&hash_table_model).unwrap()
//     };
// }

// impl Into<ActorMsg<Protocol>> for Protocol {
//     fn into(self) -> ActorMsg<Protocol> {
//         ActorMsg::User(self)
//     }
// }

// type HTAsk = Box<Future<Item=Protocol, Error=Canceled> + Send>;

/// anything that can be asked Protocol aHashTablend block on responses
/// needed to support implementing ask on upstream ActorRef from riker
pub trait AskHashTable {
    fn ask(&self, message: Protocol) -> Protocol;
}

impl AskHashTable for ActorRef<Protocol> {
    fn ask(&self, message: Protocol) -> Protocol {
        let a = ask(&(*SYS), self, message);
        // loop {
        //     match a.poll(context)? {
        //         Ready(v) => break v,
        //         _ => println!("polling"),
        //     }
        // }
        // println!("{:?}", &(*HASH_TABLE_SYS);
        // println!("asking table");
        block_on(a).unwrap()
        // spawn_with_handle(a)
    }
}

impl HashTable for ActorRef<Protocol> {
    fn setup(&mut self) -> Result<(), HolochainError> {
        let response = self.ask(Protocol::HashTableSetup);
        unwrap_to!(response => Protocol::HashTableSetupResult).clone()
    }

    fn teardown(&mut self) -> Result<(), HolochainError> {
        let response = self.ask(Protocol::HashTableTeardown);
        unwrap_to!(response => Protocol::HashTableTeardownResult).clone()
    }

    fn commit(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        let response = self.ask(Protocol::HashTableCommit(pair.clone()));
        unwrap_to!(response => Protocol::HashTableCommitResult).clone()
    }

    fn get(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        let response = self.ask(Protocol::HashTableGetPair(key.to_string()));
        unwrap_to!(response => Protocol::HashTableGetPairResult).clone()
    }

    fn modify(
        &mut self,
        keys: &Keys,
        old_pair: &Pair,
        new_pair: &Pair,
    ) -> Result<(), HolochainError> {
        let response = self.ask(Protocol::HashTableModify {
            keys: keys.clone(),
            old_pair: old_pair.clone(),
            new_pair: new_pair.clone(),
        });
        unwrap_to!(response => Protocol::HashTableModifyResult).clone()
    }

    fn retract(&mut self, keys: &Keys, pair: &Pair) -> Result<(), HolochainError> {
        let response = self.ask(Protocol::HashTableRetract {
            keys: keys.clone(),
            pair: pair.clone(),
        });
        unwrap_to!(response => Protocol::HashTableRetractResult).clone()
    }

    fn assert_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
        let response = self.ask(Protocol::HashTableAssertMeta(meta.clone()));
        unwrap_to!(response => Protocol::HashTableAssertMetaResult).clone()
    }

    fn get_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError> {
        let response = self.ask(Protocol::HashTableGetMeta(key.to_string()));
        unwrap_to!(response => Protocol::HashTableGetMetaResult).clone()
    }

    fn get_pair_meta(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        let response = self.ask(Protocol::HashTableGetPairMeta(pair.clone()));
        unwrap_to!(response => Protocol::HashTableGetPairMetaResult).clone()
    }
}

#[derive(Clone, Debug)]
pub struct HashTableActor<HT: HashTable> {
    table: HT,
}

impl<HT: HashTable> HashTableActor<HT> {
    pub fn new(table: HT) -> HashTableActor<HT> {
        HashTableActor { table }
    }

    pub fn actor(table: HT) -> BoxActor<Protocol> {
        Box::new(HashTableActor::new(table))
    }

    pub fn props(table: HT) -> BoxActorProd<Protocol> {
        Props::new_args(Box::new(HashTableActor::actor), table)
    }

    pub fn new_ref(table: HT) -> ActorRef<Protocol> {
        SYS.actor_of(
            HashTableActor::props(table),
            &snowflake::ProcessUniqueId::new().to_string(),
        ).unwrap()
    }
}

impl<HT: HashTable> Actor for HashTableActor<HT> {
    type Msg = Protocol;

    fn receive(
        &mut self,
        context: &Context<Self::Msg>,
        message: Self::Msg,
        sender: Option<ActorRef<Self::Msg>>,
    ) {
        println!("received {:?}", message);

        sender
            .try_tell(
                match message {
                    Protocol::HashTableSetup => Protocol::HashTableSetupResult(self.table.setup()),
                    Protocol::HashTableSetupResult(_) => unreachable!(),

                    Protocol::HashTableTeardown => {
                        Protocol::HashTableTeardownResult(self.table.teardown())
                    }
                    Protocol::HashTableTeardownResult(_) => unreachable!(),

                    Protocol::HashTableCommit(pair) => {
                        Protocol::HashTableCommitResult(self.table.commit(&pair))
                    }
                    Protocol::HashTableCommitResult(_) => unreachable!(),

                    Protocol::HashTableGetPair(hash) => {
                        Protocol::HashTableGetPairResult(self.table.get(&hash))
                    }
                    Protocol::HashTableGetPairResult(_) => unreachable!(),

                    Protocol::HashTableModify {
                        keys,
                        old_pair,
                        new_pair,
                    } => Protocol::HashTableModifyResult(
                        self.table.modify(&keys, &old_pair, &new_pair),
                    ),
                    Protocol::HashTableModifyResult(_) => unreachable!(),

                    Protocol::HashTableRetract { keys, pair } => {
                        Protocol::HashTableRetractResult(self.table.retract(&keys, &pair))
                    }
                    Protocol::HashTableRetractResult(_) => unreachable!(),

                    Protocol::HashTableAssertMeta(pair_meta) => {
                        Protocol::HashTableAssertMetaResult(self.table.assert_meta(&pair_meta))
                    }
                    Protocol::HashTableAssertMetaResult(_) => unreachable!(),

                    Protocol::HashTableGetMeta(key) => {
                        Protocol::HashTableGetMetaResult(self.table.get_meta(&key))
                    }
                    Protocol::HashTableGetMetaResult(_) => unreachable!(),

                    Protocol::HashTableGetPairMeta(pair) => {
                        Protocol::HashTableGetPairMetaResult(self.table.get_pair_meta(&pair))
                    }
                    Protocol::HashTableGetPairMetaResult(_) => unreachable!(),

                    _ => unreachable!(),
                },
                Some(context.myself()),
            )
            .unwrap();
    }
}

#[cfg(test)]
pub mod tests {

    use super::HashTableActor;
    use hash_table::{actor::Protocol, memory::tests::test_table};
    use riker::actors::*;

    pub fn test_table_actor() -> ActorRef<Protocol> {
        HashTableActor::new_ref(test_table())
    }

    #[test]
    fn round_trip() {}

}
