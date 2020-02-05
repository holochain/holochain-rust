#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugData {
    pub limbo: DebugLimboData,
    pub msg_queue_size: usize,
    pub wss_queue_size: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugLimboData {
    pub total_connections: usize,
    pub total_messages: usize,
    pub max_messages: usize,
}
