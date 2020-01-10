use crate::*;
use holochain_metrics::{metrics::MetricPublisher, with_latency_publishing};

/// a cpu-count pool of threads that can execute jobs
pub(crate) struct Pool {
    job_cont: Arc<Mutex<bool>>,
    job_threads: Vec<std::thread::JoinHandle<()>>,
    job_send: crossbeam_channel::Sender<Box<dyn Job>>,
}

lazy_static! {
    static ref SET_THREAD_PANIC_FATAL: bool = {
        let orig_handler = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            eprintln!("THREAD PANIC {:#?}", panic_info);
            // invoke the default handler and exit the process
            orig_handler(panic_info);
            std::process::exit(1);
        }));
        true
    };
}

[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
impl Pool {
    pub(crate) fn new() -> Self {
        // make sure if a thread panics, the whole process exits
        assert!(*SET_THREAD_PANIC_FATAL);

        let (job_send, job_recv) = crossbeam_channel::unbounded::<Box<dyn Job>>();
        let job_cont = Arc::new(Mutex::new(true));
        let mut job_threads = Vec::new();
        for cpu_index in 0..num_cpus::get() {
            let cont = job_cont.clone();
            let send = job_send.clone();
            let recv = job_recv.clone();
            job_threads.push(
                std::thread::Builder::new()
                    .name(format!("sim2h-pool-thread-{}", cpu_index))
                    .spawn(move || {
                        let mut parked_jobs = Vec::new();
                        let metric_publisher: std::sync::Arc<
                            std::sync::RwLock<holochain_metrics::logger::LoggerMetricPublisher>,
                        > = Default::default();
                        loop {
                            {
                                if !*cont.f_lock() {
                                    return;
                                }
                            }

                            let thread = std::thread::current();
                            let thread_name = thread.name().unwrap_or_else(|| "sim2h-pool-thread");

                            let parked_jobs_metric = holochain_metrics::Metric::new_timestamped_now(
                                format!("{}.parked_jobs", thread_name),
                                None,
                                parked_jobs.len() as f64,
                            );

                            metric_publisher
                                .write()
                                .unwrap()
                                .publish(&parked_jobs_metric);

                            let now = std::time::Instant::now();
                            let chk = parked_jobs.drain(..).collect::<Vec<_>>();
                            for (t, job) in chk {
                                if now >= t {
                                    send.f_send(job);
                                } else {
                                    parked_jobs.push((t, job));
                                }
                            }

                            if let Ok(mut job) =
                                recv.recv_timeout(std::time::Duration::from_millis(5))
                            {
                                let r = with_latency_publishing!(
                                    format!("{}.job", thread_name),
                                    metric_publisher,
                                    || job.run()
                                );
                                if r.cont {
                                    if r.wait_ms == 0 {
                                        send.f_send(job);
                                    } else {
                                        parked_jobs.push((
                                            std::time::Instant::now()
                                                .checked_add(std::time::Duration::from_millis(
                                                    r.wait_ms,
                                                ))
                                                .unwrap(),
                                            job,
                                        ));
                                    }
                                }
                            }

                            std::thread::sleep(std::time::Duration::from_millis(5));
                        }
                    })
                    .unwrap(),
            );
        }
        Self {
            job_cont,
            job_threads,
            job_send,
        }
    }

    pub(crate) fn push_job(&self, job: Box<dyn Job>) {
        self.job_send.send(job).expect("failed to send job");
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        *self.job_cont.f_lock() = false;
        for thread in self.job_threads.drain(..) {
            // ignore poisoned threads... we're shutting down anyways
            let _ = thread.join();
        }
    }
}
