extern crate crossbeam_channel;
extern crate num_cpus;

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

/// utitily for recording stress test metrics
#[derive(Clone)]
pub struct StressJobMetricLogger {
    job_index: usize,
    log_send: crossbeam_channel::Sender<StressJobLog>,
}

impl StressJobMetricLogger {
    /// private constructor
    fn priv_new(job_index: usize, log_send: crossbeam_channel::Sender<StressJobLog>) -> Self {
        Self {
            job_index,
            log_send,
        }
    }

    /// log a metric with a name, such as
    /// `log("received_pong_count", 1.0)`
    pub fn log(&mut self, name: &str, value: f64) {
        self.log_send
            .send(StressJobLog {
                job_index: self.job_index,
                name: name.to_string(),
                value,
            })
            .unwrap();
    }
}

/// respond if you want this job to continue or not
pub struct StressJobTickResult {
    /// true if this job should continue
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
    /// tick will be called periodically on your job by the stress suite
    fn tick(&mut self, logger: &mut StressJobMetricLogger) -> StressJobTickResult;
}

/// please provide a factory function for stress jobs
pub type JobFactory<J> = Box<dyn FnMut(StressJobMetricLogger) -> J + 'static + Send + Sync>;

/// an individual stress metric
#[derive(Debug, Clone)]
pub struct StressLogStats {
    pub count: u64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

/// a collection of stress stats for a whole suite run
#[derive(Debug, Clone)]
pub struct StressStats {
    pub master_tick_count: u64,
    pub log_stats: HashMap<String, StressLogStats>,
}

/// internal job metric log struct
#[derive(Debug, Clone)]
struct StressJobLog {
    pub job_index: usize,
    pub name: String,
    pub value: f64,
}

/// a struct implementing this trait can serve as a stress suite for a test
pub trait StressSuite: 'static {
    fn start(&mut self, logger: StressJobMetricLogger);
    fn warmup_complete(&mut self) {}
    fn progress(&mut self, stats: &StressStats);
    fn stop(&mut self, stats: StressStats);
    fn tick(&mut self) {}
}

/// configure the stress suite runner with these parameters
pub struct StressRunConfig<S: StressSuite, J: StressJob> {
    /// how many threads should be spun up in the job management thread pool
    pub thread_pool_size: usize,
    /// how many total jobs should we try to keep arount
    pub job_count: usize,
    /// the total runtime of the stress test run
    pub run_time_ms: u64,
    /// how often should we report progress statistics
    pub progress_interval_ms: u64,
    /// the suite to execute
    pub suite: S,
    /// the job factory for creating individual jobs
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

/// internal job tracking struct
struct StressJobInfo<J: StressJob> {
    job: J,
    logger: StressJobMetricLogger,
}

/// internal stress runner struct
struct StressRunner<S: StressSuite, J: StressJob> {
    config: StressRunConfig<S, J>,
    is_warmup: bool,
    warmup_target: std::time::Instant,
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
    /// private stress runner constructor
    #[allow(clippy::mutex_atomic)]
    fn priv_new(config: StressRunConfig<S, J>) -> Self {
        let (log_send, log_recv) = crossbeam_channel::unbounded();

        let warmup_target = std::time::Instant::now()
            .checked_add(std::time::Duration::from_millis(5000))
            .unwrap();

        let run_until = std::time::Instant::now()
            .checked_add(std::time::Duration::from_millis(5000 + config.run_time_ms))
            .unwrap();

        let next_progress = std::time::Instant::now()
            .checked_add(std::time::Duration::from_millis(
                5000 + config.progress_interval_ms,
            ))
            .unwrap();

        let mut runner = StressRunner {
            config,
            is_warmup: true,
            warmup_target,
            run_until,
            next_progress,
            thread_pool: Vec::new(),
            should_continue: Arc::new(Mutex::new(true)),
            job_count: Arc::new(Mutex::new(0)),
            job_queue: Arc::new(Mutex::new(VecDeque::new())),
            job_last_index: 1,
            log_recv,
            log_send,
            stats: StressStats {
                master_tick_count: 0,
                log_stats: HashMap::new(),
            },
        };

        let cpu_count = if runner.config.thread_pool_size == 0 {
            num_cpus::get()
        } else {
            runner.config.thread_pool_size
        };
        for _ in 0..cpu_count {
            runner.create_thread();
        }

        runner
            .config
            .suite
            .start(StressJobMetricLogger::priv_new(0, runner.log_send.clone()));

        runner
    }

    /// give the stress runner some processor time
    fn tick(&mut self) -> bool {
        if std::time::Instant::now() > self.run_until {
            *self.should_continue.lock().unwrap() = false;
            return false;
        }
        {
            let mut cur_job_count = self.job_count.lock().unwrap();
            let mut job_queue = self.job_queue.lock().unwrap();
            while *cur_job_count < self.config.job_count {
                let logger =
                    StressJobMetricLogger::priv_new(self.job_last_index, self.log_send.clone());
                let job = StressJobInfo {
                    job: (self.config.job_factory)(logger.clone()),
                    logger,
                };
                (*job_queue).push_front(job);
                self.job_last_index += 1;
                *cur_job_count += 1
            }
        }

        self.config.suite.tick();

        // just a guard incase logs are generated faster than pulled off here
        // we want it to end at some point : )
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

        if std::time::Instant::now() < self.warmup_target {
            return true;
        }

        if self.is_warmup {
            // we've completed our warmup period,
            // let's reset our statistics
            self.is_warmup = false;
            self.stats.master_tick_count = 0;
            self.stats.log_stats = HashMap::new();
            self.config.suite.warmup_complete();
        }

        self.stats.master_tick_count += 1;

        if std::time::Instant::now() >= self.next_progress {
            self.next_progress = std::time::Instant::now()
                .checked_add(std::time::Duration::from_millis(
                    self.config.progress_interval_ms,
                ))
                .unwrap();
            self.config.suite.progress(&self.stats);
        }

        true
    }

    /// stress runner shutdown logic
    fn cleanup(mut self) {
        *self.should_continue.lock().unwrap() = false;
        for t in self.thread_pool.drain(..) {
            t.join().expect("failed to join thread, poisoned?");
        }
        self.config.suite.stop(self.stats);
    }

    /// spawn a single thread-pool thread
    fn create_thread(&mut self) {
        let should_continue = self.should_continue.clone();
        let job_count = self.job_count.clone();
        let job_queue = self.job_queue.clone();
        self.thread_pool.push(std::thread::spawn(move || loop {
            if !*should_continue.lock().unwrap() {
                return;
            }
            let mut job = match (*job_queue.lock().unwrap()).pop_front() {
                Some(job) => job,
                None => continue,
            };
            let start = std::time::Instant::now();
            let result = job.job.tick(&mut job.logger);
            job.logger
                .log("job_tick_elapsed_ms", start.elapsed().as_millis() as f64);
            if result.should_continue {
                (*job_queue.lock().unwrap()).push_back(job);
            } else {
                *job_count.lock().unwrap() -= 1;
            }
        }));
    }
}

/// execute a single run of a stress test suite, with given config parameters
pub fn stress_run<S: StressSuite, J: StressJob>(config: StressRunConfig<S, J>) {
    let mut runner = StressRunner::priv_new(config);
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
            fn start(&mut self, _: StressJobMetricLogger) {
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
            job_factory: Box::new(move |_| Job {
                job_tick_count: job_tick_count_clone.clone(),
            }),
        });

        println!("job tick count: {}", *job_tick_count.lock().unwrap());
    }
}
