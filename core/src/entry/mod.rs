//! This module extends Entry and EntryType with the CanPublish trait.

use holochain_core_types::entry::entry_type::EntryType;
use crate::context::Context;

pub trait CanPublish {
    fn can_publish(&self, context:&Context) -> bool;
}

impl CanPublish for EntryType {

    fn can_publish(&self, context:&Context) -> bool {

       match self {
            EntryType::Dna => return false,
            EntryType::CapTokenGrant => return false,
            _ => ()
        }

        let dna = context
            .get_dna()
            .expect("context must hold DNA in order to publish an entry.");
        let maybe_def = dna.get_entry_type_def(self.to_string().as_str());

        if maybe_def.is_none() {
            context.log("context must hold an entry type definition to publish an entry.");
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
