use crate::{
    context::Context, network::entry_with_header::EntryWithHeader,
    workflows::hold_link::hold_link_workflow,
};
use holochain_core_types::{cas::content::Address, entry::entry_type::EntryType};
use std::{sync::Arc, thread};

pub type PendingValidation = Box<(EntryWithHeader, Vec<Address>)>;

fn retry_validation(pending: PendingValidation, context: Arc<Context>) {
    thread::spawn(move || context.block_on(hold_link_workflow(&pending.0, &context)));
}

pub fn run_pending_validations(context: Arc<Context>) {
    context
        .state()
        .unwrap()
        .nucleus()
        .pending_validations
        .iter()
        .for_each(|(_, boxed)| match boxed.0.entry.entry_type() {
            EntryType::LinkAdd => retry_validation(boxed.clone(), context.clone()),
            EntryType::LinkRemove => retry_validation(boxed.clone(), context.clone()),
            _ => panic!("Pending validations are (currently) comming "),
        });
}
