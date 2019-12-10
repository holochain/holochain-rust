use crate::*;

/// a job that prints a debug trace line once per second
/// indicating that the thread pool is still processing jobs
pub(crate) struct Tick {
    next_tick: std::time::Instant,
}

impl Tick {
    pub(crate) fn new() -> Self {
        Self {
            next_tick: std::time::Instant::now()
                .checked_add(std::time::Duration::from_secs(1))
                .expect("failed to add 1 second"),
        }
    }
}

impl Job for Arc<Mutex<Tick>> {
    fn run(&mut self) -> JobContinue {
        let now = std::time::Instant::now();
        let mut me = self.f_lock();
        if now >= me.next_tick {
            me.next_tick = now
                .checked_add(std::time::Duration::from_secs(1))
                .expect("failed to add 1 second");
            trace!("sim2h job loop - 1 second tick");
        }
        true
    }
}
