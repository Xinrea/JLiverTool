//! Windows native TTS using SAPI (Speech API)

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, error, warn};
use windows::core::BSTR;
use windows::Win32::Media::Speech::{ISpVoice, SpVoice, SPF_ASYNC, SPF_PURGEBEFORESPEAK};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};

/// Windows system TTS using SAPI
pub struct SystemTts {
    voice: ISpVoice,
    is_speaking: Arc<AtomicBool>,
    com_initialized: bool,
}

// Safety: ISpVoice is thread-safe when COM is properly initialized per-thread
unsafe impl Send for SystemTts {}
unsafe impl Sync for SystemTts {}

impl SystemTts {
    /// Create a new SystemTts instance
    pub fn new() -> Option<Self> {
        unsafe {
            // Initialize COM for this thread
            let com_result = CoInitializeEx(None, COINIT_MULTITHREADED);
            let com_initialized = com_result.is_ok();

            if !com_initialized {
                // COM might already be initialized, which is fine
                debug!("COM initialization returned: {:?}", com_result);
            }

            // Create SpVoice instance
            let voice: ISpVoice = match CoCreateInstance(&SpVoice, None, CLSCTX_ALL) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to create SpVoice: {:?}", e);
                    if com_initialized {
                        CoUninitialize();
                    }
                    return None;
                }
            };

            debug!("SpVoice created successfully");

            Some(Self {
                voice,
                is_speaking: Arc::new(AtomicBool::new(false)),
                com_initialized,
            })
        }
    }

    /// Speak the given text with the specified volume
    pub fn speak(&self, text: &str, volume: f32) {
        unsafe {
            // Set volume (SAPI uses 0-100 scale)
            let sapi_volume = (volume * 100.0).clamp(0.0, 100.0) as u16;
            if let Err(e) = self.voice.SetVolume(sapi_volume) {
                warn!("Failed to set volume: {:?}", e);
            }

            // Convert text to BSTR
            let bstr = BSTR::from(text);

            self.is_speaking.store(true, Ordering::SeqCst);

            // Speak the text asynchronously, then wait for completion
            if let Err(e) = self.voice.Speak(&bstr, SPF_ASYNC.0 as u32, None) {
                error!("Failed to speak: {:?}", e);
                self.is_speaking.store(false, Ordering::SeqCst);
                return;
            }

            // Wait for speech to complete (INFINITE timeout)
            if let Err(e) = self.voice.WaitUntilDone(u32::MAX) {
                warn!("WaitUntilDone failed: {:?}", e);
            }

            self.is_speaking.store(false, Ordering::SeqCst);
        }
    }

    /// Stop current speech
    pub fn stop(&self) {
        unsafe {
            // Purge all queued speech by speaking empty string with purge flag
            let empty = BSTR::new();
            let _ = self
                .voice
                .Speak(&empty, (SPF_ASYNC.0 | SPF_PURGEBEFORESPEAK.0) as u32, None);
        }
        self.is_speaking.store(false, Ordering::SeqCst);
    }

    /// Check if currently speaking
    #[allow(dead_code)]
    pub fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::SeqCst)
    }
}

impl Drop for SystemTts {
    fn drop(&mut self) {
        // Stop any ongoing speech
        self.stop();

        // Uninitialize COM if we initialized it
        if self.com_initialized {
            unsafe {
                CoUninitialize();
            }
        }
    }
}
