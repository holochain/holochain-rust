//! This module holds message type u8 constants.

/// client initiated heartbeat
pub const MSG_PING: u8 = 0x10;

/// response to client initiated heartbeat
pub const MSG_PONG: u8 = 0x11;

/// send a message to either side, await a response
pub const MSG_CALL: u8 = 0x20;

/// success response to a call
pub const MSG_CALL_OK: u8 = 0x21;

/// failure response to a call
pub const MSG_CALL_FAIL: u8 = 0x22;
