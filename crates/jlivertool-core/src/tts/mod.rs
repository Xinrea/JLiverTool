//! TTS (Text-to-Speech) module for JLiverTool
//!
//! Provides text-to-speech functionality using platform-native APIs.

mod manager;
mod queue;

#[cfg(target_os = "macos")]
mod system_macos;

#[cfg(target_os = "windows")]
mod system_windows;

pub use manager::{TtsEnabled, TtsManager, TtsMessage, TtsMessageType};
pub use queue::TtsQueue;
