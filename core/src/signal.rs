use std::{
  sync::{
    mpsc::{channel, Receiver},
  },
  thread,
};

use crate::action::Action;

#[derive(Debug)]
pub enum Signal {
  Internal(Action),
  User
}

/// Pass on messages from multiple receivers into a single receiver
pub fn combine_receivers<T>(rxs: Vec<Receiver<T>>) -> Receiver<T>
where T: 'static + Send {
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
