//! UI Components

pub mod qrcode;
pub mod window_controls;
pub mod update_dialog;
pub mod feedback_dialog;
mod feedback_log;
mod feedback_store;

pub use qrcode::QrCodeView;
pub use window_controls::{draggable_area, render_window_controls};
