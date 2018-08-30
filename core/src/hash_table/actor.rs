use actor::{AskSelf, Protocol, SYS};
use agent::keys::Keys;
use error::HolochainError;
use hash_table::{pair::Pair, pair_meta::PairMeta, HashTable};
use riker::actors::*;
use snowflake;

// anything that can be asked of HashTable and block on responses
// needed to support implementing ask on upstream ActorRef from riker
pub trait AskHashTable: HashTable {}

impl AskHashTable for ActorRef<Protocol> {}

impl HashTable for ActorRef<Protocol> {
    fn setup(&mut self) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::Setup);
        unwrap_to!(response => Protocol::SetupResult).clone()
    }

    fn teardown(&mut self) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::Teardown);
        unwrap_to!(response => Protocol::TeardownResult).clone()
    }

    fn commit(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::Commit(pair.clone()));
        unwrap_to!(response => Protocol::CommitResult).clone()
    }

    fn pair(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        let response = self.block_on_ask(Protocol::Pair(key.to_string()));
        unwrap_to!(response => Protocol::PairResult).clone()
    }

    fn modify(
        &mut self,
        keys: &Keys,
        old_pair: &Pair,
        new_pair: &Pair,
    ) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::Modify {
            keys: keys.clone(),
            old_pair: old_pair.clone(),
            new_pair: new_pair.clone(),
        });
        unwrap_to!(response => Protocol::ModifyResult).clone()
    }

    fn retract(&mut self, keys: &Keys, pair: &Pair) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::Retract {
            keys: keys.clone(),
            pair: pair.clone(),
        });
        unwrap_to!(response => Protocol::RetractResult).clone()
    }

    fn assert_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::AssertMeta(meta.clone()));
        unwrap_to!(response => Protocol::AssertMetaResult).clone()
    }

    fn get_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError> {
        let response = self.block_on_ask(Protocol::Meta(key.to_string()));
        unwrap_to!(response => Protocol::MetaResult).clone()
    }

    fn get_pair_meta(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        let response = self.block_on_ask(Protocol::PairMeta(pair.clone()));
        unwrap_to!(response => Protocol::PairMetaResult).clone()
    }
}

#[derive(Clone, Debug)]
pub struct HashTableActor<HT: HashTable> {
    table: HT,
}

impl<HT: HashTable> HashTableActor<HT> {
    /// returns a new HastTablActor struct
    /// internal use for riker, use new_ref instead
    fn new(table: HT) -> HashTableActor<HT> {
        HashTableActor { table }
    }

    /// actor() for riker
    fn actor(table: HT) -> BoxActor<Protocol> {
        Box::new(HashTableActor::new(table))
    }

    /// props() for riker
    fn props(table: HT) -> BoxActorProd<Protocol> {
        Props::new_args(Box::new(HashTableActor::actor), table)
    }

    /// returns a new actor ref for a new HashTableActor in the main actor system
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
        sender
            .try_tell(
                // every Protocol for HashTable maps directly to a method of the same name
                match message {
                    Protocol::Setup => Protocol::SetupResult(self.table.setup()),

                    Protocol::Teardown => Protocol::TeardownResult(self.table.teardown()),

                    Protocol::Commit(pair) => Protocol::CommitResult(self.table.commit(&pair)),

                    Protocol::Pair(hash) => Protocol::PairResult(self.table.pair(&hash)),

                    Protocol::Modify {
                        keys,
                        old_pair,
                        new_pair,
                    } => Protocol::ModifyResult(self.table.modify(&keys, &old_pair, &new_pair)),
                    Protocol::Retract { keys, pair } => {
                        Protocol::RetractResult(self.table.retract(&keys, &pair))
                    }

                    Protocol::AssertMeta(pair_meta) => {
                        Protocol::AssertMetaResult(self.table.assert_meta(&pair_meta))
                    }

                    Protocol::Meta(key) => Protocol::MetaResult(self.table.get_meta(&key)),

                    Protocol::PairMeta(pair) => {
                        Protocol::PairMetaResult(self.table.get_pair_meta(&pair))
                    }

                    _ => unreachable!(),
                },
                Some(context.myself()),
            )
            .expect("could not tell to HashTableActor sender");
    }
}

#[cfg(test)]
pub mod tests {

    use super::HashTableActor;
    use actor::Protocol;
    use hash::tests::test_hash;
    use hash_table::{memory::tests::test_table, pair::tests::test_pair, HashTable};
    use riker::actors::*;
    use std::{sync::mpsc, thread};

    /// dummy table actor ref
    /// every call produces a new actor, not just a new ref to the same actor
    pub fn test_table_actor() -> ActorRef<Protocol> {
        HashTableActor::new_ref(test_table())
    }

    #[test]
    fn round_trip() {
        let mut table_actor = test_table_actor();

        assert_eq!(table_actor.pair(&test_hash()).unwrap(), None,);

        table_actor.commit(&test_pair()).unwrap();

        assert_eq!(
            table_actor.pair(&test_pair().key()).unwrap(),
            Some(test_pair()),
        );
    }

    #[test]
    /// show two things here:
    /// - we can clone some stateful thing (i.e. actor ref) and mutate one clone and have that
    ///   consistent across all the clones
    /// - we can send the cloned stateful thing into threads and have them see a consistent world
    ///   view without juggling direct message passing through channels
    fn test_round_trip_threads() {
        let table_actor = test_table_actor();

        let table_actor_thread = table_actor.clone();
        let (tx1, rx1) = mpsc::channel();
        thread::spawn(move || {
            assert_eq!(table_actor_thread.pair(&test_hash()).unwrap(), None,);
            // kick off the next thread
            tx1.send(true).unwrap();
        });

        // mutate this clone of the original actor ref
        let mut table_actor_thread = table_actor.clone();
        let (tx2, rx2) = mpsc::channel();
        thread::spawn(move || {
            rx1.recv().unwrap();
            let pair = test_pair();
            table_actor_thread.commit(&pair).unwrap();
            // push the committed pair through to the next thread
            tx2.send(pair).unwrap();
        });

        let table_actor_thread = table_actor.clone();
        let handle = thread::spawn(move || {
            let pair = rx2.recv().unwrap();
            assert_eq!(table_actor_thread.pair(&pair.key()).unwrap(), Some(pair),);
        });

        handle.join().unwrap();
    }

    #[test]
    fn hash_table_suite() {
        // @TODO there is a suite of standard HashTable tests coming
        // @see https://github.com/holochain/holochain-rust/pull/246
    }

}
