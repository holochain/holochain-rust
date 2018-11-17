use super::NetResult;

use super::{
    net_connection::{NetConnection, NetHandler, NetWorkerFactory},
    protocol::Protocol,
};

use std::{thread, time};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};

/// a NetConnection instance that is managed on another thread
#[derive(Debug)]
pub struct NetConnectionThread {
    keep_running: Arc<AtomicBool>,
    send_channel: mpsc::Sender<Protocol>,
    thread: thread::JoinHandle<()>,
}

impl NetConnection for NetConnectionThread {
    /// send a message to the worker within this NetConnectionThread instance
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.send_channel.send(data)?;
        Ok(())
    }
}

impl NetConnectionThread {
    /// stop (join) the worker thread
    pub fn stop(self) -> NetResult<()> {
        self.keep_running.store(false, Ordering::Relaxed);
        match self.thread.join() {
            Ok(_) => Ok(()),
            Err(_) => {
                bail!("NetConnectionThread failed to join on stop() call");
            }
        }
    }

    /// create a new NetConnectionThread instance with given handler / worker
    pub fn new(handler: NetHandler, worker_factory: NetWorkerFactory) -> NetResult<Self> {
        let keep_running = Arc::new(AtomicBool::new(true));
        let keep_running2 = keep_running.clone();

        let (sender, receiver) = mpsc::channel();
        Ok(NetConnectionThread {
            keep_running,
            send_channel: sender,
            thread: thread::spawn(move || {
                let mut us = 100_u64;
                let mut worker = worker_factory(handler).unwrap_or_else(|e| panic!("{:?}", e));

                while keep_running2.load(Ordering::Relaxed) {
                    let mut did_something = false;
                    receiver
                        .try_recv()
                        .and_then(|data| {
                            did_something = true;
                            worker.receive(data).unwrap_or_else(|e| panic!("{:?}", e));
                            Ok(())
                        })
                        .unwrap_or(());

                    worker
                        .tick()
                        .and_then(|b| {
                            if b {
                                did_something = true;
                            }
                            Ok(())
                        })
                        .unwrap_or_else(|e| panic!("{:?}", e));

                    if did_something {
                        us = 100_u64;
                    } else {
                        us *= 2_u64;
                        if us > 10_000_u64 {
                            us = 10_000_u64;
                        }
                    }

                    thread::sleep(time::Duration::from_micros(us));
                }
            }),
        })
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
        ).unwrap();

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
        ).unwrap();

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
        ).unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("tick".to_string(), String::from(res.as_json_string()));

        con.stop().unwrap();
    }
}
