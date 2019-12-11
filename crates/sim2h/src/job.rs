/// typedef for code clarity
pub(crate) type JobContinue = bool;

/// an item that can be executed via thread pool
pub(crate) trait Job: 'static + Send {
    /// execute one iteration of this job - try to be as short-lived as possible
    fn run(&mut self) -> JobContinue;
}

mod pool;
pub(crate) use pool::*;

mod tick;
pub(crate) use tick::*;

mod listen;
pub(crate) use listen::*;

mod connection;
pub(crate) use connection::*;
