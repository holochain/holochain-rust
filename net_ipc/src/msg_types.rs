//! This module holds message type u8 constants.

// -- client initiated messages -- //

// pub const MSG_CLI_RES_AUTH_1: u8 = 0x00;
// pub const MSG_CLI_RES_AUTH_2: u8 = 0x01;

/// client-initiated heartbeat
pub const MSG_CLI_PING: u8 = 0x02;

/// send a fire-and-foget message
pub const MSG_CLI_SEND: u8 = 0x03;

/// send an rpc-style message, expecting a response
pub const MSG_CLI_CALL: u8 = 0x04;

/// send a responce to a `call` request that another node made of us
pub const MSG_CLI_CALL_RESP: u8 = 0x05;

// pub const MSG_SRV_RES_AUTH_1: u8 = 0x00;
// pub const MSG_SRV_RES_AUTH_2: u8 = 0x01;

/// server response to a client ping request
pub const MSG_SRV_PONG: u8 = 0x02;

/// indicates the p2p client was able to transmit a `send` or `call` message
pub const MSG_SRV_RESP_OK: u8 = 0x03;

/// indicates the p2p client was NOT able to transmit a `send` or `call` message
pub const MSG_SRV_RESP_FAIL: u8 = 0x04;

/// we received a `send` from another node
pub const MSG_SRV_RECV_SEND: u8 = 0x05;

/// we received a `call` from another node
pub const MSG_SRV_RECV_CALL: u8 = 0x06;

/// we received a `call_resp` from another node
pub const MSG_SRV_RECV_CALL_RESP: u8 = 0x07;
