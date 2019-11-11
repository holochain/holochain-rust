extern crate crossbeam_channel;

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

pub struct StressJobMetricLogger {
    pub job_index: usize,
    logs: Vec<StressJobLog>,
}

impl StressJobMetricLogger {
    fn new(job_index: usize) -> Self {
        Self {
            job_index,
            logs: Vec::new(),
        }
    }

    pub fn log(&mut self, name: &str, value: f64) {
        self.logs.push(StressJobLog {
            job_index: self.job_index,
            name: name.to_string(),
            value,
        });
    }
}

pub struct StressJobTickResult {
    pub should_continue: bool,
}

impl Default for StressJobTickResult {
    fn default() -> Self {
        Self {
            should_continue: true,
        }
    }
}

pub trait StressJob: 'static + Send + Sync {
    fn tick(&mut self, logger: &mut StressJobMetricLogger) -> StressJobTickResult;
}

pub type JobFactory<J> = Box<dyn FnMut() -> J + 'static + Send + Sync>;

#[derive(Debug, Clone)]
pub struct StressLogStats {
    pub count: u64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

#[derive(Debug, Clone)]
pub struct StressStats {
    pub master_tick_count: u64,
    pub log_stats: HashMap<String, StressLogStats>,
}

#[derive(Debug, Clone)]
struct StressJobLog {
    pub job_index: usize,
    pub name: String,
    pub value: f64,
}

pub trait StressSuite: 'static {
    fn start(&mut self);
    fn progress(&mut self, stats: &StressStats);
    fn stop(&mut self, stats: StressStats);
    fn tick(&mut self) {}
}

pub struct StressRunConfig<S: StressSuite, J: StressJob> {
    pub thread_pool_size: usize,
    pub job_count: usize,
    pub run_time_ms: u64,
    pub progress_interval_ms: u64,
    pub suite: S,
    pub job_factory: JobFactory<J>,
}

impl<S: StressSuite, J: StressJob> std::fmt::Debug for StressRunConfig<S, J> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StressRunConfig")
            .field("thread_pool_size", &self.thread_pool_size)
            .field("job_count", &self.job_count)
            .field("run_time_ms", &self.run_time_ms)
            .field("progress_interval_ms", &self.progress_interval_ms)
            .finish()
    }
}

struct StressJobInfo<J: StressJob> {
    job_index: usize,
    job: J,
}

struct StressRunner<S: StressSuite, J: StressJob> {
    config: StressRunConfig<S, J>,
    run_until: std::time::Instant,
    next_progress: std::time::Instant,
    thread_pool: Vec<std::thread::JoinHandle<()>>,
    should_continue: Arc<Mutex<bool>>,
    job_count: Arc<Mutex<usize>>,
    job_queue: Arc<Mutex<VecDeque<StressJobInfo<J>>>>,
    job_last_index: usize,
    log_recv: crossbeam_channel::Receiver<StressJobLog>,
    log_send: crossbeam_channel::Sender<StressJobLog>,
    stats: StressStats,
}

impl<S: StressSuite, J: StressJob> StressRunner<S, J> {
    pub fn new(config: StressRunConfig<S, J>) -> Self {
        let (log_send, log_recv) = crossbeam_channel::unbounded();
        let run_until = std::time::Instant::now()
            .checked_add(std::time::Duration::from_millis(config.run_time_ms))
            .unwrap();
        let next_progress = std::time::Instant::now()
            .checked_add(std::time::Duration::from_millis(
                config.progress_interval_ms,
            ))
            .unwrap();
        let mut runner = StressRunner {
            config,
            run_until,
            next_progress,
            thread_pool: Vec::new(),
            should_continue: Arc::new(Mutex::new(true)),
            job_count: Arc::new(Mutex::new(0)),
            job_queue: Arc::new(Mutex::new(VecDeque::new())),
            job_last_index: 0,
            log_recv,
            log_send,
            stats: StressStats {
                master_tick_count: 0,
                log_stats: HashMap::new(),
            },
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
                    job_index: self.job_last_index,
                    job: (self.config.job_factory)(),
                });
                self.job_last_index += 1;
                *cur_job_count += 1
            }
        }
        for _ in 0..1000 {
            match self.log_recv.try_recv() {
                Err(_) => break,
                Ok(log) => {
                    let r =
                        self.stats
                            .log_stats
                            .entry(log.name)
                            .or_insert_with(|| StressLogStats {
                                count: 0,
                                min: std::f64::MAX,
                                max: std::f64::MIN,
                                avg: 0.0,
                            });
                    r.avg = r.avg * r.count as f64;
                    r.avg += log.value;
                    r.count += 1;
                    r.avg /= r.count as f64;
                    if log.value < r.min {
                        r.min = log.value;
                    }
                    if log.value > r.max {
                        r.max = log.value;
                    }
                }
            }
        }
        if std::time::Instant::now() > self.next_progress {
            self.next_progress = std::time::Instant::now()
                .checked_add(std::time::Duration::from_millis(
                    self.config.progress_interval_ms,
                ))
                .unwrap();
            self.config.suite.progress(&self.stats);
        }
        self.config.suite.tick();
        self.stats.master_tick_count += 1;
        true
    }

    pub fn cleanup(mut self) {
        *self.should_continue.lock().unwrap() = false;
        for t in self.thread_pool.drain(..) {
            t.join().unwrap();
        }
        self.config.suite.stop(self.stats);
    }

    // -- private -- //

    fn priv_create_thread(&mut self) {
        let should_continue = self.should_continue.clone();
        let job_count = self.job_count.clone();
        let job_queue = self.job_queue.clone();
        let log_send = self.log_send.clone();
        self.thread_pool.push(std::thread::spawn(move || loop {
            if !*should_continue.lock().unwrap() {
                return;
            }
            let mut job = match (*job_queue.lock().unwrap()).pop_front() {
                Some(job) => job,
                None => continue,
            };
            let mut logger = StressJobMetricLogger::new(job.job_index);
            let start = std::time::Instant::now();
            let result = job.job.tick(&mut logger);
            logger.log("tick_elapsed_ms", start.elapsed().as_millis() as f64);
            for l in logger.logs.drain(..) {
                log_send.send(l).unwrap();
            }
            if result.should_continue {
                (*job_queue.lock().unwrap()).push_back(job);
            } else {
                *job_count.lock().unwrap() -= 1;
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
            fn tick(&mut self, _logger: &mut StressJobMetricLogger) -> StressJobTickResult {
                let tick_count = {
                    let mut lock = self.job_tick_count.lock().unwrap();
                    *lock += 1;
                    *lock
                };
                std::thread::sleep(std::time::Duration::from_millis(tick_count % 4));
                StressJobTickResult::default()
            }
        }

        struct Suite;
        impl StressSuite for Suite {
            fn start(&mut self) {
                println!("got start");
            }

            fn progress(&mut self, stats: &StressStats) {
                println!("got progress: {:#?}", stats);
            }

            fn stop(&mut self, stats: StressStats) {
                println!("got stop: {:#?}", stats);
            }
        }

        let job_tick_count_clone = job_tick_count.clone();
        stress_run(StressRunConfig {
            thread_pool_size: 10,
            job_count: 100,
            run_time_ms: 200,
            progress_interval_ms: 50,
            suite: Suite,
            job_factory: Box::new(move || Job {
                job_tick_count: job_tick_count_clone.clone(),
            }),
        });

        println!("job tick count: {}", *job_tick_count.lock().unwrap());
    }
}
