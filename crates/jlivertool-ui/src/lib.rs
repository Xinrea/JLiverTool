//! JLiverTool UI Library
//!
//! This crate provides GPUI-based UI components for JLiverTool.

pub mod app;
pub mod components;
pub mod http;
pub mod platform;
pub mod theme;
pub mod tray;
pub mod views;

pub use app::{run_app, run_app_with_plugins, run_app_with_tray, window_bounds_from_config, EventReceiver, UiCommand};
pub use components::QrCodeView;
pub use http::IsahcHttpClient;
pub use platform::{hide_window, is_window_visible, set_window_always_on_top, set_window_click_through, show_window};
pub use theme::{set_theme, update_gpui_component_theme};
pub use tray::{TrayCommand, TrayManager, TrayState};
pub use views::setting_view::PluginInfo;
