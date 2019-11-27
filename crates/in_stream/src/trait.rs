use crate::*;
use std::{fmt::Debug, io::Result};
use url2::prelude::*;

pub trait InStreamListener<R: Sized + Debug + Send + Sync, W: Sized + Debug + Send + Sync>:
    Sized + Debug + Send + Sync
{
    type Stream: InStream<R, W>;
    type BindConfig: Default;

    fn bind(url: &Url2, config: Self::BindConfig) -> Result<Self>;
    fn binding(&self) -> Url2;
    fn accept(&mut self) -> Result<<Self as InStreamListener<R, W>>::Stream>;
}

pub trait InStream<R: Sized + Debug + Send + Sync, W: Sized + Debug + Send + Sync>:
    Sized + Debug + Send + Sync
{
    type ConnectConfig: Default;

    const URL_SCHEME: &'static str;

    fn connect(url: &Url2, config: Self::ConnectConfig) -> Result<Self>;
    fn read(&mut self, data: R) -> Result<usize>;
    fn write(&mut self, data: W) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
}

pub trait InStreamStd<R: Sized + Debug + Send + Sync, W: Sized + Debug + Send + Sync>:
    InStream<R, W>
{
    fn into_std_stream(self) -> StdStreamAdapter<Self> {
        StdStreamAdapter(self)
    }
}
