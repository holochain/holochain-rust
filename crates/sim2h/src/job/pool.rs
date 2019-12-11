use crate::*;

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
            // invoke the default handler and exit the process
            orig_handler(panic_info);
            std::process::exit(1);
        }));
        true
    };
}

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
                    .spawn(move || loop {
                        {
                            if !*cont.f_lock() {
                                return;
                            }
                        }

                        if let Ok(mut job) = recv.try_recv() {
                            if job.run() {
                                send.f_send(job);
                            }
                        }

                        std::thread::yield_now();
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
