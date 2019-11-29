use std::fmt::Debug;

pub fn lax_send<T: Debug>(tx: crossbeam_channel::Sender<T>, val: T, _failure_reason: &str) -> bool {
    match tx.send(val) {
        Ok(()) => true,
        Err(_) => {
            // println!("[lax_send]\n{}\n{:?}\n", _failure_reason, val);
            false
        }
    }
}

pub fn lax_send_wrapped<T: Send + Debug>(
    tx: holochain_tracing::SpanSender<T>,
    val: T,
    _failure_reason: &str,
) -> bool {
    match tx.send_wrapped(val) {
        Ok(()) => true,
        Err(_) => {
            // println!("[lax_send]\n{}\n{:?}\n", _failure_reason, val);
            false
        }
    }
}
