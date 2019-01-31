use crate::{context::Context, network, nucleus};
use holochain_core_types::{chain_header::ChainHeader, time::Timeout};

use holochain_core_types::{
    cas::content::Address, crud_status::CrudStatus, entry::EntryWithMeta, error::HolochainError,
};
use holochain_wasm_utils::api_serialization::get_entry::{
    GetEntryArgs, GetEntryResult, StatusRequestKind,
};
use std::sync::Arc;

/// Get Entry workflow
pub async fn get_entry_with_meta_workflow<'a>(
    context: &'a Arc<Context>,
    address: &'a Address,
    timeout: &'a Timeout,
) -> Result<Option<EntryWithMeta>, HolochainError> {
    // 1. Try to get the entry locally (i.e. local DHT shard)
    let maybe_entry_with_meta =
        nucleus::actions::get_entry::get_entry_with_meta(context, address.clone())?;
    if maybe_entry_with_meta.is_some() {
        return Ok(maybe_entry_with_meta);
    }
    // 2. No result, so try on the network
    await!(network::actions::get_entry::get_entry(
        context.clone(),
        address.clone(),
        timeout.clone(),
    ))
}

/// Get GetEntryResult workflow
pub async fn get_entry_result_workflow<'a>(
    context: &'a Arc<Context>,
    args: &'a GetEntryArgs,
) -> Result<GetEntryResult, HolochainError> {
    // Setup
    let mut entry_result = GetEntryResult::new(args.options.status_request.clone(), None);
    let mut maybe_address = Some(args.address.clone());

    // Accumulate entry history in a loop unless only request initial.
    while maybe_address.is_some() {
        let address = maybe_address.unwrap();
        maybe_address = None;
        // Try to get entry
        let maybe_entry_with_meta = await!(get_entry_with_meta_workflow(
            context,
            &address,
            &args.options.timeout
        ))?;
        // Entry found
        if let Some(entry_with_meta) = maybe_entry_with_meta {
            // Erase history if request is for latest
            if args.options.status_request == StatusRequestKind::Latest {
                if entry_with_meta.crud_status == CrudStatus::Deleted {
                    entry_result.clear();
                    break;
                }
            }

            // Add entry
            let headers: Vec<ChainHeader> = if args.options.headers {
                let state = context.state().expect("state uninitialized! :)");
                let mut headers: Vec<_> = state
                    .agent()
                    .get_header_for_entry(&entry_with_meta.entry)
                    .into_iter()
                    .collect();
                let mut dht_headers = state.dht().get_headers(address)?;
                headers.append(&mut dht_headers);
                headers
            } else {
                Vec::new()
            };
            entry_result.push(&entry_with_meta, headers);

            if args.options.status_request == StatusRequestKind::Initial {
                break;
            }

            // Follow crud-link if possible
            if entry_with_meta.maybe_crud_link.is_some()
                && entry_with_meta.crud_status != CrudStatus::Deleted
                && args.options.status_request != StatusRequestKind::Initial
            {
                maybe_address = Some(entry_with_meta.maybe_crud_link.unwrap());
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
//    use holochain_wasm_utils::api_serialization::get_entry::*;
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
////        let content_storage = &context.state().unwrap().dht().content_storage().clone();
////        (*content_storage.write().unwrap()).add(&entry).unwrap();
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
