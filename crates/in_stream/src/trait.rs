use crate::*;
use std::{
    fmt::Debug,
    io::{Read, Result, Write},
};
use url2::prelude::*;

macro_rules! make_listener {
    ($name:ident, $part:ident) => {
        /// a listener implementation that accepts partial streams
        pub trait $name: Sized + Debug + Send + Sync {
            type Partial: $part;
            type BindConfig: Default;

            /// bind to a network interface
            fn bind(url: &Url2, config: Self::BindConfig) -> Result<Self>;

            /// return the bound url
            fn binding(&self) -> Url2;

            /// accept any pending connections, or return WouldBlock
            fn accept(&mut self) -> Result<<Self as $name>::Partial>;

            /// block the current thread until we accept a connection
            fn accept_blocking(&mut self) -> Result<<Self as $name>::Partial> {
                loop {
                    match self.accept() {
                        Ok(s) => return Ok(s),
                        Err(ref e) if e.would_block() => (),
                        Err(e) => return Err(e),
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        }
    };
}

macro_rules! make_partial {
    ($name:ident, $stream:ident) => {
        /// a partial stream that can be processed to produce a real stream
        pub trait $name: Sized + Debug + Send + Sync {
            type Stream: $stream;
            type ConnectConfig: Default;

            /// the url scheme expected by this instance
            const URL_SCHEME: &'static str;

            /// sometimes you need to re-wrap a result stream as a partial
            /// to pass into a higher level stream
            fn with_stream(stream: Self::Stream) -> Result<Self>;

            /// establish a connection to a remote listener
            fn connect(url: &Url2, config: Self::ConnectConfig) -> Result<Self>;

            /// attempt to process any required handshaking
            /// will either return a full stream, WouldBlock, or other io::Error
            fn process(&mut self) -> Result<Self::Stream>;

            /// block the current thread until process returns not WouldBlock
            fn process_blocking(&mut self) -> Result<Self::Stream> {
                loop {
                    match self.process() {
                        Ok(s) => return Ok(s),
                        Err(ref e) if e.would_block() => (),
                        Err(e) => return Err(e),
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        }
    };
}

make_listener!(InStreamListener, InStreamPartial);

make_partial!(InStreamPartial, InStream);

/// a stream implementing the core std::io::Read and std::io::Write traits
pub trait InStream: Sized + Debug + Read + Write + Send + Sync {}

make_listener!(InStreamFramedListener, InStreamFramedPartial);
make_partial!(InStreamFramedPartial, InStreamFramed);

/// a framed stream designed to read and write whole messages at once
pub trait InStreamFramed: Sized + Debug + Send + Sync {
    type FrameType: Sized + Debug + Send + Sync;

    /// read one frame (or WouldBlock)
    fn read_frame<T: From<Self::FrameType>>(&mut self) -> Result<T>;

    /// write one frame
    fn write_frame<T: Into<Self::FrameType>>(&mut self, data: T) -> Result<()>;
}
