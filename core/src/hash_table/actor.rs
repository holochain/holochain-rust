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

}

impl Into<ActorMsg<HashTableProtocol>> for HashTableProtocol {

    fn into(self) -> ActorMsg<HashTableProtocol> {
        ActorMsg::User(self)
    }

}

#[derive(Clone, Debug)]
pub struct HashTableActor<T: HashTable> {
    table: T,
}

impl<T: HashTable> HashTableActor<T> {
    pub fn new(table: T) -> HashTableActor<T> {
        HashTableActor {
            table
        }
    }

    pub fn actor(table: &T) -> BoxActor<HashTableProtocol> {
        Box::new(HashTableActor::new(&table))
    }

    pub fn props(table: &T) -> BoxActorProd<HashTableProtocol> {
        Props::new(Box::new_args(HashTableActor::actor, &table))
    }
}

#[test]
pub mod tests {


}
