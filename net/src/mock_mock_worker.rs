use holochain_net_connection::{net_connection::NetWorker, protocol::Protocol, NetResult};

pub struct MockMockWorker {}

impl NetWorker for MockMockWorker {
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

impl MockMockWorker {
    pub fn new() -> Self {
        MockMockWorker {}
    }
}
