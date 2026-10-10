use std::time::Duration;

use proto::control::v1::{
    ConnectRequest, ConnectResponse, Disconnect, Hello, Welcome, connect_response::Payload,
    disconnect::Reason as WireReason,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DisconnectReason {
    VersionMismatch,
    Replaced,
    Shutdown,
    ProtocolViolation,
    Unknown,
}

impl From<WireReason> for DisconnectReason {
    fn from(r: WireReason) -> Self {
        match r {
            WireReason::VersionMismatch => Self::VersionMismatch,
            WireReason::Replaced => Self::Replaced,
            WireReason::Shutdown => Self::Shutdown,
            WireReason::ProtocolViolation => Self::ProtocolViolation,
            WireReason::Unspecified => Self::Unknown,
        }
    }
}

pub struct GrpcConfig {
    pub endpoint: String,
    pub connect_timeout: Duration,
    pub welcome_timeout: Duration,
}

pub struct HelloInfo {
    pub protocol_version: u32,
    pub agent_version: String,
    pub hostname: String,
    pub capabilities: Vec<String>,
}

pub struct SessionInfo {
    pub session_id: String,
    pub heartbeat_interval: Duration,
}

pub enum ToServer {
    Heartbeat { seq: u64 },
}

pub enum FromServer {
    HeartbeatAck {
        seq: u64,
    },
    Disconnect {
        reason: DisconnectReason,
        message: String,
    },
}

pub enum GrpcError {
    Transport(String),
    Handshake(String),
    Rejected(DisconnectReason),
    Stream(String),
}

pub async fn connect(_cfg: &GrpcConfig, _hello: HelloInfo) -> Result<ControlConnection, GrpcError> {
    todo!()
}

struct ControlConnection {}

impl ControlConnection {
    pub fn session(&self) -> &SessionInfo {
        todo!()
    }
    pub async fn send(&self, _msg: ToServer) -> Result<(), GrpcError> {
        todo!()
    }
    pub async fn recv(&mut self) -> Result<Option<FromServer>, GrpcError> {
        todo!()
    }
    pub fn close(&mut self) {
        todo!()
    }
}
