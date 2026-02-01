//! JLiverTool Core Library
//!
//! This crate provides the core functionality for JLiverTool, including:
//! - Bilibili API client
//! - WebSocket danmaku connection
//! - Event system
//! - Configuration storage
//! - Data models
//! - SQLite database

pub mod bilibili;
pub mod config;
pub mod database;
pub mod events;
pub mod messages;
pub mod types;

pub use bilibili::api::BiliApi;
pub use bilibili::ws::BiliWebSocket;
pub use config::ConfigStore;
pub use database::Database;
pub use events::{Event, EventBus};
