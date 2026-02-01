pub mod events;
pub mod ipc;
pub mod manager;
pub mod plugin;
pub mod ws_server;

pub use events::PluginEvent;
pub use manager::PluginManager;
pub use plugin::{Plugin, PluginMeta, PluginState};
pub use ws_server::PluginWsServer;
