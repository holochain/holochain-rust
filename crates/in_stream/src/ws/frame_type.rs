/// enumerates the different websocket message types
#[derive(Debug, Clone)]
pub enum WsFrameType {
    /// utf8 text
    Text,
    /// a string of bytes
    Binary,
    /// a ping message, content must be < 125 bytes
    Ping,
    /// a pong message, content must be < 125 bytes
    Pong,
    /// a close message, with a websocket close code
    /// https://www.iana.org/assignments/websocket/websocket.xml#close-code-number
    Close { code: u16 },
}

/// represents a single websocket message frame
/// contains binary content and a type indicator
/// in the case of a Close frame, there is also a close code
#[derive(Shrinkwrap, Debug, Clone)]
#[shrinkwrap(mutable)]
pub struct WsFrame {
    /// The raw content of this frame. If this is a Text frame
    /// please use `as_str()` or `to_string()` to access the utf8 content.
    #[shrinkwrap(main_field)]
    pub content: Vec<u8>,
    /// the frame type indicator
    pub frame_type: WsFrameType,
}

impl From<Vec<u8>> for WsFrame {
    fn from(content: Vec<u8>) -> Self {
        Self {
            content,
            frame_type: WsFrameType::Binary,
        }
    }
}

impl From<String> for WsFrame {
    fn from(content: String) -> Self {
        Self {
            content: content.into_bytes(),
            frame_type: WsFrameType::Text,
        }
    }
}

impl From<WsFrame> for Vec<u8> {
    fn from(frame: WsFrame) -> Self {
        frame.into_vec()
    }
}

impl From<WsFrame> for String {
    fn from(frame: WsFrame) -> Self {
        frame.as_str().to_string()
    }
}

impl std::fmt::Display for WsFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl WsFrame {
    /// create a new WsFrame from binary content
    pub fn new(content: Vec<u8>, frame_type: WsFrameType) -> Self {
        Self {
            content,
            frame_type,
        }
    }

    /// create a new WsFrame from utf8 string content
    pub fn with_str(content: &str, frame_type: WsFrameType) -> Self {
        Self {
            content: content.as_bytes().to_vec(),
            frame_type,
        }
    }

    /// access the raw binary content
    pub fn as_bytes(&self) -> &[u8] {
        &self.content
    }

    /// access the utf8 string content
    pub fn as_str(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.content)
    }

    /// accent the frame type indicator
    pub fn frame_type(&self) -> &WsFrameType {
        &self.frame_type
    }

    /// extract the raw binary frame content without cloning
    pub fn into_vec(self) -> Vec<u8> {
        self.content
    }

    /// true if frame type is WsFrameType::Text
    pub fn is_text(&self) -> bool {
        if let WsFrameType::Text = self.frame_type {
            true
        } else {
            false
        }
    }

    /// true if frame type is WsFrameType::Binary
    pub fn is_binary(&self) -> bool {
        if let WsFrameType::Binary = self.frame_type {
            true
        } else {
            false
        }
    }

    /// true if frame type is WsFrameType::Ping
    pub fn is_ping(&self) -> bool {
        if let WsFrameType::Ping = self.frame_type {
            true
        } else {
            false
        }
    }

    /// true if frame type is WsFrameType::Pong
    pub fn is_pong(&self) -> bool {
        if let WsFrameType::Pong = self.frame_type {
            true
        } else {
            false
        }
    }

    /// true if frame type is WsFrameType::Close
    pub fn is_close(&self) -> bool {
        if let WsFrameType::Close { code: _ } = self.frame_type {
            true
        } else {
            false
        }
    }
}
