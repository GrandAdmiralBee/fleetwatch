use crate::grpc::convert;

use super::error::GrpcError;
use super::types::{FromServer, HelloInfo, SessionInfo, ToServer};

use proto::control::v1::{
    SessionRequest, SessionResponse, control_service_client::ControlServiceClient,
    session_response::Payload,
};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Streaming;
use tonic::transport::{Channel, Endpoint};

use std::time::Duration;

#[derive(Debug)]
pub struct GrpcConfig {
    pub endpoint: String,
    pub connect_timeout: Duration,
    pub welcome_timeout: Duration,
}

#[derive(Debug)]
pub struct ControlConnection {
    // Keep `Option` so that we can close stream from our end
    // and still be able to recv server responses
    tx: Option<mpsc::Sender<SessionRequest>>,
    inbound: tonic::Streaming<SessionResponse>,
    session: SessionInfo,
    // Cheap `Channel` clone. Doesn't allow channel worker to
    // finish, while stream is active
    _channel: Channel,
}

impl ControlConnection {
    pub(super) fn new(
        tx: mpsc::Sender<SessionRequest>,
        inbound: Streaming<SessionResponse>,
        session: SessionInfo,
        channel: Channel,
    ) -> Self {
        Self {
            tx: Some(tx),
            inbound,
            session,
            _channel: channel,
        }
    }

    /// Session info recieved from `Welcome` msg
    pub fn session(&self) -> &SessionInfo {
        &self.session
    }

    /// Send msg to server
    /// if the queue is full -> backpressure
    pub async fn send(&self, msg: ToServer) -> anyhow::Result<(), GrpcError> {
        let tx = self.tx.as_ref().ok_or(GrpcError::Closed)?;
        tx.send(convert::to_wire(msg))
            .await
            .map_err(|e| GrpcError::Stream(format!("server side of the stream is gone: {}", &e)))
    }

    /// Next msg from server
    /// if `Ok(None)` is received - the server closed the stream
    pub async fn recv(&mut self) -> anyhow::Result<Option<FromServer>, GrpcError> {
        // Using loop to receive only approved by `from_wire` msg
        loop {
            let msg = match self.inbound.message().await {
                Ok(Some(m)) => m,
                Ok(None) => return Ok(None),
                Err(status) => return Err(GrpcError::Stream(status.to_string())),
            };
            if let Some(ev) = convert::from_wire(msg) {
                return Ok(Some(ev));
            }
        }
    }

    /// Close the agent side of channel.
    /// We can now read the stream till the end with `recv()`
    pub fn close(&mut self) {
        self.tx = None;
    }
}

pub async fn connect(
    cfg: &GrpcConfig,
    hello: HelloInfo,
) -> anyhow::Result<ControlConnection, GrpcError> {
    let channel = open_channel(cfg).await?;
    let (tx, mut inbound) = open_stream(channel.clone(), hello).await?;
    let session = await_welcome(&mut inbound, cfg.welcome_timeout).await?;
    Ok(ControlConnection::new(tx, inbound, session, channel))
}

async fn open_channel(cfg: &GrpcConfig) -> anyhow::Result<Channel, GrpcError> {
    Endpoint::from_shared(cfg.endpoint.clone())
        .map_err(|e| GrpcError::Transport(e.to_string()))?
        .connect_timeout(cfg.connect_timeout)
        .http2_keep_alive_interval(Duration::from_secs(20))
        .keep_alive_while_idle(true)
        .connect()
        .await
        .map_err(|e| GrpcError::Transport(e.to_string()))
}

async fn open_stream(
    channel: Channel,
    hello: HelloInfo,
) -> anyhow::Result<(mpsc::Sender<SessionRequest>, Streaming<SessionResponse>), GrpcError> {
    let (tx, rx) = mpsc::channel(32);

    tx.send(convert::hello_to_wire(hello))
        .await
        .map_err(|e| GrpcError::Handshake(e.to_string()))?;

    let response = ControlServiceClient::new(channel)
        .open_session(ReceiverStream::new(rx))
        .await
        .map_err(|e| GrpcError::Handshake(e.to_string()))?;
    Ok((tx, response.into_inner()))
}

async fn await_welcome(
    inbound: &mut Streaming<SessionResponse>,
    wait: Duration,
) -> anyhow::Result<SessionInfo, GrpcError> {
    let first = timeout(wait, inbound.message())
        .await
        .map_err(|_| GrpcError::Handshake("welcome timeout".into()))?
        .map_err(|s| GrpcError::Handshake(s.to_string()))?
        .ok_or_else(|| GrpcError::Handshake("stream closed before Welcome".into()))?;

    match first.payload {
        Some(Payload::Welcome(w)) => convert::session_from_wire(w),
        Some(Payload::Disconnect(d)) => {
            Err(GrpcError::Rejected(convert::reason_from_wire(d.reason())))
        }
        _ => Err(GrpcError::Handshake("expected Welcome".into())),
    }
}
