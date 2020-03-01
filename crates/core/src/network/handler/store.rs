use crate::{
    context::Context,
    dht::{
        actions::queue_holding_workflow::dispatch_queue_holding_workflow,
        pending_validations::PendingValidationStruct,
    },

};
use holochain_core_types::network::entry_aspect::EntryAspect;
use holochain_json_api::json::JsonString;
use lib3h_protocol::data_types::StoreEntryAspectData;
use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
};

/// The network requests us to store (i.e. hold) the given entry aspect data.
#[autotrace]
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn handle_store(dht_data: StoreEntryAspectData, context: Arc<Context>) {
    let aspect_json =
        JsonString::from_json(std::str::from_utf8(&*dht_data.entry_aspect.aspect).unwrap());
    let maybe_aspect: Result<EntryAspect, _> = aspect_json.clone().try_into();
    if let Ok(aspect) = maybe_aspect {
        if context
            .state()
            .unwrap()
            .dht()
            .get_holding_map()
            .contains(&aspect)
        {
            log_error!(
                context,
                "handle_store: Aspect already being held: {:?}",
                aspect
            );
            return;
        }
        match PendingValidationStruct::try_from(aspect) {
            Err(e) => log_error!(
                context,
                "net/handle: handle_store: received bad aspect: {:?}",
                e,
            ),
            Ok(pending) => {
                log_debug!(
                    context,
                    "net/handle: handle_store: Adding {} to holding queue...",
                    pending.workflow,
                );
                dispatch_queue_holding_workflow(Arc::clone(&context), Arc::new(pending), None);
            }
        }
    } else {
        log_error!(
            context,
            "net/handle_store: Unable to parse entry aspect: {}",
            aspect_json
        )
    }
}
