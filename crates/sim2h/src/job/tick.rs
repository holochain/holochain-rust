use crate::*;

/// a job that prints a debug trace line once per second
/// indicating that the thread pool is still processing jobs
pub(crate) struct Tick;

impl Tick {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Job for Arc<Mutex<Tick>> {
    fn run(&mut self) -> JobResult {
        trace!("sim2h job loop - 1 second tick");
        JobResult::default().wait_ms(1000)
    }
}
