//! Platform-specific utilities

#![allow(unexpected_cfgs)]

#[cfg(target_os = "macos")]
mod macos {
    use gpui::Window;
    use raw_window_handle::HasWindowHandle;

    /// Window level constants for macOS
    const NS_NORMAL_WINDOW_LEVEL: i64 = 0;
    const NS_FLOATING_WINDOW_LEVEL: i64 = 3; // NSFloatingWindowLevel

    /// Set the window to always on top or normal level
    pub fn set_window_always_on_top(window: &Window, always_on_top: bool) {
        use objc::runtime::Object;
        use objc::{msg_send, sel, sel_impl};

        // Use HasWindowHandle trait to get the raw window handle
        let handle = match HasWindowHandle::window_handle(window) {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!("Failed to get window handle: {:?}", e);
                return;
            }
        };

        let raw_window_handle::RawWindowHandle::AppKit(appkit_handle) = handle.as_raw() else {
            tracing::warn!("Not an AppKit window handle");
            return;
        };

        let ns_view = appkit_handle.ns_view.as_ptr() as *mut Object;

        unsafe {
            // Get NSWindow from NSView
            let ns_window: *mut Object = msg_send![ns_view, window];
            if ns_window.is_null() {
                tracing::warn!("Failed to get NSWindow from NSView");
                return;
            }

            // Set window level
            let level = if always_on_top {
                NS_FLOATING_WINDOW_LEVEL
            } else {
                NS_NORMAL_WINDOW_LEVEL
            };

            let _: () = msg_send![ns_window, setLevel: level];
            tracing::info!("Set window level to {}", level);
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod other {
    use gpui::Window;

    /// Set the window to always on top or normal level (no-op on non-macOS)
    pub fn set_window_always_on_top(_window: &Window, _always_on_top: bool) {
        tracing::warn!("Always on top is not supported on this platform");
    }
}

#[cfg(target_os = "macos")]
pub use macos::set_window_always_on_top;

#[cfg(not(target_os = "macos"))]
pub use other::set_window_always_on_top;
