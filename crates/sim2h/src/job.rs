/// typedef for code clarity
pub(crate) struct JobResult {
    pub(crate) cont: bool,
    pub(crate) wait_ms: u64,
}

impl Default for JobResult {
    fn default() -> Self {
        Self {
            cont: true,
            wait_ms: 0,
        }
    }
}

#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
impl JobResult {
    pub(crate) fn done() -> Self {
        Self {
            cont: false,
            wait_ms: 0,
        }
    }

    pub(crate) fn wait_ms(mut self, wait_ms: u64) -> Self {
        self.wait_ms = wait_ms;
        self
    }
}

/// an item that can be executed via thread pool
pub(crate) trait Job: 'static + Send {
    /// execute one iteration of this job - try to be as short-lived as possible
    fn run(&mut self) -> JobResult;
}

mod connection;
pub(crate) use connection::*;
