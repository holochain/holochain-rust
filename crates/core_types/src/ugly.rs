use log::error;
use std::fmt::Debug;

pub fn lax_send<T: Clone + Debug>(
    tx: crossbeam_channel::Sender<T>,
    val: T,
    failure_reason: &str,
) -> bool {
    match tx.send(val.clone()) {
        Ok(()) => true,
        Err(_) => {
            error!("[lax_send]\n{}\n{:?}\n", failure_reason, val);
            false
        }
    }
}

pub fn lax_send_sync<T: Clone + Debug>(
    tx: crossbeam_channel::Sender<T>,
    val: T,
    failure_reason: &str,
) -> bool {
    match tx.send(val.clone()) {
        Ok(()) => true,
        Err(_) => {
            error!("[lax_send_sync]\n{}\n{:?}\n", failure_reason, val);
            false
        }
    }
}
