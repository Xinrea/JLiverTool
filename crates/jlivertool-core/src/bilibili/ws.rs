//! Bilibili WebSocket client for danmaku
//!
//! Protocol:
//! - Packet format: [packetLen(4)][headerLen(2)][ver(2)][op(4)][seq(4)][body]
//! - Header is always 16 bytes
//! - Ver 0: Normal JSON
//! - Ver 2: Deflate compressed
//! - Ver 3: Brotli compressed

use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::cell::RefCell;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};

// Thread-local buffer pool for decompression to reduce allocations
thread_local! {
    static DECOMPRESS_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(64 * 1024));
}

/// WebSocket connection info
#[derive(Debug, Clone)]
pub struct WsInfo {
    pub server: String,
    pub room_id: u64,
    pub uid: u64,
    pub token: String,
}

/// WebSocket message operation codes
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageOp {
    KeepAlive = 2,
    KeepAliveReply = 3,
    SendMsg = 4,
    SendMsgReply = 5,
    Auth = 7,
    AuthReply = 8,
}

impl TryFrom<u32> for MessageOp {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(Self::KeepAlive),
            3 => Ok(Self::KeepAliveReply),
            4 => Ok(Self::SendMsg),
            5 => Ok(Self::SendMsgReply),
            7 => Ok(Self::Auth),
            8 => Ok(Self::AuthReply),
            _ => Err(anyhow!("Unknown message op: {}", value)),
        }
    }
}

/// WebSocket body version
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsBodyVer {
    Normal = 0,
    Heartbeat = 1,
    Deflate = 2,
    Brotli = 3,
}

impl TryFrom<u16> for WsBodyVer {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::Heartbeat),
            2 => Ok(Self::Deflate),
            3 => Ok(Self::Brotli),
            _ => Err(anyhow!("Unknown body version: {}", value)),
        }
    }
}

/// Parsed WebSocket packet result
#[derive(Debug)]
pub struct PackResult {
    pub packet_len: u32,
    pub header_len: u16,
    pub ver: WsBodyVer,
    pub op: MessageOp,
    pub seq: u32,
    pub body: Vec<Value>,
}

/// WebSocket message builder
pub struct BiliWsMessage {
    buffer: Vec<u8>,
}

impl BiliWsMessage {
    /// Create a new message with operation and body
    pub fn new(op: MessageOp, body: &str) -> Self {
        let body_bytes = body.as_bytes();
        let packet_len = 16 + body_bytes.len();

        let mut buffer = vec![0u8; packet_len];

        // Write header
        Self::write_u32_be(&mut buffer, 0, packet_len as u32); // packet_len
        Self::write_u16_be(&mut buffer, 4, 16); // header_len
        Self::write_u16_be(&mut buffer, 6, 1); // ver
        Self::write_u32_be(&mut buffer, 8, op as u32); // op
        Self::write_u32_be(&mut buffer, 12, 1); // seq

        // Write body
        buffer[16..].copy_from_slice(body_bytes);

        Self { buffer }
    }

    /// Create from raw buffer
    pub fn from_buffer(buffer: Vec<u8>) -> Self {
        Self { buffer }
    }

    /// Get buffer as bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    /// Parse buffer into PackResult
    pub fn parse(&self) -> Result<PackResult> {
        if self.buffer.len() < 16 {
            return Err(anyhow!("Buffer too short"));
        }

        let packet_len = Self::read_u32_be(&self.buffer, 0);
        let header_len = Self::read_u16_be(&self.buffer, 4);
        let ver = WsBodyVer::try_from(Self::read_u16_be(&self.buffer, 6))?;
        let op = MessageOp::try_from(Self::read_u32_be(&self.buffer, 8))?;
        let seq = Self::read_u32_be(&self.buffer, 12);

        let mut body = Vec::new();

        match op {
            MessageOp::AuthReply => {
                debug!("Received auth reply");
            }
            MessageOp::KeepAliveReply => {
                debug!("Received keepalive reply");
                if self.buffer.len() >= 20 {
                    let count = Self::read_u32_be(&self.buffer, 16);
                    body.push(serde_json::json!({ "count": count }));
                }
            }
            MessageOp::SendMsgReply => {
                debug!(
                    "Received msg: length={}, ver={:?}",
                    packet_len - header_len as u32,
                    ver
                );

                match ver {
                    WsBodyVer::Normal => {
                        let data = &self.buffer[header_len as usize..packet_len as usize];
                        if let Ok(json) = serde_json::from_slice(data) {
                            body.push(json);
                        }
                    }
                    WsBodyVer::Deflate => {
                        let compressed = &self.buffer[header_len as usize..packet_len as usize];
                        if let Ok(decompressed) = Self::inflate_deflate(compressed) {
                            body = Self::parse_decompressed(&decompressed);
                        }
                    }
                    WsBodyVer::Brotli => {
                        let compressed = &self.buffer[header_len as usize..packet_len as usize];
                        if let Ok(decompressed) = Self::inflate_brotli(compressed) {
                            body = Self::parse_decompressed(&decompressed);
                        }
                    }
                    _ => {
                        warn!("Unknown message body version: {:?}", ver);
                    }
                }
            }
            _ => {
                warn!("Unknown message op: {:?}", op);
            }
        }

        Ok(PackResult {
            packet_len,
            header_len,
            ver,
            op,
            seq,
            body,
        })
    }

    /// Parse decompressed buffer into JSON bodies
    fn parse_decompressed(buffer: &[u8]) -> Vec<Value> {
        let mut bodies = Vec::new();
        let mut offset = 0;

        while offset < buffer.len() {
            if buffer.len() - offset < 16 {
                break;
            }

            let packet_len = Self::read_u32_be(buffer, offset) as usize;
            let header_len = 16;

            if offset + packet_len > buffer.len() {
                break;
            }

            let data = &buffer[offset + header_len..offset + packet_len];
            if let Ok(json) = serde_json::from_slice::<Value>(data) {
                bodies.push(json);
            }

            offset += packet_len;
        }

        bodies
    }

    /// Decompress deflate data using pooled buffer
    fn inflate_deflate(data: &[u8]) -> Result<Vec<u8>> {
        DECOMPRESS_BUFFER.with(|buf| {
            let mut buffer = buf.borrow_mut();
            buffer.clear();

            let mut decoder = flate2::read::ZlibDecoder::new(data);
            decoder.read_to_end(&mut buffer)?;

            // Return a copy of the decompressed data
            Ok(buffer.clone())
        })
    }

    /// Decompress brotli data using pooled buffer
    fn inflate_brotli(data: &[u8]) -> Result<Vec<u8>> {
        DECOMPRESS_BUFFER.with(|buf| {
            let mut buffer = buf.borrow_mut();
            buffer.clear();

            brotli::BrotliDecompress(&mut std::io::Cursor::new(data), &mut *buffer)?;

            // Return a copy of the decompressed data
            Ok(buffer.clone())
        })
    }

    fn read_u32_be(buf: &[u8], offset: usize) -> u32 {
        u32::from_be_bytes([
            buf[offset],
            buf[offset + 1],
            buf[offset + 2],
            buf[offset + 3],
        ])
    }

    fn read_u16_be(buf: &[u8], offset: usize) -> u16 {
        u16::from_be_bytes([buf[offset], buf[offset + 1]])
    }

    fn write_u32_be(buf: &mut [u8], offset: usize, value: u32) {
        let bytes = value.to_be_bytes();
        buf[offset..offset + 4].copy_from_slice(&bytes);
    }

    fn write_u16_be(buf: &mut [u8], offset: usize, value: u16) {
        let bytes = value.to_be_bytes();
        buf[offset..offset + 2].copy_from_slice(&bytes);
    }
}

/// WebSocket event
#[derive(Debug, Clone)]
pub enum WsEvent {
    /// Connection established
    Connected,
    /// Authentication successful
    Authenticated,
    /// Received message
    Message(Value),
    /// Heartbeat reply with viewer count
    HeartbeatReply(u32),
    /// Connection closed
    Disconnected,
    /// Error occurred
    Error(String),
}

/// Bilibili WebSocket client
pub struct BiliWebSocket {
    ws_info: WsInfo,
    event_tx: mpsc::UnboundedSender<WsEvent>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl BiliWebSocket {
    /// Create a new WebSocket client
    pub fn new(ws_info: WsInfo) -> (Self, mpsc::UnboundedReceiver<WsEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let client = Self {
            ws_info,
            event_tx,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };

        (client, event_rx)
    }

    /// Start the WebSocket connection
    pub async fn connect(&self) -> Result<()> {
        self.is_running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let url = &self.ws_info.server;
        info!("Connecting to WebSocket: {}", url);

        let (ws_stream, _) = connect_async(url).await?;
        let (mut write, mut read) = ws_stream.split();

        let _ = self.event_tx.send(WsEvent::Connected);

        // Send auth message
        let auth_info = serde_json::json!({
            "uid": self.ws_info.uid,
            "roomid": self.ws_info.room_id,
            "protover": 3,
            "type": 2,
            "platform": "web",
            "key": self.ws_info.token,
        });

        let auth_msg = BiliWsMessage::new(MessageOp::Auth, &auth_info.to_string());
        write.send(Message::Binary(auth_msg.into_bytes())).await?;

        // Start heartbeat task
        let heartbeat_running = self.is_running.clone();
        let mut heartbeat_write = write;

        let heartbeat_handle = tokio::spawn(async move {
            let mut heartbeat_interval = interval(Duration::from_secs(10));

            while heartbeat_running.load(std::sync::atomic::Ordering::SeqCst) {
                heartbeat_interval.tick().await;

                let heartbeat_msg = BiliWsMessage::new(MessageOp::KeepAlive, "");
                if heartbeat_write
                    .send(Message::Binary(heartbeat_msg.into_bytes()))
                    .await
                    .is_err()
                {
                    break;
                }
            }

            heartbeat_write
        });

        // Process incoming messages
        let event_tx = self.event_tx.clone();

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    let ws_msg = BiliWsMessage::from_buffer(data);
                    match ws_msg.parse() {
                        Ok(pack) => {
                            match pack.op {
                                MessageOp::AuthReply => {
                                    let _ = event_tx.send(WsEvent::Authenticated);
                                }
                                MessageOp::KeepAliveReply => {
                                    if let Some(count_json) = pack.body.first() {
                                        if let Some(count) =
                                            count_json.get("count").and_then(|v| v.as_u64())
                                        {
                                            let _ =
                                                event_tx.send(WsEvent::HeartbeatReply(count as u32));
                                        }
                                    }
                                }
                                MessageOp::SendMsgReply => {
                                    for body in pack.body {
                                        let _ = event_tx.send(WsEvent::Message(body));
                                    }
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse WebSocket message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket closed by server");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    let _ = event_tx.send(WsEvent::Error(e.to_string()));
                    break;
                }
                _ => {}
            }
        }

        // Cleanup
        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        let _ = self.event_tx.send(WsEvent::Disconnected);

        // Wait for heartbeat task to finish
        let _ = heartbeat_handle.await;

        Ok(())
    }

    /// Disconnect the WebSocket
    pub fn disconnect(&self) {
        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Managed WebSocket connection with auto-reconnect
pub struct ManagedBiliWebSocket {
    ws_info: WsInfo,
    event_tx: mpsc::UnboundedSender<WsEvent>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
    reconnect_delay: Duration,
}

impl ManagedBiliWebSocket {
    pub fn new(ws_info: WsInfo) -> (Self, mpsc::UnboundedReceiver<WsEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let client = Self {
            ws_info,
            event_tx,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            reconnect_delay: Duration::from_secs(5),
        };

        (client, event_rx)
    }

    /// Start connection with auto-reconnect
    pub async fn run(&self) {
        self.is_running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        while self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            let (ws, mut rx) = BiliWebSocket::new(self.ws_info.clone());

            // Forward events
            let event_tx = self.event_tx.clone();
            let forward_handle = tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    if event_tx.send(event).is_err() {
                        break;
                    }
                }
            });

            // Run connection
            if let Err(e) = ws.connect().await {
                error!("WebSocket connection error: {}", e);
                let _ = self.event_tx.send(WsEvent::Error(e.to_string()));
            }

            forward_handle.abort();

            // Check if we should reconnect
            if !self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            info!("Reconnecting in {:?}...", self.reconnect_delay);
            tokio::time::sleep(self.reconnect_delay).await;
        }
    }

    pub fn stop(&self) {
        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
