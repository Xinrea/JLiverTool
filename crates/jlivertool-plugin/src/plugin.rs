use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    pub id: String,
    pub name: String,
    pub author: String,
    pub desc: String,
    pub version: String,
    #[serde(default)]
    pub url: Option<String>,
    pub index: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    Unloaded,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub meta: PluginMeta,
    pub path: PathBuf,
    pub state: PluginState,
    pub enabled: bool,
}

impl Plugin {
    pub fn new(meta: PluginMeta, path: PathBuf) -> Self {
        Self {
            meta,
            path,
            state: PluginState::Unloaded,
            enabled: true,
        }
    }

    pub fn index_path(&self) -> PathBuf {
        self.path.join(&self.meta.index)
    }
}
