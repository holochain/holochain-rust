use action::{Action, ActionWrapper};
use agent::state::ActionResponse;
use holochain_core_types::get_links_args::GetLinksArgs;
use nucleus::ribosome::runtime::Runtime;
use serde_json;
use std::sync::mpsc::channel;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// ZomeApiFunction::GetLinks function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: GetLinksArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_get_links(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime.load_utf8_from_args(&args);
    let res_entry: Result<GetLinksArgs, _> = serde_json::from_str(&args_str);
    // Exit on error
    if res_entry.is_err() {
        return ribosome_error_code!(ArgumentDeserializationFailed);
    }
    let input = res_entry.unwrap();
    // Create GetLinks Action
    let action_wrapper = ActionWrapper::new(Action::GetLinks(input));
    // Send Action and block for result
    let (sender, receiver) = channel();
    // TODO #338 - lookup in DHT instead when it will be available (for caching). Will also be redesigned with Futures.
    ::instance::dispatch_action_with_observer(
        &runtime.context.action_channel,
        &runtime.context.observer_channel,
        action_wrapper.clone(),
        move |state: &::state::State| {
            // TODO #338 - lookup in DHT instead when it will be available. Will also be redesigned with Futures.
            let mut actions_copy = state.agent().actions();
            match actions_copy.remove(&action_wrapper) {
                Some(v) => {
                    // @TODO never panic in wasm
                    // @see https://github.com/holochain/holochain-rust/issues/159
                    sender
                        .send(v)
                        // the channel stays connected until the first message has been sent
                        // if this fails that means that it was called after having returned done=true
                        .expect("observer called after done");
                    true
                }
                None => false,
            }
        },
    );
    // TODO #97 - Return error if timeout or something failed
    // return Err(_);
    let action_result = receiver.recv().expect("observer dropped before done");
    if let ActionResponse::GetLinks(maybe_links) = action_result {
        if let Ok(link_list) = maybe_links {
            return runtime.store_utf8(&json!(link_list).as_str().expect("should jsonify"));
        }
    }
    // Fail
    ribosome_error_code!(ReceivedWrongActionResult)
}
