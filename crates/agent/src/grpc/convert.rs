use std::time::{Duration, SystemTime};

use proto::control::v1::{
    Heartbeat, Hello, SessionRequest, SessionResponse, Welcome, disconnect::Reason as WireReason,
    session_request::Payload, session_response,
};

use super::error::GrpcError;
use super::types::{DisconnectReason, FromServer, HelloInfo, SessionInfo, ToServer};

const MAX_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3600);

pub(super) fn hello_to_wire(h: HelloInfo) -> SessionRequest {
    SessionRequest {
        payload: Some(Payload::Hello(Hello {
            protocol_version: h.protocol_version,
            agent_version: h.agent_version,
            hostname: h.hostname,
            capabilities: h.capabilities,
        })),
    }
}

pub(super) fn to_wire(msg: ToServer) -> SessionRequest {
    let payload = match msg {
        ToServer::Heartbeat { seq } => Payload::Heartbeat(Heartbeat {
            seq,
            sent_at: Some(SystemTime::now().into()),
        }),
    };
    SessionRequest {
        payload: Some(payload),
    }
}

pub(super) fn from_wire(msg: SessionResponse) -> Option<FromServer> {
    match msg.payload? {
        session_response::Payload::HeartbeatAck(a) => Some(FromServer::HeartbeatAck { seq: a.seq }),
        session_response::Payload::Disconnect(d) => Some(FromServer::Disconnect {
            reason: reason_from_wire(d.reason()),
            message: d.message,
        }),
        session_response::Payload::Welcome(_) => {
            tracing::warn!("Unexpected `Welcome` in active session, ignoring");
            None
        }
    }
}

pub(super) fn session_from_wire(w: Welcome) -> anyhow::Result<SessionInfo, GrpcError> {
    let interval = w
        .heartbeat_interval
        .ok_or_else(|| GrpcError::Handshake("`Welcome` without heartbeat_interval".into()))?;
    let interval = Duration::try_from(interval)
        .map_err(|_| GrpcError::Handshake("Negative heartbeat_interval".into()))?;

    if interval.is_zero() {
        return Err(GrpcError::Handshake("Zero heartbeat_interval".into()));
    }
    if interval > MAX_HEARTBEAT_INTERVAL {
        return Err(GrpcError::Handshake("Heartbeat_interval too large".into()));
    }
    if w.session_id.is_empty() {
        return Err(GrpcError::Handshake("Empty session_id".into()));
    }

    Ok(SessionInfo {
        session_id: w.session_id,
        heartbeat_interval: interval,
    })
}

pub(super) fn reason_from_wire(r: WireReason) -> DisconnectReason {
    match r {
        WireReason::VersionMismatch => DisconnectReason::VersionMismatch,
        WireReason::Replaced => DisconnectReason::Replaced,
        WireReason::Shutdown => DisconnectReason::Shutdown,
        WireReason::ProtocolViolation => DisconnectReason::ProtocolViolation,
        WireReason::Unspecified => DisconnectReason::Unknown,
    }
}
