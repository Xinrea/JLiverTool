use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

use crate::events::PluginEvent;
use crate::plugin::{Plugin, PluginMeta, PluginState};
use crate::ws_server::PluginWsServer;

/// GitHub API response for repository contents
#[derive(Debug, serde::Deserialize)]
struct GitHubContent {
    name: String,
    path: String,
    #[serde(rename = "type")]
    content_type: String,
    download_url: Option<String>,
}

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

    /// Clear all loaded plugins
    pub fn clear_plugins(&self) {
        self.plugins.write().unwrap().clear();
    }

    /// Remove a plugin by ID and delete its directory
    pub fn remove_plugin(&self, plugin_id: &str, plugin_path: &PathBuf) -> Result<()> {
        // Remove from loaded plugins
        self.plugins.write().unwrap().remove(plugin_id);

        // Delete the plugin directory
        if plugin_path.exists() && plugin_path.is_dir() {
            fs::remove_dir_all(plugin_path)?;
            log::info!("Removed plugin directory: {:?}", plugin_path);
        }

        Ok(())
    }

    pub fn scan_plugins_dir(&self, plugins_dir: &PathBuf) -> Result<Vec<String>> {
        log::info!("Scanning plugins directory: {:?}", plugins_dir);
        let mut loaded = Vec::new();

        if !plugins_dir.exists() {
            log::info!("Plugins directory does not exist, creating it");
            fs::create_dir_all(plugins_dir)?;
            return Ok(loaded);
        }

        for entry in fs::read_dir(plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let meta_path = path.join("meta.json");
                if meta_path.exists() {
                    log::debug!("Found plugin at {:?}", path);
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

        log::info!("Loaded {} plugins", loaded.len());
        Ok(loaded)
    }

    /// Import a plugin from a GitHub URL
    /// Supports URLs like: https://github.com/owner/repo/tree/branch/path/to/plugin
    /// Branch can contain slashes (e.g., feat/refactor-rust)
    /// Returns the plugin directory path on success
    pub async fn download_plugin_from_github(
        github_url: &str,
        plugins_dir: &PathBuf,
    ) -> Result<PathBuf> {
        // Parse GitHub URL
        // Format: https://github.com/owner/repo/tree/branch/path/to/plugin
        let url = github_url.trim();

        let url_path = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("github.com/");

        // Find /tree/ to split owner/repo from branch/path
        let tree_idx = url_path.find("/tree/");
        if tree_idx.is_none() {
            anyhow::bail!("Invalid GitHub URL format. URL should contain '/tree/'");
        }
        let tree_idx = tree_idx.unwrap();

        let owner_repo = &url_path[..tree_idx];
        let branch_and_path = &url_path[tree_idx + 6..]; // Skip "/tree/"

        let owner_repo_parts: Vec<&str> = owner_repo.split('/').collect();
        if owner_repo_parts.len() < 2 {
            anyhow::bail!("Invalid GitHub URL format. Expected: https://github.com/owner/repo/tree/branch/path");
        }

        let owner = owner_repo_parts[0];
        let repo = owner_repo_parts[1];

        // Now we need to figure out where branch ends and path begins
        // Branch names can contain slashes, so we try progressively longer branch names
        // until we find one that works with the GitHub API
        let branch_path_parts: Vec<&str> = branch_and_path.split('/').collect();

        let mut branch = String::new();
        let mut path = String::new();
        let mut found = false;

        let client = reqwest::Client::new();

        // Try each possible split point
        for i in 1..=branch_path_parts.len() {
            let try_branch = branch_path_parts[..i].join("/");
            let try_path = if i < branch_path_parts.len() {
                branch_path_parts[i..].join("/")
            } else {
                String::new()
            };

            // Test if this branch exists by trying to fetch the contents
            let api_url = format!(
                "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
                owner, repo, try_path, try_branch
            );

            log::debug!("Trying branch '{}' with path '{}'", try_branch, try_path);

            let response = client
                .get(&api_url)
                .header("User-Agent", "JLiverTool")
                .header("Accept", "application/vnd.github.v3+json")
                .send()
                .await;

            if let Ok(resp) = response {
                if resp.status().is_success() {
                    branch = try_branch;
                    path = try_path;
                    found = true;
                    break;
                }
            }
        }

        if !found {
            anyhow::bail!(
                "Could not find valid branch/path combination. Please check the URL is correct."
            );
        }

        // Extract plugin folder name from path
        let plugin_folder_name = path
            .split('/')
            .last()
            .filter(|s| !s.is_empty())
            .unwrap_or(repo);

        log::info!(
            "Importing plugin from GitHub: {}/{} branch:{} path:{}",
            owner,
            repo,
            branch,
            path
        );

        // Create plugin directory
        let plugin_dir = plugins_dir.join(plugin_folder_name);
        if plugin_dir.exists() {
            // Remove existing plugin directory
            fs::remove_dir_all(&plugin_dir)?;
        }
        fs::create_dir_all(&plugin_dir)?;

        // Download plugin files recursively
        Self::download_github_folder(owner, repo, &branch, &path, &plugin_dir).await?;

        Ok(plugin_dir)
    }

    /// Recursively download a folder from GitHub
    async fn download_github_folder(
        owner: &str,
        repo: &str,
        branch: &str,
        path: &str,
        local_dir: &PathBuf,
    ) -> Result<()> {
        let api_url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
            owner, repo, path, branch
        );

        log::debug!("Fetching GitHub contents: {}", api_url);

        let client = reqwest::Client::new();
        let response = client
            .get(&api_url)
            .header("User-Agent", "JLiverTool")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .with_context(|| format!("Failed to fetch GitHub API: {}", api_url))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error ({}): {}", status, body);
        }

        let contents: Vec<GitHubContent> = response
            .json()
            .await
            .with_context(|| "Failed to parse GitHub API response")?;

        for item in contents {
            let local_path = local_dir.join(&item.name);

            if item.content_type == "file" {
                if let Some(download_url) = item.download_url {
                    log::debug!("Downloading file: {} -> {:?}", item.name, local_path);

                    let file_response = client
                        .get(&download_url)
                        .header("User-Agent", "JLiverTool")
                        .send()
                        .await
                        .with_context(|| format!("Failed to download file: {}", download_url))?;

                    let bytes = file_response
                        .bytes()
                        .await
                        .with_context(|| format!("Failed to read file content: {}", item.name))?;

                    fs::write(&local_path, &bytes)
                        .with_context(|| format!("Failed to write file: {:?}", local_path))?;
                }
            } else if item.content_type == "dir" {
                fs::create_dir_all(&local_path)?;

                // Recursively download subdirectory
                Box::pin(Self::download_github_folder(
                    owner,
                    repo,
                    branch,
                    &item.path,
                    &local_path,
                ))
                .await?;
            }
        }

        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
