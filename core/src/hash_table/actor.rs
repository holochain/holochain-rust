use riker::actors::*;
use hash_table::HashTable;

#[derive(Debug, Clone)]
pub enum HashTableProtocol {

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
