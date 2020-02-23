use crate::{
    agent::{self, find_chain_header},
    content_store::GetContent,
    workflows::{
        callback::validation_package::get_validation_package_definition,
    },
    wasm_engine::callback::CallbackResult,
    context::Context,
    entry::CanPublish,
    state::{State, StateWrapper},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::{
    chain_header::ChainHeader,
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
    signature::Provenance,
    validation::{ValidationPackage, ValidationPackageDefinition::*},
};
use std::{sync::Arc, vec::Vec};

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn build_validation_package<'a>(
    entry: &'a Entry,
    context: Arc<Context>,
    provenances: &'a Vec<Provenance>,
) -> Result<ValidationPackage, HolochainError> {
    match entry.entry_type() {
        EntryType::App(app_entry_type) => {
            if context
                .state()
                .expect("No state in build_validation_package")
                .nucleus()
                .dna()
                .expect("No DNA in build_valdation_package")
                .get_zome_name_for_app_entry_type(&app_entry_type)
                .is_none()
            {
                return Err(HolochainError::ValidationFailed(format!(
                    "Unknown app entry type '{}'",
                    String::from(app_entry_type),
                )));
            }
        }

        EntryType::LinkAdd => {
            // LinkAdd can always be validated
        }

        EntryType::LinkRemove => {
            // LinkAdd can always be validated
        }

        EntryType::Deletion => {
            // FIXME
        }

        EntryType::CapTokenGrant => {
            // FIXME
        }

        EntryType::AgentId => {
            // FIXME
        }
        _ => {
            return Err(HolochainError::ValidationFailed(format!(
                "Attempted to validate system entry type {:?}",
                entry.entry_type(),
            )));
        }
    };

    let entry = entry.clone();
    let context = context;
    let maybe_entry_header = find_chain_header(
        &entry.clone(),
        &context
            .state()
            .expect("No state in build_validation_package"),
    );
    let entry_header = match maybe_entry_header {
        None => {
            // TODO: make sure that we don't run into race conditions with respect to the chain
            // We need the source chain header as part of the validation package.
            // For an already committed entry (when asked to deliver the validation package to
            // a DHT node) we should have gotten one from chain_header() above.
            // But when we commit an entry, there is no header for it in the chain yet.
            // That is why we have to create a pre-flight header here.
            // If there is another zome function call that also calls commit before this commit
            // is done, we might create two pre-flight chain headers linking to the same
            // previous header. Since these pre-flight headers are not written to the chain
            // and just used for the validation, I don't see why it would be a problem.
            // If it was a problem, we would have to make sure that the whole commit process
            // (including validtion) is atomic.
            let state = State::new(context.clone());
            agent::state::create_new_chain_header(
                &entry,
                &context.state()?.agent(),
                &StateWrapper::from(state),
                &None,
                provenances,
            )?
        }
        Some(entry_header) => entry_header,
    };

    get_validation_package_definition(&entry, context.clone())
        .and_then(|callback_result| match callback_result {
            CallbackResult::Fail(error_string) => Err(HolochainError::ErrorGeneric(error_string)),
            CallbackResult::ValidationPackageDefinition(def) => Ok(def),
            CallbackResult::NotImplemented(reason) => Err(HolochainError::ErrorGeneric(format!(
                "ValidationPackage callback not implemented for {:?} ({})",
                entry.entry_type(),
                reason
            ))),
            _ => unreachable!(),
        })
        .and_then(|package_definition| {
            Ok(match package_definition {
                Entry => ValidationPackage::only_header(entry_header),
                ChainEntries => {
                    let mut package = ValidationPackage::only_header(entry_header);
                    package.source_chain_entries = Some(public_chain_entries_from_headers(
                        &context,
                        &all_chain_headers_before_header(&context, &package.chain_header),
                    ));
                    package
                }
                ChainHeaders => {
                    let mut package = ValidationPackage::only_header(entry_header);
                    package.source_chain_headers = Some(all_chain_headers_before_header(
                        &context,
                        &package.chain_header,
                    ));
                    package
                }
                ChainFull => {
                    let mut package = ValidationPackage::only_header(entry_header);
                    let headers = all_chain_headers_before_header(&context, &package.chain_header);
                    package.source_chain_entries =
                        Some(public_chain_entries_from_headers(&context, &headers));
                    package.source_chain_headers = Some(headers);
                    package
                }
                Custom(string) => {
                    let mut package = ValidationPackage::only_header(entry_header);
                    package.custom = Some(string);
                    package
                }
            })
        })
}

// given a slice of headers return the entries for those marked public
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn public_chain_entries_from_headers(
    context: &Arc<Context>,
    headers: &[ChainHeader],
) -> Vec<Entry> {
    headers
        .iter()
        .filter(|ref chain_header| chain_header.entry_type().can_publish(context))
        .map(|chain_header| {
            context
                .state()
                .expect("No state in public_chain_entries_from_headers")
                .agent()
                .chain_store()
                .get(chain_header.entry_address())
                .expect("Could not read entry from CAS")
                .expect("Entry does not exist")
        })
        .collect::<Vec<_>>()
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn all_chain_headers_before_header(
    context: &Arc<Context>,
    header: &ChainHeader,
) -> Vec<ChainHeader> {
    let chain = context
        .state()
        .expect("No state in all_chain_headers_before_header")
        .agent()
        .chain_store();
    chain.iter(&Some(header.clone())).skip(1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nucleus::actions::tests::*;

    use holochain_core_types::{time::Iso8601, validation::ValidationPackage};
    use holochain_persistence_api::cas::content::{Address, AddressableContent};

    #[test]
    fn test_building_validation_package_entry() {
        let (_instance, context) = instance(None);

        // adding other entries to not have special case of empty chain
        commit(test_entry_package_chain_entries(), &context);
        commit(test_entry_package_chain_full(), &context);

        // commit entry to build validation package for
        let chain_header = commit(test_entry_package_entry(), &context);

        let maybe_validation_package =
            build_validation_package(&test_entry_package_entry(), context.clone(), &vec![]);
        println!("{:?}", maybe_validation_package);
        assert!(maybe_validation_package.is_ok());

        let expected = ValidationPackage {
            chain_header,
            source_chain_entries: None,
            source_chain_headers: None,
            custom: None,
        };

        assert_eq!(maybe_validation_package.unwrap(), expected);
    }

    #[test]
    fn test_building_validation_package_chain_entries() {
        let (_instance, context) = instance(None);

        // adding other entries to not have special case of empty chain
        commit(test_entry_package_chain_entries(), &context);
        commit(test_entry_package_chain_full(), &context);

        // commit entry to build validation package for
        let chain_header = commit(test_entry_package_chain_entries(), &context);

        let maybe_validation_package = build_validation_package(
            &test_entry_package_chain_entries(),
            context.clone(),
            &vec![],
        );
        println!("{:?}", maybe_validation_package);
        assert!(maybe_validation_package.is_ok());

        let expected = ValidationPackage {
            chain_header: chain_header.clone(),
            source_chain_entries: Some(public_chain_entries_from_headers(
                &context,
                &all_chain_headers_before_header(&context, &chain_header),
            )),
            source_chain_headers: None,
            custom: None,
        };

        assert_eq!(maybe_validation_package.unwrap(), expected);
    }

    #[test]
    fn test_building_validation_package_chain_headers() {
        let (_instance, context) = instance(None);

        // adding other entries to not have special case of empty chain
        commit(test_entry_package_chain_entries(), &context);
        commit(test_entry_package_chain_full(), &context);

        // commit entry to build validation package for
        let chain_header = commit(test_entry_package_chain_headers(), &context);

        let maybe_validation_package = build_validation_package(
            &test_entry_package_chain_headers(),
            context.clone(),
            &vec![],
        );
        assert!(maybe_validation_package.is_ok());

        let expected = ValidationPackage {
            chain_header: chain_header.clone(),
            source_chain_entries: None,
            source_chain_headers: Some(all_chain_headers_before_header(&context, &chain_header)),
            custom: None,
        };

        assert_eq!(maybe_validation_package.unwrap(), expected);
    }

    #[test]
    fn test_building_validation_package_chain_full() {
        let (_instance, context) = instance(None);

        // adding other entries to not have special case of empty chain
        commit(test_entry_package_chain_entries(), &context);
        commit(test_entry_package_entry(), &context);

        // commit entry to build validation package for
        let chain_header = commit(test_entry_package_chain_full(), &context);

        let maybe_validation_package =
            build_validation_package(&test_entry_package_chain_full(), context.clone(), &vec![]);
        assert!(maybe_validation_package.is_ok());

        let headers = all_chain_headers_before_header(&context, &chain_header);

        let expected = ValidationPackage {
            chain_header,
            source_chain_entries: Some(public_chain_entries_from_headers(&context, &headers)),
            source_chain_headers: Some(headers),
            custom: None,
        };

        assert_eq!(maybe_validation_package.unwrap(), expected);
    }

    // test can make validation package with empty chain
    #[test]
    fn test_all_chain_headers_before_header_empty_chain() {
        let (_instance, context) = instance(None);
        let top_header = context
            .state()
            .unwrap()
            .agent()
            .top_chain_header()
            .expect("There must be a top chain header");
        let headers = all_chain_headers_before_header(&context, &top_header);
        assert_eq!(headers.len(), 1) // includes the DNA entry only (no agent entry)
    }

    #[test]
    fn test_all_chain_headers_before_header_entry_local_commit_validation() {
        let (_instance, context) = instance(None);

        let top_header = context
            .state()
            .unwrap()
            .agent()
            .top_chain_header()
            .expect("There must be a top chain header");
        // new entry header is created so it points to previous top header but not added to the local chain
        let new_entry_header = ChainHeader::new(
            &EntryType::from("test-new-entry"),
            &Address::from("Qmtestaddress"),
            &Vec::new(),
            &Some(top_header.address()),
            &None,
            &None,
            &Iso8601::new(0, 0),
        );

        let headers = all_chain_headers_before_header(&context, &new_entry_header);
        // entry should not appear in the validating chain
        assert_eq!(headers.contains(&new_entry_header), false);
        assert_eq!(headers.len(), 2) // includes the DNA and agent entries
    }

    #[test]
    fn test_all_chain_headers_before_header_entry_dht_validation() {
        let (_instance, context) = instance(None);
        // entry is added to the local chain
        let chain_header = commit(test_entry_package_chain_full(), &context);
        let headers = all_chain_headers_before_header(&context, &chain_header);
        // entry should not appear in the validating chain
        assert_eq!(headers.contains(&chain_header), false);
        assert_eq!(headers.len(), 2) // includes the DNA and agent entries
    }

    #[test]
    fn test_later_headers_not_included() {
        let (_instance, context) = instance(None);
        // entry is added to the local chain
        let chain_header = commit(test_entry_package_chain_full(), &context);
        let pre_commit_headers = all_chain_headers_before_header(&context, &chain_header);

        // commit come more entries
        commit(test_entry_package_chain_entries(), &context);
        commit(test_entry_package_entry(), &context);

        let post_commit_headers = all_chain_headers_before_header(&context, &chain_header);
        assert_eq!(pre_commit_headers, post_commit_headers)
    }
}
