//! This module contains all the necessary definitions for Entry, which broadly speaking
//! refers to any data which will be written into the ContentAddressableStorage, or the EntityAttributeValueStorage.
//! It defines serialization behaviour for entries. Here you can find the complete list of
//! entry_types, and special entries, like deletion_entry and cap_entry.

use holochain_core_types::entry::Entry;
use crate::context::Context;

trait CanPublish {
    fn can_publish(&self, context:&Context) -> bool;
}

impl CanPublish for Entry {

    fn can_publish(&self, context:&Context) -> bool {

        let entry_type = self.entry_type().clone();

        if !entry_type.can_publish() {
            return false;
        }

        let dna = context
            .get_dna()
            .expect("context must hold DNA in order to publish an entry.");
        let maybe_def = dna.get_entry_type_def(entry_type.to_string().as_str());

        if maybe_def.is_none() {
            // TODO #439 - Log the error. Once we have better logging.
            return false;
        }
        let entry_type_def = maybe_def.unwrap();

        // app entry type must be publishable
        if !entry_type_def.sharing.clone().can_publish() {
            return false;
        }

        true
    }
}
