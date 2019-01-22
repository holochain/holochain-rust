use holochain_net_connection::{net_connection::NetWorker, protocol::Protocol, NetResult};

pub struct StubWorker {}

impl NetWorker for StubWorker {
    fn stop(self: Box<Self>) -> NetResult<()> {
        Ok(())
    }

    fn receive(&mut self, _: Protocol) -> NetResult<()> {
        Ok(())
    }

    fn tick(&mut self) -> NetResult<bool> {
        Ok(false)
    }
}

impl StubWorker {
    pub fn new() -> Self {
        StubWorker {}
    }
}
