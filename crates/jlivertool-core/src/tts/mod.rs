//! TTS (Text-to-Speech) module for JLiverTool
//!
//! Provides text-to-speech functionality using macOS native AVSpeechSynthesizer.

mod manager;
mod queue;

#[cfg(target_os = "macos")]
mod system_macos;

pub use manager::{TtsEnabled, TtsManager, TtsMessage, TtsMessageType};
pub use queue::TtsQueue;
