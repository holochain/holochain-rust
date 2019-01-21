use crate::action::ActionWrapper;
use std::{
    sync::mpsc::{channel, sync_channel, Receiver, SyncSender},
    thread,
};

#[derive(Debug)]
pub enum Signal {
    Internal(ActionWrapper),
    User,
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
