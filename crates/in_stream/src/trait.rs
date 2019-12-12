use crate::*;
use std::{
    fmt::Debug,
    io::{Error, ErrorKind, Result},
};
use url2::prelude::*;

/// dynamic type for passing around in_stream configuration
/// this is a stopgap until rust allows better constraints on associated types
pub type InStreamConfigAny = Box<dyn std::any::Any + 'static + Send>;

/// mixin helper for converting structs into an Any and back again
pub trait InStreamConfig: 'static + Sized + Debug + Send {
    /// given an any, attempt to downcast to a concrete type
    fn from_any(any: InStreamConfigAny) -> Result<Self> {
        match any.downcast::<Self>() {
            Ok(v) => Ok(*v),
            Err(_) => Err(Error::new(ErrorKind::InvalidInput, "bad config type")),
        }
    }

    /// convert a generic type into a concrete one
    fn from_gen<C: InStreamConfig>(c: C) -> Result<Self> {
        Self::from_any(c.to_any())
    }

    /// convert this type into an any
    fn to_any(self) -> InStreamConfigAny {
        Box::new(self)
    }
}

// allow unit type to be used for configuration
impl InStreamConfig for () {}

// if you've already got an any, we also want to be able to use that
impl InStreamConfig for InStreamConfigAny {
    fn to_any(self) -> InStreamConfigAny {
        // we're already a box, don't re-box
        self
    }
}

/// implement this trait to provide listening/server socket type functionality
pub trait InStreamListener<R: Sized + Debug + Send + Sync, W: Sized + Debug + Send + Sync>:
    Sized + Debug + Send + Sync
{
    type Stream: InStream<R, W>;

    /// begin listening on the given url spec
    /// this function does the actual work of binding, but it is recommended
    /// your struct provide a wrapper with a concrete config type
    fn raw_bind<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self>;

    /// access the url for the bound interface
    fn binding(&self) -> Url2;

    /// attempt to accept a stream/socket from this binding
    /// may return Err(ErrorKind::WouldBlock.into())
    fn accept(&mut self) -> Result<<Self as InStreamListener<R, W>>::Stream>;
}

/// implement this if your listener accepts std::io::{Read, Write} streams
pub trait InStreamListenerStd
where
    for<'a> Self: InStreamListener<&'a mut [u8], &'a [u8]>,
{
    type StreamStd: InStreamStd;

    /// use `accept_std` if you want your streams to work with byte data
    fn accept_std(&mut self) -> Result<<Self as InStreamListenerStd>::StreamStd>;
}

/// implement this trait to provide a stream endpoint / socket type connection
/// works like combined std::io::{Read, Write}, but with generic types
/// the underlying stream should be treated as non-blocking.
/// For example, if this is a TLS stream, we may still need to complete
/// a handshaking process before data can actually be written.
///
/// `read` will return WouldBlock if there is no pending data to be read
/// `write` will buffer any data if the stream is not ready to write
/// `flush` will block until pending data has been written
pub trait InStream<R: Sized + Debug + Send + Sync, W: Sized + Debug + Send + Sync>:
    Sized + Debug + Send + Sync
{
    /// your implementation should work with a single url scheme/protocol
    /// if you need to support multiple (i.e. ws:// vs wss://) consider
    /// using macros/code generation to DRY
    const URL_SCHEME: &'static str;

    /// create a new connection / stream of this type.
    /// this function does the actual work of connecting, but it is recommended
    /// your struct provide a wrapper with a concrete config type
    fn raw_connect<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self>;

    /// access the remote url this connection represents
    fn remote_url(&self) -> Url2;

    /// non-blocking read.
    /// if R is an array-type, success result is number of elements read
    /// otherwise it is 1
    fn read(&mut self, data: R) -> Result<usize>;

    /// buffered write. Implementors should buffer data on WouldBlock
    fn write(&mut self, data: W) -> Result<usize>;

    /// blocking flush all pending buffered write data.
    fn flush(&mut self) -> Result<()>;
}

/// implement this if your stream deals with binary [u8] data
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
