impl NetWorker for MockMockWorker {
    fn stop(self: Box<Self>) -> NetResult<()> {
        Ok(())
    }

    fn receive(&mut self, data: Protocol) -> NetResult<()> {
        Ok(())
    }

    fn tick(&mut self) -> NetResult<bool> {
        Ok(false)
    }
}
