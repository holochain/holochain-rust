use super::{net_connection::*, protocol::Protocol, NetResult};

/// a simple pass-through NetSend instance
/// this struct can be use to compose one type of NetWorker into another
pub struct NetConnectionRelay {
    worker: Box<NetWorker>,
    done: NetShutdown,
}

impl NetSend for NetConnectionRelay {
    /// send a message to the worker within this NetConnectionRelay instance
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.worker.receive(data)?;
        Ok(())
    }
}

impl NetConnectionRelay {
    ///
    pub fn stop(self) -> NetResult<()> {
        self.worker.stop()?;
        if let Some(done) = self.done {
            done();
        }
        Ok(())
    }

    /// call tick to perform any worker upkeep
    pub fn tick(&mut self) -> NetResult<bool> {
        self.worker.tick()
    }

    /// create a new NetSendRelay instance with given handler & factory
    pub fn new(
        handler: NetHandler,
        worker_factory: NetWorkerFactory,
        done: NetShutdown,
    ) -> NetResult<Self> {
        Ok(NetConnectionRelay {
            worker: worker_factory(handler)?,
            done,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::mpsc;

    struct DefWorker;

    impl NetWorker for DefWorker {}

    #[test]
    fn it_can_defaults() {
        let mut con = NetConnectionRelay::new(
            Box::new(move |_r| Ok(())),
            Box::new(|_h| Ok(Box::new(DefWorker) as Box<NetWorker>)),
            None,
        )
        .unwrap();

        con.send("test".into()).unwrap();
        con.tick().unwrap();
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
    fn it_invokes_connection_relay() {
        let (sender, receiver) = mpsc::channel();

        let mut con = NetConnectionRelay::new(
            Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            }),
            Box::new(|h| Ok(Box::new(SimpleWorker { handler: h }) as Box<NetWorker>)),
            None,
        )
        .unwrap();

        con.send("test".into()).unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("test".to_string(), String::from(res.as_json_string()));

        con.stop().unwrap();
    }

    #[test]
    fn it_can_tick() {
        let (sender, receiver) = mpsc::channel();

        let mut con = NetConnectionRelay::new(
            Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            }),
            Box::new(|h| Ok(Box::new(SimpleWorker { handler: h }) as Box<NetWorker>)),
            None,
        )
        .unwrap();

        con.tick().unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("tick".to_string(), String::from(res.as_json_string()));

        con.stop().unwrap();
    }
}
