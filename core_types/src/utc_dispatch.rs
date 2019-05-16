use chrono::offset::Utc;

pub struct UTCMock;
pub struct UTCConcrete;

pub trait UTCDispatch: Send + Sync
{
    fn now_dispatch(&self) -> i64;
}

impl UTCDispatch for UTCMock
{
    fn now_dispatch(&self)->i64
    {
        0
    }
}

impl UTCDispatch for UTCConcrete
{
    fn now_dispatch(&self) ->i64
    {
        Utc::now().timestamp_nanos()
    }
}