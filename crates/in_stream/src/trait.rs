use crate::*;
use std::{
    fmt::Debug,
    io::{Error, ErrorKind, Result},
};
use url2::prelude::*;

pub type InStreamConfigAny = Box<dyn std::any::Any + 'static + Send>;

pub trait InStreamConfig: 'static + Sized + Debug + Send {
    fn from_any(any: InStreamConfigAny) -> Result<Self> {
        match any.downcast::<Self>() {
            Ok(v) => Ok(*v),
            Err(_) => Err(Error::new(ErrorKind::InvalidInput, "bad config type")),
        }
    }

    fn from_gen<C: InStreamConfig>(c: C) -> Result<Self> {
        Self::from_any(c.to_any())
    }

    fn to_any(self) -> InStreamConfigAny {
        Box::new(self)
    }
}

impl InStreamConfig for () {}
impl InStreamConfig for InStreamConfigAny {
    fn to_any(self) -> InStreamConfigAny {
        // we're already a box, don't re-box
        self
    }
}

pub trait InStreamListener<R: Sized + Debug + Send + Sync, W: Sized + Debug + Send + Sync>:
    Sized + Debug + Send + Sync
{
    type Stream: InStream<R, W>;

    fn raw_bind<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self>;

    fn binding(&self) -> Url2;
    fn accept(&mut self) -> Result<<Self as InStreamListener<R, W>>::Stream>;
}

pub trait InStreamListenerStd
where
    for<'a> Self: InStreamListener<&'a mut [u8], &'a [u8]>,
{
    type StreamStd: InStreamStd;

    fn accept_std(&mut self) -> Result<<Self as InStreamListenerStd>::StreamStd>;
}

pub trait InStream<R: Sized + Debug + Send + Sync, W: Sized + Debug + Send + Sync>:
    Sized + Debug + Send + Sync
{
    const URL_SCHEME: &'static str;

    fn raw_connect<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self>;

    fn read(&mut self, data: R) -> Result<usize>;
    fn write(&mut self, data: W) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
}

pub trait InStreamStd
where
    for<'a> Self: InStream<&'a mut [u8], &'a [u8]>,
{
    fn into_std_stream(self) -> StdStreamAdapter<Self> {
        StdStreamAdapter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_use_struct_or_any_for_config() {
        #[derive(Debug, Clone)]
        struct S {
            data: String,
        }

        impl InStreamConfig for S {}

        fn test_either<C: InStreamConfig>(c: C) {
            let c = S::from_gen(c).unwrap();
            assert_eq!("test_string", &c.data);
        }

        let s = S {
            data: "test_string".to_string(),
        };

        test_either(s.clone());
        test_either(s.clone().to_any());
    }
}
