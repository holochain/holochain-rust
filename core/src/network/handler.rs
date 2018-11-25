use crate::context::Context;
use holochain_net_connection::{
    net_connection::{NetHandler},
    /*protocol_wrapper::{
        DhtData,
        ProtocolWrapper, TrackAppData,
    }*/
};
use std::sync::Arc;

pub fn create_handler(_c: &Arc<Context>) -> NetHandler {
    //let context = c.clone();
    Box::new(|_| {
        Ok(())
    })
}