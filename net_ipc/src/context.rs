use std;
use errors::*;
use zmq;

lazy_static! {
    pub static ref ZMQ_CTX: std::sync::Mutex<zmq::Context> =
        std::sync::Mutex::new(zmq::Context::new());
}

pub fn socket (socket_type: zmq::SocketType) -> Result<zmq::Socket> {
    match ZMQ_CTX.lock() {
        Ok(s) => Ok(s.socket(socket_type)?),
        Err(_) => gerr!("cannot access zmq context"),
    }
}

pub fn destroy () -> Result<()> {
    match ZMQ_CTX.lock() {
        Ok(mut s) => s.destroy()?,
        Err(_) => gerr!("cannot access zmq context"),
    }
    Ok(())
}
