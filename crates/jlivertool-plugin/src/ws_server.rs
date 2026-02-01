//! WebSocket server for plugin communication
//!
//! Provides a local WebSocket server that plugins can connect to for:
//! - Receiving events (danmu, gifts, etc.)
//! - Calling APIs (user info, room info, etc.)

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::events::PluginEvent;

/// WebSocket message from server to client
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum WsServerMessage {
    /// Welcome message with server info
    Welcome { port: u16 },
    /// Event broadcast - contains the event in the "Event" field
    Event { #[serde(rename = "Event")] event: PluginEvent },
    /// Response to a request
    Response { id: String, data: serde_json::Value },
    /// Error response
    Error { id: Option<String>, message: String },
}

/// WebSocket message from client to server
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum WsClientMessage {
    /// Subscribe to specific event channels
    Subscribe { channels: Vec<String> },
    /// Unsubscribe from event channels
    Unsubscribe { channels: Vec<String> },
    /// API request
    Request {
        id: String,
        method: String,
        params: serde_json::Value,
    },
}

/// Client connection state
struct ClientState {
    subscribed_channels: Vec<String>,
}

/// Plugin WebSocket server
pub struct PluginWsServer {
    port: u16,
    event_tx: broadcast::Sender<PluginEvent>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl PluginWsServer {
    /// Create a new plugin WebSocket server
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            port: 0,
            event_tx,
            shutdown_tx: None,
        }
    }

    /// Get the port the server is listening on
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the WebSocket URL for plugins to connect to
    pub fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }

    /// Broadcast an event to all connected plugins
    pub fn broadcast_event(&self, event: PluginEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Get a sender for broadcasting events
    pub fn event_sender(&self) -> broadcast::Sender<PluginEvent> {
        self.event_tx.clone()
    }

    /// Start the WebSocket server
    pub async fn start(&mut self) -> Result<()> {
        // Bind to a random available port on localhost
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        self.port = addr.port();

        log::info!("Plugin WebSocket server listening on {}", addr);

        let event_tx = self.event_tx.clone();
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Spawn the server task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                let event_rx = event_tx.subscribe();
                                tokio::spawn(handle_connection(stream, addr, event_rx));
                            }
                            Err(e) => {
                                log::error!("Failed to accept connection: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        log::info!("Plugin WebSocket server shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the WebSocket server
    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
    }
}

impl Default for PluginWsServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle a single WebSocket connection
async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    mut event_rx: broadcast::Receiver<PluginEvent>,
) {
    log::info!("New plugin connection from {}", addr);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("WebSocket handshake failed for {}: {}", addr, e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Client state
    let state = Arc::new(RwLock::new(ClientState {
        subscribed_channels: vec!["*".to_string()], // Subscribe to all by default
    }));

    // Send welcome message
    let welcome = WsServerMessage::Welcome { port: addr.port() };
    if let Ok(msg) = serde_json::to_string(&welcome) {
        let _ = ws_sender.send(Message::Text(msg)).await;
    }

    // Create a channel for sending messages to the client
    let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<String>(100);

    // Spawn task to forward events to client
    let state_clone = state.clone();
    let outgoing_tx_clone = outgoing_tx.clone();
    let event_forward_task = tokio::spawn(async move {
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    let state = state_clone.read().await;
                    let channel = event.channel();

                    // Check if client is subscribed to this channel
                    let subscribed = state.subscribed_channels.iter().any(|c| {
                        c == "*" || c.eq_ignore_ascii_case(channel)
                    });

                    if subscribed {
                        let msg = WsServerMessage::Event { event };
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if outgoing_tx_clone.send(json).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    log::warn!("Client {} lagged by {} events", addr, n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    // Spawn task to send outgoing messages
    let send_task = tokio::spawn(async move {
        while let Some(msg) = outgoing_rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Create another sender for responses
    let outgoing_tx_for_handler = outgoing_tx.clone();

    // Handle incoming messages from client
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<WsClientMessage>(&text) {
                    Ok(client_msg) => {
                        handle_client_message(client_msg, &state, &outgoing_tx_for_handler).await;
                    }
                    Err(e) => {
                        let error = WsServerMessage::Error {
                            id: None,
                            message: format!("Invalid message format: {}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&error) {
                            let _ = outgoing_tx_for_handler.send(json).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                log::info!("Client {} disconnected", addr);
                break;
            }
            Ok(Message::Ping(_data)) => {
                // Pong is handled automatically by tungstenite
            }
            Err(e) => {
                log::error!("WebSocket error from {}: {}", addr, e);
                break;
            }
            _ => {}
        }
    }

    event_forward_task.abort();
    send_task.abort();
    log::info!("Connection closed for {}", addr);
}

/// Handle a client message
async fn handle_client_message(
    msg: WsClientMessage,
    state: &Arc<RwLock<ClientState>>,
    outgoing_tx: &mpsc::Sender<String>,
) {
    match msg {
        WsClientMessage::Subscribe { channels } => {
            let mut state = state.write().await;
            for channel in channels {
                if !state.subscribed_channels.contains(&channel) {
                    state.subscribed_channels.push(channel);
                }
            }
            log::debug!("Client subscribed to: {:?}", state.subscribed_channels);
        }
        WsClientMessage::Unsubscribe { channels } => {
            let mut state = state.write().await;
            state.subscribed_channels.retain(|c| !channels.contains(c));
            log::debug!("Client unsubscribed, remaining: {:?}", state.subscribed_channels);
        }
        WsClientMessage::Request { id, method, params } => {
            let response = handle_api_request(&method, params).await;
            let msg = match response {
                Ok(data) => WsServerMessage::Response { id, data },
                Err(e) => WsServerMessage::Error {
                    id: Some(id),
                    message: e.to_string(),
                },
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = outgoing_tx.send(json).await;
            }
        }
    }
}

/// Handle an API request from a plugin
async fn handle_api_request(method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
    match method {
        "openUrl" => {
            let url = params
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing url parameter"))?;
            open::that(url)?;
            Ok(serde_json::json!({"success": true}))
        }
        "getServerInfo" => {
            Ok(serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "name": "JLiverTool Plugin Server"
            }))
        }
        _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
    }
}
