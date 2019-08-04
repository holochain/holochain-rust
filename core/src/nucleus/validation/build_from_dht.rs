use crate::{
    context::Context,
    network::{
        entry_with_header::EntryWithHeader,
        actions::get_entry::get_entry,
    },
    nucleus::{
        ribosome::callback::{
            validation_package::get_validation_package_definition, CallbackResult,
        },
    },
    entry::CanPublish,
};
use holochain_core_types::{
    error::HolochainError,
    validation::{ValidationPackage, ValidationPackageDefinition},
    entry::{Entry, 
        EntryWithMetaAndHeader, EntryWithMeta
    },
    chain_header::ChainHeader,
    time::Timeout,
};

use std::sync::Arc;

const GET_TIMEOUT_MS: usize = 500;

async fn all_chain_headers_before_header_dht(
    context: &Arc<Context>,
    header: &ChainHeader,
) -> Result<Vec<ChainHeader>, HolochainError> {
    let mut current_header = header.clone();
    let mut headers = Vec::new();

    while let Some(next_header_addr) = current_header.link() {
        let get_entry_result = await!(get_entry(context.clone(), next_header_addr.clone(), Timeout::new(GET_TIMEOUT_MS)));
        if let Ok(Some(EntryWithMetaAndHeader{entry_with_meta: EntryWithMeta{entry: Entry::ChainHeader(chain_header), ..}, ..})) = get_entry_result {
            headers.push(chain_header.clone());
            current_header = chain_header;
        } else {
            return Err(HolochainError::ErrorGeneric(
                format!("When building validation package from DHT, Could not retrieve a header entry at address: {:?}", next_header_addr))
            )
        }
    }
    Ok(headers)
}

async fn public_chain_entries_from_headers_dht(
    context: &Arc<Context>,
    headers: &[ChainHeader],
) -> Result<Vec<Entry>, HolochainError> {
    let public_headers = headers
        .iter()
        .filter(|ref chain_header| chain_header.entry_type().can_publish(context))
        .collect::<Vec<_>>();
    let mut entries = Vec::new();
    for header in public_headers {
        let get_entry_result = await!(get_entry(context.clone(), header.entry_address().clone(), Timeout::new(GET_TIMEOUT_MS)));
        if let Ok(Some(EntryWithMetaAndHeader{entry_with_meta: EntryWithMeta{entry, ..}, ..})) = get_entry_result {
            entries.push(entry.clone());
        } else {
            return Err(HolochainError::ErrorGeneric(
                format!("When building validation package from DHT, Could not retrieve entry at address: {:?}", header.entry_address()))
            )
        }
    }
    Ok(entries)
}

pub (crate) async fn try_make_validation_package_dht(
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<ValidationPackage, HolochainError> {
    context.log(format!("Constructing validation package from DHT for entry with address: {}", entry_with_header.header.entry_address()));

    let entry = &entry_with_header.entry;
    let entry_header = entry_with_header.header.clone();

    let validation_package_definition = match get_validation_package_definition(entry, context.clone())? {
        CallbackResult::ValidationPackageDefinition(def) => Ok(def),
        CallbackResult::Fail(error_string) => Err(HolochainError::ErrorGeneric(error_string)),
        CallbackResult::NotImplemented(reason) => Err(HolochainError::ErrorGeneric(format!(
            "ValidationPackage callback not implemented for {:?} ({})",
            entry.entry_type().clone(),
            reason
        ))),
        _ => unreachable!(),
    }?;

    let chain_headers = await!(all_chain_headers_before_header_dht(&context, &entry_header))?;

    let mut package = ValidationPackage::only_header(entry_header.clone());

    match validation_package_definition {
        ValidationPackageDefinition::Entry => {
            // this should never happen but it will produce the correct package anyway
        }
        ValidationPackageDefinition::ChainEntries => {
            package.source_chain_entries = Some(await!(public_chain_entries_from_headers_dht(&context, &chain_headers))?);
        }
        ValidationPackageDefinition::ChainHeaders => {
            package.source_chain_headers = Some(chain_headers)
        }
        ValidationPackageDefinition::ChainFull => {
            package.source_chain_headers = Some(chain_headers.clone());
            package.source_chain_entries = Some(await!(public_chain_entries_from_headers_dht(&context, &chain_headers))?);
        }
        ValidationPackageDefinition::Custom(string) => {
            package.custom = Some(string)
        }
    };
    Ok(package)
}

#[cfg(test)]
pub mod tests {
    
    use super::*;
    use holochain_core_types::entry::test_entry_with_value;
    use crate::workflows::author_entry::author_entry;
    use crate::nucleus::actions::tests::*;
    use std::{thread, time};
    // use holochain_persistence_api::cas::content::AddressableContent;

    #[test]
    fn test_get_all_chain_headers_returns_same_as_local_chain() {
        let mut dna = test_dna();
        dna.uuid = "test_get_all_chain_headers_returns_same_as_local_chain".to_string();
        let netname = Some("test_get_all_chain_headers_returns_same_as_local_chain, the network");
        let (_instance1, context) = instance_by_name("jill", dna.clone(), netname);

        let _entry_address = context
            .block_on(author_entry(
                &test_entry_with_value("{\"stuff\":\"test entry value\"}"),
                None,
                &context,
                &vec![],
            ))
            .unwrap()
            .address();

        thread::sleep(time::Duration::from_millis(500));

        // collect the local chain
        let mut local_chain_headers: Vec<ChainHeader> = context.state().unwrap()
            .agent()
            .iter_chain()
            .collect();
        let top_header = local_chain_headers.remove(0);


        // reconstruct from published headers
        let reconstructed = context.block_on(
            all_chain_headers_before_header_dht(&context, &top_header)
        ).expect("Could not get headers from DHT");

        assert_eq!(local_chain_headers.len(), 2);
        assert_eq!(reconstructed.len(), 2);

        assert_eq!(
            local_chain_headers,
            reconstructed
        );
    }
    
}
