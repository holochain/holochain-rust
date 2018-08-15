use riker::actors::*;
use hash_table::HashTable;
use riker_default::DefaultModel;
use hash_table::pair::Pair;
use error::HolochainError;

lazy_static! {
    pub static ref HASH_TABLE_SYS: ActorSystem<HashTableProtocol> = {
        let hash_table_model: DefaultModel<HashTableProtocol> = DefaultModel::new();
        ActorSystem::new(&hash_table_model).unwrap()
    };
}

#[derive(Debug, Clone)]
pub enum HashTableProtocol {
    /// HashTable::get()
    Get(String),
    GetResponse(Result<Option<Pair>, HolochainError>),

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

    #[test]
    fn round_trip() {

    }

}
