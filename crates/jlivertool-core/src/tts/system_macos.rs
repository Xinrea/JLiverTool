//! macOS native TTS using AVSpeechSynthesizer

use objc::runtime::{Class, Object, BOOL, YES};
use objc::{msg_send, sel, sel_impl};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, error};

/// macOS system TTS using AVSpeechSynthesizer
pub struct SystemTts {
    synthesizer: *mut Object,
    is_speaking: Arc<AtomicBool>,
}

// Safety: AVSpeechSynthesizer is thread-safe for our usage pattern
unsafe impl Send for SystemTts {}
unsafe impl Sync for SystemTts {}

impl SystemTts {
    /// Create a new SystemTts instance
    pub fn new() -> Option<Self> {
        unsafe {
            let cls = Class::get("AVSpeechSynthesizer")?;
            let synthesizer: *mut Object = msg_send![cls, alloc];
            let synthesizer: *mut Object = msg_send![synthesizer, init];

            if synthesizer.is_null() {
                error!("Failed to create AVSpeechSynthesizer");
                return None;
            }

            debug!("AVSpeechSynthesizer created successfully");

            Some(Self {
                synthesizer,
                is_speaking: Arc::new(AtomicBool::new(false)),
            })
        }
    }

    /// Speak the given text with the specified volume
    pub fn speak(&self, text: &str, volume: f32) {
        unsafe {
            // Create NSString from text
            let ns_string_cls = match Class::get("NSString") {
                Some(cls) => cls,
                None => {
                    error!("Failed to get NSString class");
                    return;
                }
            };

            let text_bytes = text.as_bytes();
            let ns_string: *mut Object = msg_send![ns_string_cls, alloc];
            let ns_string: *mut Object = msg_send![ns_string,
                initWithBytes: text_bytes.as_ptr() as *const c_void
                length: text_bytes.len()
                encoding: 4u64 // NSUTF8StringEncoding
            ];

            if ns_string.is_null() {
                error!("Failed to create NSString");
                return;
            }

            // Create AVSpeechUtterance
            let utterance_cls = match Class::get("AVSpeechUtterance") {
                Some(cls) => cls,
                None => {
                    let _: () = msg_send![ns_string, release];
                    error!("Failed to get AVSpeechUtterance class");
                    return;
                }
            };

            let utterance: *mut Object = msg_send![utterance_cls, speechUtteranceWithString: ns_string];

            if utterance.is_null() {
                let _: () = msg_send![ns_string, release];
                error!("Failed to create AVSpeechUtterance");
                return;
            }

            // Set volume (0.0 - 1.0)
            let _: () = msg_send![utterance, setVolume: volume];

            // Set rate (0.0 - 1.0, default is ~0.5)
            let _: () = msg_send![utterance, setRate: 0.5f32];

            // Set voice to Chinese (zh-CN)
            if let Some(voice_cls) = Class::get("AVSpeechSynthesisVoice") {
                let lang_str: *mut Object = msg_send![ns_string_cls, stringWithUTF8String: b"zh-CN\0".as_ptr()];
                let voice: *mut Object = msg_send![voice_cls, voiceWithLanguage: lang_str];
                if !voice.is_null() {
                    let _: () = msg_send![utterance, setVoice: voice];
                }
            }

            // Speak
            self.is_speaking.store(true, Ordering::SeqCst);
            let _: () = msg_send![self.synthesizer, speakUtterance: utterance];

            // Wait for speech to complete
            loop {
                let speaking: BOOL = msg_send![self.synthesizer, isSpeaking];
                if speaking != YES {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            self.is_speaking.store(false, Ordering::SeqCst);

            // Release NSString
            let _: () = msg_send![ns_string, release];
        }
    }

    /// Stop current speech
    pub fn stop(&self) {
        unsafe {
            // AVSpeechBoundaryImmediate = 0
            let _: BOOL = msg_send![self.synthesizer, stopSpeakingAtBoundary: 0u64];
        }
        self.is_speaking.store(false, Ordering::SeqCst);
    }

    /// Check if currently speaking
    pub fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::SeqCst)
    }
}

impl Drop for SystemTts {
    fn drop(&mut self) {
        unsafe {
            // Stop any ongoing speech
            self.stop();
            // Release the synthesizer
            let _: () = msg_send![self.synthesizer, release];
        }
    }
}
