use crate::action::ActionWrapper;
use holochain_core_types::{error::HolochainError, json::JsonString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    sync::mpsc::{channel, sync_channel, Receiver, SyncSender},
    thread,
};

#[derive(Clone, Debug, DefaultJson)]
pub enum Signal {
    Internal(ActionWrapper),
    User(JsonString),
    // NB: this is part of a temporary hack that will be removed
    // as soon as a browser light client is implemented!
    Holo(JsonString),
}

impl Serialize for Signal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Signal::Internal(_) => serializer.serialize_newtype_variant(
                "Signal",
                0,
                "Internal",
                "(Internal signal serialization not yet implemented)",
            ),
            Signal::User(msg) => {
                serializer.serialize_newtype_variant("Signal", 1, "User", &msg.to_string())
            }
            Signal::Holo(msg) => {
                serializer.serialize_newtype_variant("Signal", 2, "Holo", &msg.to_string())
            }
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

pub type SignalSender = SyncSender<Signal>;
pub type SignalReceiver = Receiver<Signal>;

pub fn signal_channel() -> (SignalSender, SignalReceiver) {
    sync_channel(1000)
}

/// Pass on messages from multiple receivers into a single receiver
/// A potentially useful utility, but currently unused.
pub fn _combine_receivers<T>(rxs: Vec<Receiver<T>>) -> Receiver<T>
where
    T: 'static + Send,
{
    let (master_tx, master_rx) = channel::<T>();
    for rx in rxs {
        let tx = master_tx.clone();
        thread::spawn(move || {
            while let Ok(item) = rx.recv() {
                tx.send(item).unwrap_or(());
            }
        });
    }
    master_rx
}
