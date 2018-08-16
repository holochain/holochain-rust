use riker::actors::*;
use hash_table::HashTable;
use riker_default::DefaultModel;
use hash_table::pair::Pair;
use error::HolochainError;
use futures::executor::block_on;
use riker_patterns::ask::ask;

lazy_static! {
    pub static ref HASH_TABLE_SYS: ActorSystem<HashTableProtocol> = {
        let hash_table_model: DefaultModel<HashTableProtocol> = DefaultModel::new();
        ActorSystem::new(&hash_table_model).unwrap()
    };
}

/// anything that can be asked HashTableProtocol and block on responses
/// needed to support implementing ask on upstream ActorRef from riker
pub trait AskHashTable {

    fn ask(&self, message: HashTableProtocol) -> HashTableProtocol;

}

impl AskHashTable for ActorRef<HashTableProtocol> {
    fn ask(&self, message: HashTableProtocol) -> HashTableProtocol {
        block_on(
            ask(
                &(*HASH_TABLE_SYS),
                self,
                message,
            )
        ).unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum HashTableProtocol {
    /// HashTable::get()
    GetPair(String),
    GetPairResult(Result<Option<Pair>, HolochainError>),

    /// HashTable::commit()
    Commit(Pair),
    CommitResponse(Result<(), HolochainError>),

}

impl Into<ActorMsg<HashTableProtocol>> for HashTableProtocol {

    fn into(self) -> ActorMsg<HashTableProtocol> {
        ActorMsg::User(self)
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
            "table",
        ).unwrap()
    }

}

impl<HT: HashTable> Actor for HashTableActor<HT> {
    type Msg = HashTableProtocol;

    fn receive(
        &mut self,
        _context: &Context<Self::Msg>,
        _message: Self::Msg,
        _sender: Option<ActorRef<Self::Msg>>,
    ) {

    }

}

#[cfg(test)]
pub mod tests {

    use super::HashTableActor;
    use hash_table::memory::tests::test_table;
    use riker::actors::*;
    use hash_table::actor::HashTableProtocol;

    pub fn test_table_actor() -> ActorRef<HashTableProtocol> {
        HashTableActor::new_ref(test_table());
    }

    #[test]
    fn round_trip() {

    }

}
