//! This module uses lazy_static! to make the zmq::Context easier to work with
//! Just make sure to call IpcClient::destroy_context() when ready

use super::errors::*;
use std;
use zmq;

lazy_static! {
    /// pseudo global for zmq::Context
    pub static ref ZMQ_CTX: std::sync::Mutex<zmq::Context> =
        std::sync::Mutex::new(zmq::Context::new());
}

/// Create a new zmq socket using the lazy_static! global
pub fn socket(socket_type: zmq::SocketType) -> Result<zmq::Socket> {
    match ZMQ_CTX.lock() {
        Ok(s) => Ok(s.socket(socket_type)?),
        Err(_) => bail_generic!("cannot access zmq context"),
    }
}

/// Destroy the lazy_static! global
pub fn destroy() -> Result<()> {
    match ZMQ_CTX.lock() {
        Ok(mut s) => s.destroy()?,
        Err(_) => bail_generic!("cannot access zmq context"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_create() {
        socket(zmq::ROUTER).unwrap();
        //destroy().unwrap();
    }
}
