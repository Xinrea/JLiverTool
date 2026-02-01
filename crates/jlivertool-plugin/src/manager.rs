use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

use crate::events::PluginEvent;
use crate::plugin::{Plugin, PluginMeta, PluginState};
use crate::ws_server::PluginWsServer;

/// Plugin manager handles plugin lifecycle and event broadcasting
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Plugin>>>,
    ws_server: Option<PluginWsServer>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            ws_server: None,
        }
    }

    /// Get the WebSocket server (if started)
    pub fn ws_server(&self) -> Option<&PluginWsServer> {
        self.ws_server.as_ref()
    }

    /// Get the WebSocket URL for plugins
    pub fn ws_url(&self) -> Option<String> {
        self.ws_server.as_ref().map(|s| s.ws_url())
    }

    /// Start the WebSocket server for plugin communication
    pub async fn start_ws_server(&mut self) -> Result<u16> {
        let mut server = PluginWsServer::new();
        server.start().await?;
        let port = server.port();
        self.ws_server = Some(server);
        Ok(port)
    }

    /// Stop the WebSocket server
    pub async fn stop_ws_server(&mut self) {
        if let Some(mut server) = self.ws_server.take() {
            server.stop().await;
        }
    }

    /// Broadcast an event to all connected plugins
    pub fn broadcast_event(&self, event: &PluginEvent) {
        if let Some(ref server) = self.ws_server {
            server.broadcast_event(event.clone());
        }
    }

    /// Get the event sender for broadcasting events from other threads
    /// Returns None if the WebSocket server hasn't been started
    pub fn get_event_sender(&self) -> Option<broadcast::Sender<PluginEvent>> {
        self.ws_server.as_ref().map(|s| s.event_sender())
    }

    pub fn load_plugin(&self, path: PathBuf) -> Result<String> {
        let meta_path = path.join("meta.json");
        let meta_content = fs::read_to_string(&meta_path)
            .with_context(|| format!("Failed to read meta.json at {:?}", meta_path))?;

        let meta: PluginMeta = serde_json::from_str(&meta_content)
            .with_context(|| format!("Failed to parse meta.json at {:?}", meta_path))?;

        let plugin_id = meta.id.clone();

        // Verify index file exists
        let index_path = path.join(&meta.index);
        if !index_path.exists() {
            anyhow::bail!("Plugin index file not found: {:?}", index_path);
        }

        let mut plugin = Plugin::new(meta, path);
        plugin.state = PluginState::Loaded;

        self.plugins
            .write()
            .unwrap()
            .insert(plugin_id.clone(), plugin);

        log::info!("Loaded plugin: {}", plugin_id);
        Ok(plugin_id)
    }

    pub fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(mut plugin) = plugins.remove(plugin_id) {
            plugin.state = PluginState::Unloaded;
            log::info!("Unloaded plugin: {}", plugin_id);
            Ok(())
        } else {
            anyhow::bail!("Plugin not found: {}", plugin_id)
        }
    }

    pub fn get_plugins(&self) -> Vec<PluginMeta> {
        self.plugins
            .read()
            .unwrap()
            .values()
            .map(|p| p.meta.clone())
            .collect()
    }

    /// Get all plugins with their paths
    pub fn get_plugins_with_paths(&self) -> Vec<(PluginMeta, PathBuf)> {
        self.plugins
            .read()
            .unwrap()
            .values()
            .map(|p| (p.meta.clone(), p.path.clone()))
            .collect()
    }

    pub fn get_plugin(&self, plugin_id: &str) -> Option<Plugin> {
        self.plugins.read().unwrap().get(plugin_id).cloned()
    }

    pub fn scan_plugins_dir(&self, plugins_dir: &PathBuf) -> Result<Vec<String>> {
        let mut loaded = Vec::new();

        if !plugins_dir.exists() {
            fs::create_dir_all(plugins_dir)?;
            return Ok(loaded);
        }

        for entry in fs::read_dir(plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let meta_path = path.join("meta.json");
                if meta_path.exists() {
                    match self.load_plugin(path.clone()) {
                        Ok(plugin_id) => {
                            loaded.push(plugin_id);
                        }
                        Err(e) => {
                            log::error!("Failed to load plugin at {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        Ok(loaded)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
