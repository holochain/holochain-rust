use std::{ops::Range, thread, time::Duration};

pub const INTERFACE_CONNECT_ATTEMPTS_MAX: usize = 30;
pub const INTERFACE_CONNECT_INTERVAL: Duration = Duration::from_secs(1);

[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CONDUCTOR_LIB)]
pub fn try_with_port<T, F: FnOnce() -> T>(port: u16, f: F) -> T {
    let mut attempts = 0;
    while attempts <= INTERFACE_CONNECT_ATTEMPTS_MAX {
        if port_is_available(port) {
            return f();
        }
        warn!(
            "Waiting for port {} to be available, sleeping (attempt #{})",
            port, attempts
        );
        thread::sleep(INTERFACE_CONNECT_INTERVAL);
        attempts += 1;
    }
    f()
}

[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CONDUCTOR_LIB)]
pub fn port_is_available(port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(format!("0.0.0.0:{}", port)).is_ok()
}

[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CONDUCTOR_LIB)]
pub fn get_free_port(range: Range<u16>) -> Option<u16> {
    for i in range {
        if port_is_available(i) {
            return Some(i);
        }
    }
    None
}
