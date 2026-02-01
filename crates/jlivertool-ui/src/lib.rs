//! JLiverTool UI Library
//!
//! This crate provides GPUI-based UI components for JLiverTool.

pub mod app;
pub mod components;
pub mod http;
pub mod platform;
pub mod theme;
pub mod views;

pub use app::{run_app, window_bounds_from_config, EventReceiver, UiCommand};
pub use components::QrCodeView;
pub use http::IsahcHttpClient;
pub use platform::set_window_always_on_top;
pub use theme::{set_theme, update_gpui_component_theme};
