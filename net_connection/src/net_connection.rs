use super::NetResult;
use crate::protocol::Protocol;

/// closure for getting Protocol messages from the p2p abstraction system
pub type NetHandler = Box<FnMut(NetResult<Protocol>) -> NetResult<()> + Send>;

/// closure for signaling shutdown incase of required cleanup
pub type NetShutdown = Option<Box<::std::boxed::FnBox() + Send>>;

/// net connection - a worker manager can send Protocol messages
pub trait NetConnection {
    fn send(&mut self, data: Protocol) -> NetResult<()>;
}

/// represents a worker that handles protocol messages
pub trait NetWorker {
    /// stop the worker
    fn stop(self: Box<Self>) -> NetResult<()> {
        Ok(())
    }

    /// when somebody has called `send` to send this worker a message
    fn receive(&mut self, _data: Protocol) -> NetResult<()> {
        Ok(())
    }

    /// perform any upkeep return `false` if there was no upkeep to perform
    fn tick(&mut self) -> NetResult<bool> {
        Ok(false)
    }
}

/// closure for instantiating a NetWorker
pub type NetWorkerFactory =
    Box<::std::boxed::FnBox(NetHandler) -> NetResult<Box<NetWorker>> + Send>;

/// a simple pass-through NetConnection instance
/// this struct can be use to compose one type of NetWorker into another
pub struct NetConnectionRelay {
    worker: Box<NetWorker>,
    done: NetShutdown,
}

impl NetConnection for NetConnectionRelay {
    /// send a message to the worker within this NetConnectionRelay instance
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.worker.receive(data)?;
        Ok(())
    }
}

impl NetConnectionRelay {
    /// stop this NetConnectionRelay instance
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

    /// create a new NetConnectionRelay instance with give handler / factory
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
    fn it_invokes_connection_relay() {
        let (sender, receiver) = mpsc::channel();

        let mut con = NetConnectionRelay::new(
            Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            }),
            Box::new(|h| Ok(Box::new(Worker { handler: h }) as Box<NetWorker>)),
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
            Box::new(|h| Ok(Box::new(Worker { handler: h }) as Box<NetWorker>)),
            None,
        )
        .unwrap();

        con.tick().unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("tick".to_string(), String::from(res.as_json_string()));

        con.stop().unwrap();
    }
}
