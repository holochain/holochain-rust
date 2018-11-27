use crate::action::ActionWrapper;
use crate::network::state::NetworkState;
use crate::context::Context;
use std::sync::Arc;
use holochain_core_types::error::HolochainError;
use holochain_core_types::entry::Entry;
use crate::action::Action;

fn entry_to_cas(entry: &Entry, context: &Arc<Context>,) -> Result<(), HolochainError>{
    Ok(context.file_storage.read()?.add(entry)?)
}

pub fn reduce_receive(
    context: Arc<Context>,
    _state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {

    let action = action_wrapper.action();
    let address = unwrap_to!(action => Action::Receive);

    let result = entry_to_cas(address, &context);
    if result.is_err() {
        return;
    };

}
