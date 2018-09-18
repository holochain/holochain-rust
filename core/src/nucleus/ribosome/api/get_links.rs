use action::{Action, ActionWrapper};
use agent::state::ActionResponse;
use hash::HashString;
use nucleus::ribosome::api::{HcApiReturnCode, Runtime};
use serde_json;
use std::sync::mpsc::channel;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct GetLinksArgs {
    pub entry_hash: HashString,
    pub tag: String,
}

impl GetLinksArgs {
    pub fn to_attribute_name(&self) -> String {
        format!("link:{}:{}", &self.entry_hash, &self.tag)
    }
}

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
        // Return Error code in i32 format
        return Ok(Some(RuntimeValue::I32(HcApiReturnCode::ArgumentDeserializationFailed as i32)));
    }
    let input = res_entry.unwrap();
    // Create GetLinks Action
    let action_wrapper = ActionWrapper::new(Action::GetLinks(input));
    // Send Action and block for result
    let (sender, receiver) = channel();
    ::instance::dispatch_action_with_observer(
        &runtime.action_channel,
        &runtime.observer_channel,
        action_wrapper.clone(),
        move |state: &::state::State| {
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
    Ok(Some(RuntimeValue::I32(
        HcApiReturnCode::ReceivedWrongActionResult as i32,
    )))
}
