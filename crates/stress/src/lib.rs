use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub struct StressJobTickResult {
    pub should_continue: bool,
}

pub trait StressJob: 'static + Send + Sync {
    fn tick(&mut self) -> StressJobTickResult;
}

pub type JobFactory<J> = Box<dyn FnMut() -> J + 'static + Send + Sync>;

#[derive(Debug, Clone)]
pub struct StressStats {
    master_tick_count: u64,
    job_tick_count: u64,
}

pub trait StressSuite: 'static + Send + Sync {
    fn start(&mut self);
    fn stop(&mut self, stats: StressStats);
}

pub struct StressRunConfig<S: StressSuite, J: StressJob> {
    pub thread_pool_size: usize,
    pub job_count: usize,
    pub run_time: std::time::Duration,
    pub suite: S,
    pub job_factory: JobFactory<J>,
}

struct StressJobInfo<J: StressJob> {
    job: J,
}

struct StressRunner<S: StressSuite, J: StressJob> {
    config: StressRunConfig<S, J>,
    run_until: std::time::Instant,
    thread_pool: Vec<std::thread::JoinHandle<()>>,
    should_continue: Arc<Mutex<bool>>,
    job_count: Arc<Mutex<usize>>,
    job_queue: Arc<Mutex<VecDeque<StressJobInfo<J>>>>,
    stats: Arc<Mutex<StressStats>>,
}

impl<S: StressSuite, J: StressJob> StressRunner<S, J> {
    pub fn new(config: StressRunConfig<S, J>) -> Self {
        let run_until = std::time::Instant::now()
            .checked_add(config.run_time)
            .unwrap();
        let mut runner = StressRunner {
            config,
            run_until,
            thread_pool: Vec::new(),
            should_continue: Arc::new(Mutex::new(true)),
            job_count: Arc::new(Mutex::new(0)),
            job_queue: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(Mutex::new(StressStats {
                master_tick_count: 0,
                job_tick_count: 0,
            })),
        };
        for _ in 0..runner.config.thread_pool_size {
            runner.priv_create_thread();
        }
        runner.config.suite.start();
        runner
    }

    pub fn tick(&mut self) -> bool {
        if std::time::Instant::now() > self.run_until {
            *self.should_continue.lock().unwrap() = false;
            return false;
        }
        {
            let mut cur_job_count = self.job_count.lock().unwrap();
            while *cur_job_count < self.config.job_count {
                (*self.job_queue.lock().unwrap()).push_front(StressJobInfo {
                    job: (self.config.job_factory)(),
                });
                *cur_job_count += 1
            }
        }
        (*self.stats.lock().unwrap()).master_tick_count += 1;
        true
    }

    pub fn cleanup(mut self) {
        *self.should_continue.lock().unwrap() = false;
        for t in self.thread_pool.drain(..) {
            t.join().unwrap();
        }
        let stats = Arc::try_unwrap(self.stats).unwrap().into_inner().unwrap();
        self.config.suite.stop(stats);
    }

    // -- private -- //

    fn priv_create_thread(&mut self) {
        let should_continue = self.should_continue.clone();
        let job_count = self.job_count.clone();
        let job_queue = self.job_queue.clone();
        let stats = self.stats.clone();
        self.thread_pool.push(std::thread::spawn(move || {
            loop {
                if !*should_continue.lock().unwrap() {
                    return;
                }
                let mut job = match (*job_queue.lock().unwrap()).pop_front() {
                    Some(job) => job,
                    None => continue,
                };
                let result = job.job.tick();
                (*stats.lock().unwrap()).job_tick_count += 1;
                if result.should_continue {
                    (*job_queue.lock().unwrap()).push_back(job);
                } else {
                    *job_count.lock().unwrap() -= 1;
                }
            }
        }));
    }
}

pub fn stress_run<S: StressSuite, J: StressJob>(config: StressRunConfig<S, J>) {
    let mut runner = StressRunner::new(config);
    while runner.tick() {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    runner.cleanup();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_stress() {
        let job_tick_count = Arc::new(Mutex::new(0_u64));

        struct Job {
            job_tick_count: Arc<Mutex<u64>>,
        };
        impl StressJob for Job {
            fn tick(&mut self) -> StressJobTickResult {
                *self.job_tick_count.lock().unwrap() += 1;
                StressJobTickResult {
                    should_continue: true,
                }
            }
        }

        struct Suite;
        impl StressSuite for Suite {
            fn start(&mut self) {
                println!("got start");
            }

            fn stop(&mut self, stats: StressStats) {
                println!("got stop: {:#?}", stats);
            }
        }

        let job_tick_count_clone = job_tick_count.clone();
        stress_run(StressRunConfig {
            thread_pool_size: 10,
            job_count: 100,
            run_time: std::time::Duration::from_millis(200),
            suite: Suite,
            job_factory: Box::new(move || {
                Job {
                    job_tick_count: job_tick_count_clone.clone(),
                }
            }),
        });

        println!("job tick count: {}", *job_tick_count.lock().unwrap());
    }
}
