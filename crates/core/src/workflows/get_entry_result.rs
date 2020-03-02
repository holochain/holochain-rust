use crate::{
    context::Context,
    network::{self, actions::query::QueryMethod, query::NetworkQueryResult},
    nucleus,
};
use holochain_core_types::{chain_header::ChainHeader, time::Timeout};

use holochain_core_types::{
    crud_status::CrudStatus, entry::EntryWithMetaAndHeader, error::HolochainError,
};
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_types::get_entry::{
    GetEntryArgs, GetEntryResult, StatusRequestKind,
};
use std::sync::Arc;
use crate::workflows::WorkflowResult;

/// Get Entry workflow
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn get_entry_with_meta_workflow(
    context: Arc<Context>,
    address: &Address,
    timeout: &Timeout,
) -> WorkflowResult<Option<EntryWithMetaAndHeader>> {
    // 1. Try to get the entry locally (i.e. local DHT shard)
    let maybe_entry_with_meta =
        nucleus::actions::get_entry::get_entry_with_meta(Arc::clone(&context), address.clone())?;
    // 2. No result, so try on the network
    let method = QueryMethod::Entry(address.clone());
    if let None = maybe_entry_with_meta {
        let response =
            network::actions::query::query(context.clone(), method.clone(), timeout.clone())
                .await?;
        match response {
            NetworkQueryResult::Entry(maybe_entry) => Ok(maybe_entry),
            _ => Err(HolochainError::ErrorGeneric(
                "Wrong respond type for Entry".to_string(),
            )),
        }
    } else {
        // 3. If we've found the entry locally we also need to get the header from the local state:
        let entry = maybe_entry_with_meta
            .ok_or_else(|| HolochainError::ErrorGeneric("Could not get entry".to_string()))?;
        match context
            .state()
            .ok_or_else(|| HolochainError::ErrorGeneric("Could not get state".to_string()))?
            .get_headers(address.clone())
        {
            Ok(headers) => Ok(Some(EntryWithMetaAndHeader {
                entry_with_meta: entry.clone(),
                headers,
            })),
            Err(_) => {
                let response = network::actions::query::query(
                    context.clone(),
                    method.clone(),
                    timeout.clone(),
                )
                .await?;
                match response {
                    NetworkQueryResult::Entry(maybe_entry) => Ok(maybe_entry),
                    _ => Err(HolochainError::ErrorGeneric(
                        "Wrong respond type for Entry".to_string(),
                    )),
                }
            }
        }
    }
}

/// Get GetEntryResult workflow
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn get_entry_result_workflow(
    context: Arc<Context>,
    args: &GetEntryArgs,
) -> Result<GetEntryResult, HolochainError> {
    // Setup
    let mut entry_result = GetEntryResult::new(args.options.status_request.clone(), None);
    let mut maybe_address = Some(args.address.clone());

    // Accumulate entry history in a loop unless only request initial.
    while maybe_address.is_some() {
        let address = maybe_address.unwrap();
        maybe_address = None;
        // Try to get entry
        let maybe_entry_with_meta_and_headers =
            get_entry_with_meta_workflow(Arc::clone(&context), &address, &args.options.timeout).await?;

        // Entry found
        if let Some(entry_with_meta_and_headers) = maybe_entry_with_meta_and_headers {
            // Erase history if request is for latest
            if args.options.status_request == StatusRequestKind::Latest
                && entry_with_meta_and_headers.entry_with_meta.crud_status == CrudStatus::Deleted
            {
                entry_result.clear();
                break;
            }

            // Add entry
            let headers: Vec<ChainHeader> = if args.options.headers {
                entry_with_meta_and_headers.headers
            } else {
                Vec::new()
            };
            entry_result.push(&entry_with_meta_and_headers.entry_with_meta, headers);

            if args.options.status_request == StatusRequestKind::Initial {
                break;
            }

            // Follow crud-link if possible
            if entry_with_meta_and_headers
                .entry_with_meta
                .maybe_link_update_delete
                .is_some()
                && entry_with_meta_and_headers.entry_with_meta.crud_status != CrudStatus::Deleted
                && args.options.status_request != StatusRequestKind::Initial
            {
                maybe_address = Some(
                    entry_with_meta_and_headers
                        .entry_with_meta
                        .maybe_link_update_delete
                        .unwrap(),
                );
            }
        }
    }

    Ok(entry_result)
}

//#[cfg(test)]
//pub mod tests {
//    use crate::instance::tests::test_context_with_state;
//    use futures::executor::block_on;
//    use holochain_core_types::{
//        cas::content::AddressableContent,
//        crud_status::{create_crud_status_eav, CrudStatus},
//        entry::test_entry,
//    };
//    use holochain_wasm_types::get_entry::*;
//
//    #[test]
//    fn can_get_entry_result_workflow() {
//        let entry = test_entry();
//        let context = test_context_with_state();
//        let args = GetEntryArgs {
//            address: entry.address(),
//            options: GetEntryOptions {
//                status_request: StatusRequestKind::Latest,
//            },
//        };
//        let maybe_entry_history = block_on(super::get_entry_result_workflow(&context, &args));
////        assert_eq!(0, maybe_entry_history.unwrap().entries.len());
////        context.state().unwrap().dht().add(&entry).unwrap();
////        let status_eav = create_crud_status_eav(&entry.address(), CrudStatus::Live);
////        let meta_storage = &context.state().unwrap().dht().meta_storage().clone();
////        (*meta_storage.write().unwrap())
////            .add_eavi(&status_eav)
////            .unwrap();
////        let maybe_entry_history = block_on(super::get_entry_result_workflow(&context, &args));
////        let entry_history = maybe_entry_history.unwrap();
////        assert_eq!(&entry, entry_history.entries.iter().next().unwrap());
//    }
//}
