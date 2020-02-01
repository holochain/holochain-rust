use crate::{network::entry_with_header::EntryWithHeader, NEW_RELIC_LICENSE_KEY};
use holochain_core_types::entry::Entry;
use holochain_persistence_api::cas::content::Address;

pub trait ValidationDependencies {
    fn get_validation_dependencies(&self) -> Vec<Address>;
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl ValidationDependencies for EntryWithHeader {
    fn get_validation_dependencies(&self) -> Vec<Address> {
        match &self.entry {
            Entry::App(_, _) => {
                // In the future an entry should be dependent its previous header but
                // for now it can require nothing by default.
                // There is also potential to add a WASM function for determining dependencies as a function
                // of the entry content.
                match self.header.link_update_delete() {
                    // If it is an update, require that the original entry is validated
                    Some(entry_to_update) => vec![entry_to_update],
                    None => Vec::new(),
                }
            }
            Entry::LinkAdd(link_data) | Entry::LinkRemove((link_data, _)) => {
                // A link or link remove depends on its base and target being validated
                vec![
                    link_data.link.base().clone(),
                    link_data.link.target().clone(),
                ]
            }
            Entry::ChainHeader(chain_header) => {
                // A chain header entry is dependent on its previous header
                // unless it is the genesis header (link is None)
                chain_header
                    .link()
                    .map(|prev_addr| vec![prev_addr])
                    .unwrap_or_else(Vec::new)
            }
            Entry::Deletion(deletion) => {
                // a deletion depends on the thing being deleted
                vec![deletion.deleted_entry_address().clone()]
            }
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use holochain_core_types::{
        agent::AgentId, chain_header::ChainHeader, link::link_data::LinkData, time::Iso8601,
    };
    use holochain_persistence_api::cas::content::AddressableContent;

    fn test_header_for_entry(entry: &Entry) -> ChainHeader {
        ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &Vec::new(),                                       // provenences
            &Some(Address::from("QmEntryPreviousHeaderHash")), // link
            &None,                                             // link same type
            &None,                                             // link update/delete
            &Iso8601::from(0),
        )
    }

    fn entry_with_header_from_entry(entry: Entry) -> EntryWithHeader {
        let header = test_header_for_entry(&entry);
        EntryWithHeader::new(entry, header)
    }

    #[test]
    fn test_get_validation_dependencies_app_entry() {
        let entry = Entry::App("entry_type".into(), "content".into());
        let entry_wh = entry_with_header_from_entry(entry);
        assert_eq!(entry_wh.get_validation_dependencies(), Vec::new(),)
    }

    #[test]
    fn test_get_validation_dependencies_link_add_entry() {
        let entry = Entry::LinkAdd(LinkData::new_add(
            &Address::from("QmBaseAddress"),
            &Address::from("QmTargetAddress"),
            "some tag",
            "some type",
            test_header_for_entry(&Entry::App("".into(), "".into())),
            AgentId::new("HcAgentId", "key".into()),
        ));
        let entry_wh = entry_with_header_from_entry(entry);
        assert_eq!(
            entry_wh.get_validation_dependencies(),
            vec![
                Address::from("QmBaseAddress"),
                Address::from("QmTargetAddress")
            ],
        )
    }

    #[test]
    fn test_get_validation_dependencies_header_entry() {
        let header_entry_conent = ChainHeader::new(
            &"some type".into(),
            &Address::from("QmAddressOfEntry"),
            &Vec::new(),                                     // provenences
            &Some(Address::from("QmPreviousHeaderAddress")), // link
            &None,                                           // link same type
            &None,                                           // link update/delete
            &Iso8601::from(0),
        );
        let entry = Entry::ChainHeader(header_entry_conent);
        let entry_wh = entry_with_header_from_entry(entry);
        assert_eq!(
            entry_wh.get_validation_dependencies(),
            vec![Address::from("QmPreviousHeaderAddress")],
        )
    }
}
