use super::{net_connection::*, NetResult};

use lib3h_protocol::protocol_client::Lib3hClientProtocol;
/// a simple pass-through NetSend instance
/// this struct can be use to compose one type of NetWorker into another
pub struct NetConnectionRelay {
    worker: Box<NetWorker>,
    done: NetShutdown,
}

impl NetSend for NetConnectionRelay {
    /// send a message to the worker within this NetConnectionRelay instance
    fn send(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
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
    use crossbeam_channel::unbounded;

    use lib3h_protocol::{
        protocol_server::Lib3hServerProtocol,
        data_types::GenericResultData
    };
    use holochain_persistence_api::hash::HashString;


    struct DefWorker;

    impl NetWorker for DefWorker {}

  //  #[test]
    fn it_can_defaults() {
        let mut con = NetConnectionRelay::new(
            NetHandler::new(Box::new(move |_r| Ok(()))),
            Box::new(|_h| Ok(Box::new(DefWorker) as Box<NetWorker>)),
            None,
        )
        .unwrap();

        con.send(Lib3hClientProtocol::SuccessResult(GenericResultData {
            request_id: "test_req_id".into(),
            space_address: HashString::from("test_space"),
            to_agent_id: HashString::from("test-agent"),
            result_info: vec![]
        }));
        con.tick().unwrap();
        con.stop().unwrap();
    }

    struct SimpleWorker {
        handler: NetHandler,
    }

    fn success_server_result() -> Lib3hServerProtocol {
        Lib3hServerProtocol::SuccessResult(GenericResultData {
            request_id: "test_req_id".into(),
            space_address: HashString::from("test_space"),
            to_agent_id: HashString::from("test-agent"),
            result_info: "tick".to_string().into_bytes()
        })
    }

    fn success_client_result() -> Lib3hClientProtocol {
        Lib3hClientProtocol::SuccessResult(GenericResultData {
            request_id: "test_req_id".into(),
            space_address: HashString::from("test_space"),
            to_agent_id: HashString::from("test-agent"),
            result_info: "tick".to_string().into_bytes()
        })
    }

    impl NetWorker for SimpleWorker {
        fn tick(&mut self) -> NetResult<bool> {
        self.handler.handle(Ok(success_server_result()));
        Ok(true)
        }

        fn receive(&mut self, _data: Lib3hClientProtocol) -> NetResult<()> {
            // TODO BLOCKER how / why to convert beteen client / server her?
            self.handler.handle(Ok(success_server_result()))
        }
    }

    #[test]
    fn it_invokes_connection_relay() {
        let (sender, receiver) = unbounded();

        let mut con = NetConnectionRelay::new(
            NetHandler::new(Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            })),
            Box::new(|h| Ok(Box::new(SimpleWorker { handler: h }) as Box<NetWorker>)),
            None,
        )
        .unwrap();

        con.send(success_client_result()).unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!(res, success_server_result());

        con.stop().unwrap();
    }

    #[test]
    fn it_can_tick() {
        let (sender, receiver) = unbounded();

        let mut con = NetConnectionRelay::new(
            NetHandler::new(Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            })),
            Box::new(|h| Ok(Box::new(SimpleWorker { handler: h }) as Box<NetWorker>)),
            None,
        )
        .unwrap();

        con.tick().unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!(res, success_server_result());

        con.stop().unwrap();
    }
}
