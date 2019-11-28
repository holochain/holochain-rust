use crate::*;
use std::io::{Read, Result, Write};

#[derive(Shrinkwrap, Debug)]
#[shrinkwrap(mutable)]
pub struct StdStreamAdapter<T: InStreamStd> {
    #[shrinkwrap(main_field)]
    pub stream: T,
}

impl<T: InStreamStd> StdStreamAdapter<T> {
    pub fn new(stream: T) -> Self {
        Self {
            stream,
        }
    }

    pub fn into_inner(self) -> T {
        self.stream
    }
}

impl<T: InStreamStd> Read for StdStreamAdapter<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.stream.read(buf)
    }
}

impl<T: InStreamStd> Write for StdStreamAdapter<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.stream.flush()
    }
}
