use crate::key_bundle;
use holochain_core_types::error::HcResult;
use holochain_sodium::{aead, kx, pwhash, secbuf::SecBuf};

//--------------------------------------------------------------------------------------------------
// SecBuf improvements
// TODO move this to sodium crate
//--------------------------------------------------------------------------------------------------

/// Load the Vec<u8> into the SecBuf
pub(crate) fn vec_to_secbuf(data: &Vec<u8>, buf: &mut SecBuf) {
    assert_eq!(data.len(), buf.len());
    let mut buf = buf.write_lock();
    for x in 0..data.len() {
        buf[x] = data[x];
    }
}

/// Load the [u8] into the SecBuf
pub(crate) fn array_to_secbuf(data: &[u8], buf: &mut SecBuf) {
    assert_eq!(data.len(), buf.len());
    let mut buf = buf.write_lock();
    for x in 0..data.len() {
        buf[x] = data[x];
    }
}


/// Check if the buffer is empty i.e. [0,0,0,0,0,0,0,0]
pub(crate) fn is_secbuf_empty(buf: &mut SecBuf) -> bool {
    let buf = buf.read_lock();
    println!("Buf{:?}", *buf);
    for i in 0..buf.len() {
        if buf[i] != 0 {
            return true;
        }
    }
    return false;
}