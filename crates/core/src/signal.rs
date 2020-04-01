use crate::{action::ActionWrapper, consistency::ConsistencySignal, CHANNEL_SIZE};
use crossbeam_channel::{bounded, Receiver, Sender};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_wasm_utils::api_serialization::emit_signal::EmitSignalArgs;
use serde::{Deserialize, Deserializer};
use snowflake::ProcessUniqueId;
use std::thread;

#[derive(Clone, Debug, Serialize, DefaultJson)]
#[serde(tag = "signal_type")]
#[allow(clippy::large_enum_variant)]
pub enum Signal {
    Trace(ActionWrapper),
    Consistency(ConsistencySignal<String>),
    User(UserSignal),
}

#[derive(Clone, Debug, Serialize, Deserialize, DefaultJson, PartialEq)]
pub struct UserSignal {
    pub name: String,
    pub arguments: JsonString,
}

impl From<EmitSignalArgs> for UserSignal {
    fn from(args: EmitSignalArgs) -> UserSignal {
        UserSignal {
            name: args.name,
            arguments: args.arguments,
        }
    }
}

impl<'de> Deserialize<'de> for Signal {
    fn deserialize<D>(_deserializer: D) -> Result<Signal, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}

pub type SignalSender = Sender<Signal>;
pub type SignalReceiver = Receiver<Signal>;

pub fn signal_channel() -> (SignalSender, SignalReceiver) {
    bounded(CHANNEL_SIZE)
}

/// Pass on messages from multiple receivers into a single receiver
/// A potentially useful utility, but currently unused.
pub fn _combine_receivers<T>(rxs: Vec<Receiver<T>>) -> Receiver<T>
where
    T: 'static + Send,
{
    let (master_tx, master_rx) = bounded::<T>(CHANNEL_SIZE);
    for rx in rxs {
        let tx = master_tx.clone();
        let _ = thread::Builder::new()
            .name(format!(
                "combine_receivers/{}",
                ProcessUniqueId::new().to_string()
            ))
            .spawn(move || {
                while let Ok(item) = rx.recv() {
                    tx.send(item).unwrap_or(());
                }
            });
    }
    master_rx
}
