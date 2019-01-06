use holochain_net_connection::net_connection::NetWorker;
use holochain_net_connection::NetResult;
use holochain_net_connection::protocol::Protocol;

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
        MockMockWorker{}
    }
}
