use std::{convert::TryFrom, time::SystemTime};
use time::Iso8601;

#[derive(Default)]
pub struct ISODispatcherMock(i64);
pub struct ISODispatcherConcrete;

impl ISODispatcherMock {
    pub fn new(timestamp: i64) -> ISODispatcherMock {
        ISODispatcherMock(timestamp)
    }
}

pub trait ISODispatch: Send + Sync {
    fn now_dispatch(&self) -> String;
}

impl ISODispatch for ISODispatcherMock {
    fn now_dispatch(&self) -> String {
        Iso8601::try_from(self.0).unwrap().to_string()
    }
}

impl ISODispatch for ISODispatcherConcrete {
    fn now_dispatch(&self) -> String {
        let duration_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System time must not be before UNIX EPOCH");
        Iso8601::from(duration_since_epoch.as_secs()).to_string()
    }
}
