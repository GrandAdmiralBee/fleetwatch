use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HelloInfo {
    pub protocol_version: u32,
    pub agent_version: String,
    pub hostname: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub heartbeat_interval: Duration,
}

#[derive(Debug, Clone)]
pub enum ToServer {
    Heartbeat { seq: u64 },
}

#[derive(Debug, Clone)]
pub enum FromServer {
    HeartbeatAck {
        seq: u64,
    },
    Disconnect {
        reason: DisconnectReason,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DisconnectReason {
    VersionMismatch,
    Replaced,
    Shutdown,
    ProtocolViolation,
    Unknown,
}
