use std::fmt::Debug;

use std::sync::mpsc::{Sender, SyncSender};

pub fn lax_send<T: Clone + Debug>(tx: Sender<T>, val: T, failure_reason: &str) -> bool {
    match tx.send(val.clone()) {
        Ok(()) => true,
        Err(_) => {
            println!("[MEMLEAK] {} {:?}", failure_reason, val);
            false
        }
    }
}

pub fn lax_send_sync<T: Clone + Debug>(tx: SyncSender<T>, val: T, failure_reason: &str) -> bool {
    match tx.send(val.clone()) {
        Ok(()) => true,
        Err(_) => {
            println!("[MEMLEAK] {} {:?}", failure_reason, val);
            false
        }
    }
}
