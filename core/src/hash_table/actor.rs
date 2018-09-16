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
        match self.block_on_ask(Protocol::Setup) {
            Ok(response) => unwrap_to!(response => Protocol::SetupResult).clone(),
            Err(error) => Err(error),
        }
    }

    fn teardown(&mut self) -> Result<(), HolochainError> {
        match self.block_on_ask(Protocol::Teardown) {
            Ok(response) => unwrap_to!(response => Protocol::TeardownResult).clone(),
            Err(error) => Err(error),
        }
    }

    fn put_pair(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        match self.block_on_ask(Protocol::PutPair(pair.clone())) {
            Ok(response) => unwrap_to!(response => Protocol::PutPairResult).clone(),
            Err(error) => Err(error),
        }
    }

    fn pair(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        let response = self.block_on_ask(Protocol::GetPair(key.to_string()))?;
        unwrap_to!(response => Protocol::GetPairResult).clone()
    }

    fn modify_pair(
        &mut self,
        keys: &Keys,
        old_pair: &Pair,
        new_pair: &Pair,
    ) -> Result<(), HolochainError> {
        match self.block_on_ask(Protocol::ModifyPair {
            keys: keys.clone(),
            old_pair: old_pair.clone(),
            new_pair: new_pair.clone(),
        }) {
            Ok(response) => unwrap_to!(response => Protocol::ModifyPairResult).clone(),
            Err(error) => Err(error),
        }
    }

    fn retract_pair(&mut self, keys: &Keys, pair: &Pair) -> Result<(), HolochainError> {
        match self.block_on_ask(Protocol::RetractPair {
            keys: keys.clone(),
            pair: pair.clone(),
        }) {
            Ok(response) => unwrap_to!(response => Protocol::RetractPairResult).clone(),
            Err(error) => Err(error),
        }
    }

    fn assert_pair_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
        match self.block_on_ask(Protocol::AssertMeta(meta.clone())) {
            Ok(response) => unwrap_to!(response => Protocol::AssertMetaResult).clone(),
            Err(error) => Err(error),
        }
    }

    fn pair_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError> {
        let response = self.block_on_ask(Protocol::GetPairMeta(key.to_string()))?;
        unwrap_to!(response => Protocol::GetPairMetaResult).clone()
    }

    fn metas_for_pair(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        let response = self.block_on_ask(Protocol::GetMetasForPair(pair.clone()))?;
        unwrap_to!(response => Protocol::GetMetasForPairResult).clone()
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

                    Protocol::PutPair(pair) => Protocol::PutPairResult(self.table.put_pair(&pair)),

                    Protocol::GetPair(hash) => Protocol::GetPairResult(self.table.pair(&hash)),

                    Protocol::ModifyPair {
                        keys,
                        old_pair,
                        new_pair,
                    } => Protocol::ModifyPairResult(
                        self.table.modify_pair(&keys, &old_pair, &new_pair),
                    ),
                    Protocol::RetractPair { keys, pair } => {
                        Protocol::RetractPairResult(self.table.retract_pair(&keys, &pair))
                    }

                    Protocol::AssertMeta(pair_meta) => {
                        Protocol::AssertMetaResult(self.table.assert_pair_meta(&pair_meta))
                    }

                    Protocol::GetPairMeta(key) => {
                        Protocol::GetPairMetaResult(self.table.pair_meta(&key))
                    }

                    Protocol::GetMetasForPair(pair) => {
                        Protocol::GetMetasForPairResult(self.table.metas_for_pair(&pair))
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
    use hash_table::{
        memory::tests::test_table, pair::tests::test_pair, test_util::standard_suite, HashTable,
    };
    use key::Key;
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

        table_actor.put_pair(&test_pair()).unwrap();

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
            table_actor_thread.put_pair(&pair).unwrap();
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
    fn test_standard_suite() {
        standard_suite(&mut test_table_actor());
    }

}
