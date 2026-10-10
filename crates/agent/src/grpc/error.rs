use super::types::DisconnectReason;

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("transport: {0}")]
    Transport(String),
    #[error("handshake: {0}")]
    Handshake(String),
    #[error("rejected by server: {0:?}")]
    Rejected(DisconnectReason),
    #[error("stream: {0}")]
    Stream(String),
    #[error("connection closed locally")]
    Closed,
}
