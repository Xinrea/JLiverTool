//! TTS Manager - orchestrates TTS functionality

use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc;
use tracing::{error, info};

use super::queue::TtsQueue;

/// TTS message type with priority
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TtsMessageType {
    /// Danmu message (lowest priority)
    Danmu,
    /// Gift message (medium priority)
    Gift,
    /// SuperChat message (highest priority)
    SuperChat,
}

impl TtsMessageType {
    /// Get the priority value (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            TtsMessageType::Danmu => 1,
            TtsMessageType::Gift => 2,
            TtsMessageType::SuperChat => 3,
        }
    }
}

/// TTS message to be spoken
#[derive(Debug, Clone)]
pub struct TtsMessage {
    pub message_type: TtsMessageType,
    pub text: String,
}

impl TtsMessage {
    /// Create a new TTS message
    pub fn new(message_type: TtsMessageType, text: String) -> Self {
        Self { message_type, text }
    }

    /// Create a danmu TTS message
    pub fn danmu(username: &str, content: &str) -> Self {
        Self::new(
            TtsMessageType::Danmu,
            format!("{} 说: {}", username, content),
        )
    }

    /// Create a gift TTS message
    pub fn gift(username: &str, gift_name: &str, count: u32) -> Self {
        let text = if count > 1 {
            format!("感谢 {} 赠送的 {} 个 {}", username, count, gift_name)
        } else {
            format!("感谢 {} 赠送的 {}", username, gift_name)
        };
        Self::new(TtsMessageType::Gift, text)
    }

    /// Create a superchat TTS message
    pub fn superchat(username: &str, price: u64, content: &str) -> Self {
        Self::new(
            TtsMessageType::SuperChat,
            format!("{} 发送了 {} 元醒目留言: {}", username, price, content),
        )
    }

    /// Get the priority of this message
    pub fn priority(&self) -> u8 {
        self.message_type.priority()
    }
}

/// TTS enabled flags
#[derive(Debug, Clone, Default)]
pub struct TtsEnabled {
    pub danmu: bool,
    pub gift: bool,
    pub superchat: bool,
}

/// Internal command for the TTS worker thread
enum TtsCommand {
    Speak(TtsMessage),
    SetVolume(f32),
    Stop,
    Shutdown,
}

/// TTS Manager - main interface for TTS functionality
pub struct TtsManager {
    command_tx: mpsc::UnboundedSender<TtsCommand>,
    enabled: Arc<Mutex<TtsEnabled>>,
    volume: Arc<Mutex<f32>>,
}

impl TtsManager {
    /// Create a new TTS manager
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let enabled = Arc::new(Mutex::new(TtsEnabled::default()));
        let volume = Arc::new(Mutex::new(1.0f32));

        // Spawn worker thread
        let volume_clone = volume.clone();
        thread::spawn(move || {
            Self::worker_thread(command_rx, volume_clone);
        });

        Self {
            command_tx,
            enabled,
            volume,
        }
    }

    /// Worker thread that processes TTS commands
    fn worker_thread(
        mut command_rx: mpsc::UnboundedReceiver<TtsCommand>,
        volume: Arc<Mutex<f32>>,
    ) {
        let mut queue = TtsQueue::new();

        #[cfg(target_os = "macos")]
        let synthesizer = super::system_macos::SystemTts::new();

        #[cfg(not(target_os = "macos"))]
        let synthesizer: Option<()> = None;

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create TTS runtime");

        runtime.block_on(async {
            loop {
                // Try to receive a command with a short timeout
                let cmd = if queue.is_empty() {
                    // If queue is empty, wait for a command
                    command_rx.recv().await
                } else {
                    // If queue has items, try to receive without blocking
                    match command_rx.try_recv() {
                        Ok(cmd) => Some(cmd),
                        Err(_) => None,
                    }
                };

                if let Some(cmd) = cmd {
                    match cmd {
                        TtsCommand::Speak(message) => {
                            queue.push(message);
                        }
                        TtsCommand::SetVolume(v) => {
                            *volume.lock() = v;
                        }
                        TtsCommand::Stop => {
                            queue.clear();
                            #[cfg(target_os = "macos")]
                            if let Some(ref synth) = synthesizer {
                                synth.stop();
                            }
                        }
                        TtsCommand::Shutdown => {
                            info!("TTS worker shutting down");
                            break;
                        }
                    }
                }

                // Process queue
                if let Some(message) = queue.pop() {
                    let current_volume = *volume.lock();

                    #[cfg(target_os = "macos")]
                    if let Some(ref synth) = synthesizer {
                        synth.speak(&message.text, current_volume);
                    }

                    #[cfg(not(target_os = "macos"))]
                    {
                        let _ = current_volume;
                        info!("TTS (not implemented): {}", message.text);
                    }
                }
            }
        });
    }

    /// Queue a message for TTS
    pub fn speak(&self, message: TtsMessage) {
        // Check if this message type is enabled
        let enabled = self.enabled.lock();
        let should_speak = match message.message_type {
            TtsMessageType::Danmu => enabled.danmu,
            TtsMessageType::Gift => enabled.gift,
            TtsMessageType::SuperChat => enabled.superchat,
        };
        drop(enabled);

        if should_speak {
            if let Err(e) = self.command_tx.send(TtsCommand::Speak(message)) {
                error!("Failed to send TTS command: {}", e);
            }
        }
    }

    /// Set which message types are enabled for TTS
    pub fn set_enabled(&self, enabled: TtsEnabled) {
        *self.enabled.lock() = enabled;
    }

    /// Set the TTS volume (0.0 - 1.0)
    pub fn set_volume(&self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        *self.volume.lock() = volume;
        let _ = self.command_tx.send(TtsCommand::SetVolume(volume));
    }

    /// Stop current speech and clear the queue
    pub fn stop(&self) {
        let _ = self.command_tx.send(TtsCommand::Stop);
    }

    /// Speak a test message
    pub fn test(&self) {
        let _ = self
            .command_tx
            .send(TtsCommand::Speak(TtsMessage::new(
                TtsMessageType::Danmu,
                "这是一条测试语音".to_string(),
            )));
    }
}

impl Default for TtsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TtsManager {
    fn drop(&mut self) {
        let _ = self.command_tx.send(TtsCommand::Shutdown);
    }
}
