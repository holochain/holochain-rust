use super::Dispatch;
use error::ZomeApiResult;
use holochain_json_api::json::JsonString;
use holochain_wasm_utils::api_serialization::emit_signal::EmitSignalArgs;

/// Emits a signal that listeners can receive.
/// (Status: MVP)
///
/// Part of Holochain's implementation of the Signal/Slot pattern.
/// DNAs can define signals that various other modules can listen to / connect a slot with.
///
/// The main use-case of this is to provide a way to have callbacks of the UI get called
/// so that UIs don't have to rely on polling in order to stay updated.
///
/// Currently, signals that are emitted with this function will be send over any websocket
/// UI interface that includes an instance of this DNA.
/// (In the future there might ways to connect signals of one instance with slots (i.e.
/// zome functions) of other instances as a more abstract form of a bridge.)
///
/// A signal has a name and a set of arguments that are send with it.
/// Currently (=MVP) both can be chosen arbitrarily and it is up to the hApp developer to make
/// the assumptions in the UI match what is provided in the call to emit_signal() in the DNA.
/// As we continue to implement the full ADR on signals (https://github.com/holochain/holochain-rust/blob/develop/doc/architecture/decisions/0013-signals-listeners-model-and-api.md),
/// signals will have to be defined in the DNA so the conductor can check signal signatures
/// when connecting them with slots.
/// # Examples
/// ```rust
/// # #[macro_use]
/// # extern crate hdk;
/// # use hdk::error::ZomeApiResult;
/// # use std::time::Duration;
/// # use hdk::holochain_json_api::json::JsonString;
/// # use hdk::holochain_core_types::error::RibosomeEncodingBits;
/// # use hdk::holochain_core_types::error::RibosomeEncodedValue;
/// # #[no_mangle]
/// # pub fn hc_emit_signal(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
///
/// # fn main() {
/// pub fn handle_receive_chat_message(message: String) -> ZomeApiResult<()> {
///     // ...
///     hdk::emit_signal("message_received", JsonString::from_json(&format!(
///         "{{message: {}}}", message
///     )));
///     // ...
///     Ok(())
/// }
///
/// # }
/// ```
pub fn emit_signal<S: Into<String>, J: Into<JsonString>>(
    name: S,
    arguments: J,
) -> ZomeApiResult<()> {
    let _: ZomeApiResult<()> = Dispatch::EmitSignal.with_input(EmitSignalArgs {
        name: name.into(),
        arguments: arguments.into(),
    });
    // internally returns RibosomeEncodedValue::Success which is a zero length allocation
    // return Ok(()) unconditionally instead of the "error" from success
    Ok(())
}
