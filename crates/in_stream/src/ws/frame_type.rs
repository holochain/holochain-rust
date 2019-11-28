/// a close message, with a websocket close code
/// https://www.iana.org/assignments/websocket/websocket.xml#close-code-number
#[derive(Debug, Clone)]
pub struct CloseData {
    pub code: u16,
    pub reason: String,
}

/// enumerates the different websocket message types
#[derive(Debug, Clone)]
pub enum WsFrame {
    /// utf8 text
    Text(String),
    /// a string of bytes
    Binary(Vec<u8>),
    /// a ping message, content must be < 125 bytes
    Ping(Vec<u8>),
    /// a pong message, content must be < 125 bytes
    Pong(Vec<u8>),
    /// a close message, with a websocket close code
    /// https://www.iana.org/assignments/websocket/websocket.xml#close-code-number
    Close(CloseData),
}

impl From<tungstenite::protocol::Message> for WsFrame {
    fn from(m: tungstenite::protocol::Message) -> Self {
        match m {
            tungstenite::protocol::Message::Text(s) => WsFrame::Text(s),
            tungstenite::protocol::Message::Binary(b) => WsFrame::Binary(b),
            tungstenite::protocol::Message::Ping(b) => WsFrame::Ping(b),
            tungstenite::protocol::Message::Pong(b) => WsFrame::Pong(b),
            tungstenite::protocol::Message::Close(c) => {
                match c {
                    None => WsFrame::Close(CloseData {
                        code: 1005, // no status received
                        reason: String::new(),
                    }),
                    Some(c) => WsFrame::Close(CloseData {
                        code: c.code.into(),
                        reason: c.reason.to_string(),
                    }),
                }
            }
        }
    }
}

impl From<WsFrame> for tungstenite::protocol::Message {
    fn from(m: WsFrame) -> Self {
        match m {
            WsFrame::Text(s) => tungstenite::protocol::Message::Text(s),
            WsFrame::Binary(b) => tungstenite::protocol::Message::Binary(b),
            WsFrame::Ping(b) => tungstenite::protocol::Message::Ping(b),
            WsFrame::Pong(b) => tungstenite::protocol::Message::Pong(b),
            WsFrame::Close(c) => tungstenite::protocol::Message::Close(
                Some(tungstenite::protocol::CloseFrame {
                    code: c.code.into(),
                    reason: std::borrow::Cow::from(&c.reason),
                }.into_owned())
            ),
        }
    }
}

impl WsFrame {
    pub fn assume<O: Into<Self>>(&mut self, oth: O) {
        *self = oth.into();
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            WsFrame::Text(s) => s.as_bytes(),
            WsFrame::Binary(b) => b,
            WsFrame::Ping(b) => b,
            WsFrame::Pong(b) => b,
            WsFrame::Close(c) => c.reason.as_bytes(),
        }
    }

    pub fn as_str(&self) -> std::borrow::Cow<'_, str> {
        match self {
            WsFrame::Text(s) => std::borrow::Cow::from(s),
            WsFrame::Binary(b) => String::from_utf8_lossy(b),
            WsFrame::Ping(b) => String::from_utf8_lossy(b),
            WsFrame::Pong(b) => String::from_utf8_lossy(b),
            WsFrame::Close(c) => std::borrow::Cow::from(&c.reason),
        }
    }

    /// true if frame type is WsFrame::Text
    pub fn is_text(&self) -> bool {
        if let WsFrame::Text(_) = self {
            true
        } else {
            false
        }
    }

    /// true if frame type is WsFrame::Binary
    pub fn is_binary(&self) -> bool {
        if let WsFrame::Binary(_) = self {
            true
        } else {
            false
        }
    }

    /// true if frame type is WsFrame::Ping
    pub fn is_ping(&self) -> bool {
        if let WsFrame::Ping(_) = self {
            true
        } else {
            false
        }
    }

    /// true if frame type is WsFrame::Pong
    pub fn is_pong(&self) -> bool {
        if let WsFrame::Pong(_) = self {
            true
        } else {
            false
        }
    }

    /// true if frame type is WsFrame::Close
    pub fn is_close(&self) -> bool {
        if let WsFrame::Close(_) = self {
            true
        } else {
            false
        }
    }
}
