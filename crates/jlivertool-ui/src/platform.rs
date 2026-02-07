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

    /// Show the window (make visible and bring to front)
    pub fn show_window(window: &Window) {
        use objc::runtime::Object;
        use objc::{msg_send, sel, sel_impl};

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
            let ns_window: *mut Object = msg_send![ns_view, window];
            if ns_window.is_null() {
                tracing::warn!("Failed to get NSWindow from NSView");
                return;
            }

            // Make window visible and bring to front
            let _: () = msg_send![ns_window, makeKeyAndOrderFront: std::ptr::null::<Object>()];
            tracing::info!("Window shown");
        }
    }

    /// Hide the window (minimize to dock or hide)
    pub fn hide_window(window: &Window) {
        use objc::runtime::Object;
        use objc::{msg_send, sel, sel_impl};

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
            let ns_window: *mut Object = msg_send![ns_view, window];
            if ns_window.is_null() {
                tracing::warn!("Failed to get NSWindow from NSView");
                return;
            }

            // Order out (hide without minimizing)
            let _: () = msg_send![ns_window, orderOut: std::ptr::null::<Object>()];
            tracing::info!("Window hidden");
        }
    }

    /// Check if window is visible
    pub fn is_window_visible(window: &Window) -> bool {
        use objc::runtime::{Object, BOOL, YES};
        use objc::{msg_send, sel, sel_impl};

        let handle = match HasWindowHandle::window_handle(window) {
            Ok(h) => h,
            Err(_) => return false,
        };

        let raw_window_handle::RawWindowHandle::AppKit(appkit_handle) = handle.as_raw() else {
            return false;
        };

        let ns_view = appkit_handle.ns_view.as_ptr() as *mut Object;

        unsafe {
            let ns_window: *mut Object = msg_send![ns_view, window];
            if ns_window.is_null() {
                return false;
            }

            let visible: BOOL = msg_send![ns_window, isVisible];
            visible == YES
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use gpui::Window;
    use raw_window_handle::HasWindowHandle;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        IsWindowVisible, SetWindowPos, ShowWindow, HWND_NOTOPMOST, HWND_TOPMOST, SW_HIDE,
        SW_SHOW, SWP_NOMOVE, SWP_NOSIZE,
    };

    /// Set the window to always on top or normal level
    pub fn set_window_always_on_top(window: &Window, always_on_top: bool) {
        // Use HasWindowHandle trait to get the raw window handle
        let handle = match HasWindowHandle::window_handle(window) {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!("Failed to get window handle: {:?}", e);
                return;
            }
        };

        let raw_window_handle::RawWindowHandle::Win32(win32_handle) = handle.as_raw() else {
            tracing::warn!("Not a Win32 window handle");
            return;
        };

        let hwnd = HWND(win32_handle.hwnd.get() as *mut _);
        let insert_after = if always_on_top {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };

        unsafe {
            let result = SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            if result.is_ok() {
                tracing::info!(
                    "Set window always on top: {}",
                    if always_on_top { "enabled" } else { "disabled" }
                );
            } else {
                tracing::warn!("Failed to set window always on top");
            }
        }
    }

    /// Show the window
    pub fn show_window(window: &Window) {
        let handle = match HasWindowHandle::window_handle(window) {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!("Failed to get window handle: {:?}", e);
                return;
            }
        };

        let raw_window_handle::RawWindowHandle::Win32(win32_handle) = handle.as_raw() else {
            tracing::warn!("Not a Win32 window handle");
            return;
        };

        let hwnd = HWND(win32_handle.hwnd.get() as *mut _);

        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            tracing::info!("Window shown");
        }
    }

    /// Hide the window
    pub fn hide_window(window: &Window) {
        let handle = match HasWindowHandle::window_handle(window) {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!("Failed to get window handle: {:?}", e);
                return;
            }
        };

        let raw_window_handle::RawWindowHandle::Win32(win32_handle) = handle.as_raw() else {
            tracing::warn!("Not a Win32 window handle");
            return;
        };

        let hwnd = HWND(win32_handle.hwnd.get() as *mut _);

        unsafe {
            ShowWindow(hwnd, SW_HIDE);
            tracing::info!("Window hidden");
        }
    }

    /// Check if window is visible
    pub fn is_window_visible(window: &Window) -> bool {
        let handle = match HasWindowHandle::window_handle(window) {
            Ok(h) => h,
            Err(_) => return false,
        };

        let raw_window_handle::RawWindowHandle::Win32(win32_handle) = handle.as_raw() else {
            return false;
        };

        let hwnd = HWND(win32_handle.hwnd.get() as *mut _);

        unsafe { IsWindowVisible(hwnd).as_bool() }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod other {
    use gpui::Window;

    /// Set the window to always on top or normal level (no-op on unsupported platforms)
    pub fn set_window_always_on_top(_window: &Window, _always_on_top: bool) {
        tracing::warn!("Always on top is not supported on this platform");
    }

    /// Show the window (no-op on unsupported platforms)
    pub fn show_window(_window: &Window) {
        tracing::warn!("Show window is not supported on this platform");
    }

    /// Hide the window (no-op on unsupported platforms)
    pub fn hide_window(_window: &Window) {
        tracing::warn!("Hide window is not supported on this platform");
    }

    /// Check if window is visible (always returns true on unsupported platforms)
    pub fn is_window_visible(_window: &Window) -> bool {
        true
    }
}

#[cfg(target_os = "macos")]
pub use macos::{hide_window, is_window_visible, set_window_always_on_top, show_window};

#[cfg(target_os = "windows")]
pub use windows_impl::{hide_window, is_window_visible, set_window_always_on_top, show_window};

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use other::{hide_window, is_window_visible, set_window_always_on_top, show_window};
