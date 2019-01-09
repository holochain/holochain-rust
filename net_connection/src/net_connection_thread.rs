use super::NetResult;

use super::{
    net_connection::{NetConnection, NetHandler, NetShutdown, NetWorkerFactory},
    protocol::Protocol,
};

use std::{thread, time};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};

/// Struct for handling a network connection that is managed on a separate thread.
/// Implements the NetConnection trait.
pub struct NetConnectionThread {
    can_keep_running: Arc<AtomicBool>,
    send_channel: mpsc::Sender<Protocol>,
    thread: thread::JoinHandle<()>,
    done: NetShutdown,
    pub endpoint: String,
}

impl NetConnection for NetConnectionThread {
    /// send a message to the worker within this NetConnectionThread instance
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.send_channel.send(data)?;
        Ok(())
    }
}

impl NetConnectionThread {
    /// Create a new NetConnectionThread instance with the given handler, worker, and shutdown
    pub fn new(
        handler: NetHandler,
        worker_factory: NetWorkerFactory,
        done: NetShutdown,
    ) -> NetResult<Self> {
        // Create atomic TRUE
        let can_keep_running = Arc::new(AtomicBool::new(true));
        let can_keep_running_shared = can_keep_running.clone();
        // Create channel
        let (send_channel, recv_channel) = mpsc::channel();
        let (send_endpoint, recv_endpoint) = mpsc::channel();

        // Spawn worker thread
        let thread = thread::spawn(move || {
            // Create worker
            let mut worker = worker_factory(handler).unwrap_or_else(|e| panic!("{:?}", e));

            let endpoint = worker.endpoint();
            send_endpoint
                .send(endpoint)
                .expect("Sending endpoint address should work.");
            drop(send_endpoint);
            // Loop as long NetConnectionThread wants to
            let mut sleep_duration_us = 100_u64;
            while can_keep_running_shared.load(Ordering::Relaxed) {
                // Check if we received something from the network
                let mut did_something = false;
                recv_channel
                    .try_recv()
                    .and_then(|data| {
                        // Send it to the worker
                        did_something = true;
                        worker.receive(data).unwrap_or_else(|e| panic!("{:?}", e));
                        Ok(())
                    })
                    .unwrap_or(());
                // Tick the worker (it might do a send)
                worker
                    .tick()
                    .and_then(|b| {
                        if b {
                            did_something = true;
                        }
                        Ok(())
                    })
                    .unwrap_or_else(|e| panic!("{:?}", e));

                // Increase sleep duration if nothing was received or sent
                if did_something {
                    sleep_duration_us = 100_u64;
                } else {
                    sleep_duration_us *= 2_u64;
                    if sleep_duration_us > 10_000_u64 {
                        sleep_duration_us = 10_000_u64;
                    }
                }
                // Sleep
                thread::sleep(time::Duration::from_micros(sleep_duration_us));
            }
            // Stop the worker
            worker.stop().unwrap_or_else(|e| panic!("{:?}", e));
        });

        // Retrieve endpoint from worker thread
        let endpoint = recv_endpoint
            .recv()
            .expect("Failed to receive endpoint address from net worker");
        let endpoint = endpoint
            .expect("Should have an endpoint address")
            .to_string();

        // Done
        Ok(NetConnectionThread {
            can_keep_running,
            send_channel,
            thread,
            done,
            endpoint,
        })
    }

    /// stop (join) the worker thread
    pub fn stop(self) -> NetResult<()> {
        self.can_keep_running.store(false, Ordering::Relaxed);
        if self.thread.join().is_err() {
            bail!("NetConnectionThread failed to join on stop() call");
        }
        if let Some(done) = self.done {
            done();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::net_connection::NetWorker;

    struct DefWorker;

    impl NetWorker for DefWorker {}

    #[test]
    fn it_can_defaults() {
        let mut con = NetConnectionThread::new(
            Box::new(move |_r| Ok(())),
            Box::new(|_h| Ok(Box::new(DefWorker) as Box<NetWorker>)),
            None,
        )
        .unwrap();

        con.send("test".into()).unwrap();
        con.stop().unwrap();
    }

    struct Worker {
        handler: NetHandler,
    }

    impl NetWorker for Worker {
        fn tick(&mut self) -> NetResult<bool> {
            (self.handler)(Ok("tick".into()))?;
            Ok(true)
        }

        fn receive(&mut self, data: Protocol) -> NetResult<()> {
            (self.handler)(Ok(data))
        }
    }

    #[test]
    fn it_invokes_connection_thread() {
        let (sender, receiver) = mpsc::channel();

        let mut con = NetConnectionThread::new(
            Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            }),
            Box::new(|h| Ok(Box::new(Worker { handler: h }) as Box<NetWorker>)),
            None,
        )
        .unwrap();

        con.send("test".into()).unwrap();

        let res;
        loop {
            let tmp = receiver.recv().unwrap();

            if &(String::from(tmp.as_json_string())) == "tick" {
                continue;
            } else {
                res = tmp;
                break;
            }
        }

        assert_eq!("test".to_string(), String::from(res.as_json_string()));

        con.stop().unwrap();
    }

    #[test]
    fn it_can_tick() {
        let (sender, receiver) = mpsc::channel();

        let con = NetConnectionThread::new(
            Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            }),
            Box::new(|h| Ok(Box::new(Worker { handler: h }) as Box<NetWorker>)),
            None,
        )
        .unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("tick".to_string(), String::from(res.as_json_string()));

        con.stop().unwrap();
    }
}
