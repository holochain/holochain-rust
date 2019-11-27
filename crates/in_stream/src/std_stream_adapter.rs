use crate::*;
use std::io::{Read, Result, Write};

#[derive(Shrinkwrap, Debug)]
#[shrinkwrap(mutable)]
pub struct StdStreamAdapter<T>(pub T);

impl<T> StdStreamAdapter<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Read for StdStreamAdapter<T>
where
    for<'a> T: InStream<&'a mut [u8], &'a [u8]>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl<T> Write for StdStreamAdapter<T>
where
    for<'a> T: InStream<&'a mut [u8], &'a [u8]>,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.flush()
    }
}
