use crate::{
    context::Context,
    network::entry_with_header::EntryWithHeader,
    workflows::{hold_entry::hold_entry_workflow, hold_link::hold_link_workflow},
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    entry::entry_type::EntryType,
};
use std::{sync::Arc, thread};

pub type PendingValidation = Arc<(EntryWithHeader, Vec<Address>)>;

fn retry_validation(pending: PendingValidation, context: Arc<Context>) {
    thread::spawn(move || match pending.0.entry.entry_type() {
        EntryType::LinkAdd | EntryType::LinkRemove => {
            let _ = context.block_on(hold_link_workflow(&pending.0, &context));
        }
        _ => {
            let _ = context.block_on(hold_entry_workflow(&pending.0, context.clone()));
        }
    });
}

pub fn run_pending_validations(context: Arc<Context>) {
    context
        .state()
        .unwrap()
        .nucleus()
        .pending_validations
        .iter()
        .for_each(|(_, pending)| {
            context.log(dbg!(format!(
                "debug/scheduled_jobs/run_pending_validations: found pending validation for {}: {}",
                pending.0.entry.entry_type(),
                pending.0.entry.address()
            )));
            retry_validation(pending.clone(), context.clone());
        });
}
