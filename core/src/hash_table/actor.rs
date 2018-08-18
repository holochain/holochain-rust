use riker::actors::*;
use hash_table::HashTable;
// use riker_default::DefaultModel;
use hash_table::pair::Pair;
use error::HolochainError;
use futures::executor::block_on;
use riker_patterns::ask::ask;
use hash_table::pair_meta::PairMeta;
use agent::keys::Keys;
use snowflake;
use riker::kernel::Dispatcher;
use config::Config;
use futures::executor::{ThreadPool, ThreadPoolBuilder};
use futures::{Future, Never};
use riker::futures_util::spawn;
use riker_default::DeadLettersActor;
use riker_default::BasicTimer;
use riker_default::MapVec;
use riker_default::SimpleLogger;
use riker::system::NoIo;

struct HashTableModel;

// @see https://github.com/riker-rs/riker/blob/master/riker-default/riker-dispatcher/src/lib.rs
pub struct HashTableDispatcher {
    inner: ThreadPool,
}

impl Dispatcher for HashTableDispatcher {
    fn new(_config: &Config, _: bool) -> HashTableDispatcher {
        HashTableDispatcher {
            inner: ThreadPoolBuilder::new()
                                        .pool_size(4)
                                        .name_prefix("pool-thread-hash-table-#")
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

impl Model for HashTableModel {
    type Msg = HashTableProtocol;
    type Dis = HashTableDispatcher;
    type Ded = DeadLettersActor<Self::Msg>;
    type Tmr = BasicTimer<Self::Msg>;
    type Evs = MapVec<Self::Msg>;
    type Tcp = NoIo<Self::Msg>;
    type Udp = NoIo<Self::Msg>;
    type Log = SimpleLogger<Self::Msg>;
}

lazy_static! {
    pub static ref HASH_TABLE_SYS: ActorSystem<HashTableProtocol> = {
        // let hash_table_model: DefaultModel<HashTableProtocol> = DefaultModel::new();
        let hash_table_model = HashTableModel{};
        ActorSystem::new(&hash_table_model).unwrap()
    };
}

#[derive(Debug, Clone)]
pub enum HashTableProtocol {
    /// HashTable::setup()
    Setup,
    SetupResult(Result<(), HolochainError>),

    /// HashTable::teardown()
    Teardown,
    TeardownResult(Result<(), HolochainError>),

    /// HashTable::modify()
    Modify{
        keys: Keys,
        old_pair: Pair,
        new_pair: Pair,
    },
    ModifyResult(Result<(), HolochainError>),

    /// HashTable::retract()
    Retract{
        keys: Keys,
        pair: Pair,
    },
    RetractResult(Result<(), HolochainError>),

    /// HashTable::assert_meta()
    AssertMeta(PairMeta),
    AssertMetaResult(Result<(), HolochainError>),

    /// HashTable::get_meta()
    GetMeta(String),
    GetMetaResult(Result<Option<PairMeta>, HolochainError>),

    /// HashTable::get_pair_meta()
    GetPairMeta(Pair),
    GetPairMetaResult(Result<Vec<PairMeta>, HolochainError>),

    /// HashTable::get()
    GetPair(String),
    GetPairResult(Result<Option<Pair>, HolochainError>),

    /// HashTable::commit()
    Commit(Pair),
    CommitResult(Result<(), HolochainError>),

}

impl Into<ActorMsg<HashTableProtocol>> for HashTableProtocol {
    fn into(self) -> ActorMsg<HashTableProtocol> {
        ActorMsg::User(self)
    }
}

/// anything that can be asked HashTableProtocol and block on responses
/// needed to support implementing ask on upstream ActorRef from riker
pub trait AskHashTable {
    fn ask(&self, message: HashTableProtocol) -> HashTableProtocol;
}

impl AskHashTable for ActorRef<HashTableProtocol> {
    fn ask(&self, message: HashTableProtocol) -> HashTableProtocol {
        let a = ask(
            &(*HASH_TABLE_SYS),
            self,
            message,
        );
        println!("asking table");
        block_on(a).unwrap()
    }
}

impl HashTable for ActorRef<HashTableProtocol> {
    fn setup(&mut self) -> Result<(), HolochainError> {
        let response = self.ask(HashTableProtocol::Setup);
        unwrap_to!(response => HashTableProtocol::SetupResult).clone()
    }

    fn teardown(&mut self) -> Result<(), HolochainError> {
        let response = self.ask(HashTableProtocol::Teardown);
        unwrap_to!(response => HashTableProtocol::TeardownResult).clone()
    }

    fn commit(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        let response = self.ask(HashTableProtocol::Commit(pair.clone()));
        unwrap_to!(response => HashTableProtocol::CommitResult).clone()
    }

    fn get(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        let response = self.ask(HashTableProtocol::GetPair(key.to_string()));
        unwrap_to!(response => HashTableProtocol::GetPairResult).clone()
    }

    fn modify(&mut self, keys: &Keys, old_pair: &Pair, new_pair: &Pair,) -> Result<(), HolochainError> {
        let response = self.ask(HashTableProtocol::Modify{
            keys: keys.clone(),
            old_pair: old_pair.clone(),
            new_pair: new_pair.clone(),
        });
        unwrap_to!(response => HashTableProtocol::ModifyResult).clone()
    }

    fn retract(&mut self, keys: &Keys, pair: &Pair) -> Result<(), HolochainError> {
        let response = self.ask(HashTableProtocol::Retract{
            keys: keys.clone(),
            pair: pair.clone(),
        });
        unwrap_to!(response => HashTableProtocol::RetractResult).clone()
    }

    fn assert_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
        let response = self.ask(HashTableProtocol::AssertMeta(meta.clone()));
        unwrap_to!(response => HashTableProtocol::AssertMetaResult).clone()
    }

    fn get_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError> {
        let response = self.ask(HashTableProtocol::GetMeta(key.to_string()));
        unwrap_to!(response => HashTableProtocol::GetMetaResult).clone()
    }

    fn get_pair_meta(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        let response = self.ask(HashTableProtocol::GetPairMeta(pair.clone()));
        unwrap_to!(response => HashTableProtocol::GetPairMetaResult).clone()
    }

}

#[derive(Clone, Debug)]
pub struct HashTableActor<HT: HashTable> {
    table: HT,
}

impl<HT: HashTable> HashTableActor<HT> {

    pub fn new (table: HT) -> HashTableActor<HT> {
        HashTableActor {
            table
        }
    }

    pub fn actor(table: HT) -> BoxActor<HashTableProtocol> {
        Box::new(HashTableActor::new(table))
    }

    pub fn props(table: HT) -> BoxActorProd<HashTableProtocol> {
        Props::new_args(Box::new(HashTableActor::actor), table)
    }

    pub fn new_ref(table: HT) -> ActorRef<HashTableProtocol> {
        HASH_TABLE_SYS.actor_of(
            HashTableActor::props(table),
            &snowflake::ProcessUniqueId::new().to_string(),
        ).unwrap()
    }

}

impl<HT: HashTable> Actor for HashTableActor<HT> {
    type Msg = HashTableProtocol;

    fn receive(
        &mut self,
        context: &Context<Self::Msg>,
        message: Self::Msg,
        sender: Option<ActorRef<Self::Msg>>,
    ) {
        println!("received {:?}", message);

        sender.try_tell(
            match message {
                HashTableProtocol::Setup => HashTableProtocol::SetupResult(self.table.setup()),
                HashTableProtocol::SetupResult(_) => unreachable!(),

                HashTableProtocol::Teardown => HashTableProtocol::TeardownResult(self.table.teardown()),
                HashTableProtocol::TeardownResult(_) => unreachable!(),

                HashTableProtocol::Commit(pair) => HashTableProtocol::CommitResult(self.table.commit(&pair)),
                HashTableProtocol::CommitResult(_) => unreachable!(),

                HashTableProtocol::GetPair(hash) => HashTableProtocol::GetPairResult(self.table.get(&hash)),
                HashTableProtocol::GetPairResult(_) => unreachable!(),

                HashTableProtocol::Modify{keys, old_pair, new_pair} => HashTableProtocol::ModifyResult(self.table.modify(&keys, &old_pair, &new_pair)),
                HashTableProtocol::ModifyResult(_) => unreachable!(),

                HashTableProtocol::Retract{keys, pair} => HashTableProtocol::RetractResult(self.table.retract(&keys, &pair)),
                HashTableProtocol::RetractResult(_) => unreachable!(),

                HashTableProtocol::AssertMeta(pair_meta) => HashTableProtocol::AssertMetaResult(self.table.assert_meta(&pair_meta)),
                HashTableProtocol::AssertMetaResult(_) => unreachable!(),

                HashTableProtocol::GetMeta(key) => HashTableProtocol::GetMetaResult(self.table.get_meta(&key)),
                HashTableProtocol::GetMetaResult(_) => unreachable!(),

                HashTableProtocol::GetPairMeta(pair) => HashTableProtocol::GetPairMetaResult(self.table.get_pair_meta(&pair)),
                HashTableProtocol::GetPairMetaResult(_) => unreachable!(),
            },
            Some(context.myself()),
        ).unwrap();

    }

}

#[cfg(test)]
pub mod tests {

    use super::HashTableActor;
    use hash_table::memory::tests::test_table;
    use riker::actors::*;
    use hash_table::actor::HashTableProtocol;

    pub fn test_table_actor() -> ActorRef<HashTableProtocol> {
        HashTableActor::new_ref(test_table())
    }

    #[test]
    fn round_trip() {

    }

}
