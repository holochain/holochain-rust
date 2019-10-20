pub fn setup() {
    let _ = env_logger::builder().is_test(true).try_init();
}

pub fn tracer(log_context: &LogContext, msg: &str) {
    setup();
    trace!("{}: {}", log_context, msg);
}

pub type LogContext = &'static str;
