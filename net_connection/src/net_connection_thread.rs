use super::{
    net_connection::{NetHandler, NetSend, NetShutdown, NetWorkerFactory},
    protocol::Protocol,
    NetResult,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread, time,
};

/// Struct for holding a network connection running on a separate thread.
/// It is itself a NetSend, and spawns a NetWorker.
pub struct NetConnectionThread {
    can_keep_running: Arc<AtomicBool>,
    send_channel: mpsc::Sender<Protocol>,
    thread: thread::JoinHandle<()>,
    done: NetShutdown,
    pub endpoint: String,
}

impl NetSend for NetConnectionThread {
    /// send a message to the worker within NetConnectionThread's child thread.
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.send_channel.send(data)?;
        Ok(())
    }
}

impl NetConnectionThread {
    /// NetSendThread Constructor.
    /// Spawns a thread that will create and run a NetWorker with the given factory, handler and
    /// shutdown closure.
    pub fn new(
        handler: NetHandler,
        worker_factory: NetWorkerFactory,
        done: NetShutdown,
    ) -> NetResult<Self> {
        // Create shared bool between self and spawned thread
        let can_keep_running = Arc::new(AtomicBool::new(true));
        let can_keep_running_child = can_keep_running.clone();
        // Create channels between self and spawned thread
        let (send_channel, recv_channel) = mpsc::channel();
        let (send_endpoint, recv_endpoint) = mpsc::channel();

        // Spawn worker thread
        let thread = thread::spawn(move || {
            // Create worker
            let mut worker = worker_factory(handler).unwrap_or_else(|e| panic!("{:?}", e));
            // Get endpoint and send it to owner (NetConnectionThread)
            let endpoint = worker.endpoint();
            send_endpoint
                .send(endpoint)
                .expect("Sending endpoint address should work.");
            drop(send_endpoint);
            // Loop as long owner wants to
            let mut sleep_duration_us = 100_u64;
            while can_keep_running_child.load(Ordering::Relaxed) {
                // Check if we received something from parent (NetConnectionThread::send())
                let mut did_something = false;
                recv_channel
                    .try_recv()
                    .and_then(|data| {
                        // Received data from parent
                        // Have the worker handle it
                        did_something = true;
                        worker.receive(data).unwrap_or_else(|e| eprintln!("Error occured in p2p network module: {:?}", e));
                        Ok(())
                    })
                    .unwrap_or(());
                // Tick the worker
                // (it might call the handler if it received a message from the network)
                worker
                    .tick()
                    .and_then(|b| {
                        if b {
                            did_something = true;
                        }
                        Ok(())
                    })
                    .unwrap_or_else(|e| eprintln!("Error occured in p2p network module: {:?}", e));

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

        // Retrieve endpoint from spawned thread
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

    /// stop the worker thread (join)
    pub fn stop(self) -> NetResult<()> {
        // tell child thread to stop running
        self.can_keep_running.store(false, Ordering::Relaxed);
        if self.thread.join().is_err() {
            bail!("NetConnectionThread failed to join on stop() call");
        }
        // Call shutdown closure if any
        if let Some(done) = self.done {
            done();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::net_connection::NetWorker, *};

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

    struct SimpleWorker {
        handler: NetHandler,
    }

    impl NetWorker for SimpleWorker {
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
            Box::new(|h| Ok(Box::new(SimpleWorker { handler: h }) as Box<NetWorker>)),
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
            Box::new(|h| Ok(Box::new(SimpleWorker { handler: h }) as Box<NetWorker>)),
            None,
        )
        .unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("tick".to_string(), String::from(res.as_json_string()));

        con.stop().unwrap();
    }
}
