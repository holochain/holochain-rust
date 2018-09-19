use actor::{AskSelf, Protocol, SYS};
use agent::keys::Keys;
use error::HolochainError;
use hash::HashString;
use hash_table::{entry::Entry, meta::EntryMeta, HashTable};
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

    fn put_entry(&mut self, entry: &Entry) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::PutEntry(entry.clone()));
        unwrap_to!(response => Protocol::PutEntryResult).clone()
    }

    fn entry(&self, key: &HashString) -> Result<Option<Entry>, HolochainError> {
        let response = self.block_on_ask(Protocol::GetEntry(key.clone()));
        unwrap_to!(response => Protocol::GetEntryResult).clone()
    }

    fn modify_entry(
        &mut self,
        keys: &Keys,
        old_entry: &Entry,
        new_entry: &Entry,
    ) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::ModifyEntry {
            keys: keys.clone(),
            old: old_entry.clone(),
            new: new_entry.clone(),
        });
        unwrap_to!(response => Protocol::ModifyEntryResult).clone()
    }

    fn retract_entry(&mut self, keys: &Keys, entry: &Entry) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::RetractEntry {
            keys: keys.clone(),
            entry: entry.clone(),
        });
        unwrap_to!(response => Protocol::RetractEntryResult).clone()
    }

    fn assert_meta(&mut self, meta: &EntryMeta) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::AssertMeta(meta.clone()));
        unwrap_to!(response => Protocol::AssertMetaResult).clone()
    }

    fn get_meta(&mut self, key: &HashString) -> Result<Option<EntryMeta>, HolochainError> {
        let response = self.block_on_ask(Protocol::GetMeta(key.clone()));
        unwrap_to!(response => Protocol::GetMetaResult).clone()
    }

    fn metas_from_entry(&mut self, entry: &Entry) -> Result<Vec<EntryMeta>, HolochainError> {
        let response = self.block_on_ask(Protocol::MetasFromEntry(entry.clone()));
        unwrap_to!(response => Protocol::MetasFromEntryResult).clone()
    }

    fn meta_from_request(
        &mut self,
        entry_hash: HashString,
        attribute_name: &str,
    ) -> Result<Option<EntryMeta>, HolochainError> {
        let response = self.block_on_ask(Protocol::MetaFromRequest {
            entry_hash: entry_hash,
            attribute_name: attribute_name.to_string(),
        });
        unwrap_to!(response => Protocol::MetaFromRequestResult).clone()
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

                    Protocol::PutEntry(entry) => {
                        Protocol::PutEntryResult(self.table.put_entry(&entry))
                    }

                    Protocol::GetEntry(hash) => Protocol::GetEntryResult(self.table.entry(&hash)),

                    Protocol::ModifyEntry { keys, old, new } => {
                        Protocol::ModifyEntryResult(self.table.modify_entry(&keys, &old, &new))
                    }

                    Protocol::RetractEntry { keys, entry } => {
                        Protocol::RetractEntryResult(self.table.retract_entry(&keys, &entry))
                    }

                    Protocol::AssertMeta(meta) => {
                        Protocol::AssertMetaResult(self.table.assert_meta(&meta))
                    }

                    Protocol::GetMeta(key) => Protocol::GetMetaResult(self.table.get_meta(&key)),

                    Protocol::MetasFromEntry(entry) => {
                        Protocol::MetasFromEntryResult(self.table.metas_from_entry(&entry))
                    }

                    Protocol::MetaFromRequest {
                        entry_hash,
                        attribute_name,
                    } => Protocol::MetaFromRequestResult(
                        self.table.meta_from_request(entry_hash, &attribute_name),
                    ),

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
        entry::tests::test_entry, memory::tests::test_table, test_util::standard_suite, HashTable,
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

        assert_eq!(table_actor.entry(&test_hash()).unwrap(), None);

        table_actor.put_entry(&test_entry()).unwrap();

        assert_eq!(
            table_actor.entry(&test_entry().key()).unwrap().unwrap(),
            test_entry(),
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
            assert_eq!(table_actor_thread.entry(&test_hash()).unwrap(), None);
            // kick off the next thread
            tx1.send(true).unwrap();
        });

        // mutate this clone of the original actor ref
        let mut table_actor_thread = table_actor.clone();
        let (tx2, rx2) = mpsc::channel();
        thread::spawn(move || {
            rx1.recv().unwrap();

            let entry = test_entry();
            table_actor_thread.put_entry(&entry).unwrap();

            // push the committed entry through to the next thread
            tx2.send(entry).unwrap();
        });

        let table_actor_thread = table_actor.clone();
        let handle = thread::spawn(move || {
            let entry = rx2.recv().unwrap();
            assert_eq!(
                table_actor_thread.entry(&entry.key()).unwrap().unwrap(),
                test_entry(),
            );
        });

        handle.join().unwrap();
    }

    #[test]
    fn test_standard_suite() {
        standard_suite(&mut test_table_actor());
    }

}
