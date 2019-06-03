use chrono::offset::Utc;

#[derive(Default)]
pub struct UTCMock(i64);
pub struct UTCConcrete;

impl UTCMock {
    pub fn new(index: i64) -> UTCMock {
        UTCMock(index)
    }
}

pub trait UTCDispatch: Send + Sync {
    fn now_dispatch(&self) -> i64;
}

impl UTCDispatch for UTCMock {
    fn now_dispatch(&self) -> i64 {
        self.0
    }
}

impl UTCDispatch for UTCConcrete {
    fn now_dispatch(&self) -> i64 {
        Utc::now().timestamp()
    }
}
