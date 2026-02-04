//! Configuration storage with persistence and change notifications

use crate::types::{Cookies, RoomId, WindowType};
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use parking_lot::RwLock;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

/// Configuration store version
const CONFIG_VERSION: u32 = 2;
const CONFIG_FILENAME: &str = "config_v2.json";

/// Custom serialization for cookies - encodes as base64
mod encoded_cookies {
    use super::*;

    pub fn serialize<S>(cookies: &Option<Cookies>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match cookies {
            Some(c) => {
                // Serialize cookies to JSON, then base64 encode
                let json = serde_json::to_string(c).map_err(serde::ser::Error::custom)?;
                let encoded = BASE64.encode(json.as_bytes());
                serializer.serialize_some(&encoded)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Cookies>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<Value> = Option::deserialize(deserializer)?;
        match opt {
            Some(Value::String(encoded)) => {
                // Try to decode as base64 first (new format)
                if let Ok(decoded_bytes) = BASE64.decode(&encoded) {
                    if let Ok(json_str) = String::from_utf8(decoded_bytes) {
                        if let Ok(cookies) = serde_json::from_str::<Cookies>(&json_str) {
                            return Ok(Some(cookies));
                        }
                    }
                }
                // If base64 decode fails, it might be old format - ignore
                Ok(None)
            }
            Some(Value::Object(map)) => {
                // Old format: cookies stored as plain JSON object
                // Convert to Cookies struct
                let cookies: Cookies =
                    serde_json::from_value(Value::Object(map)).map_err(serde::de::Error::custom)?;
                Ok(Some(cookies))
            }
            _ => Ok(None),
        }
    }
}

/// Window position and size
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowConfig {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// TTS provider type
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TtsProvider {
    #[default]
    None,
    System,
    Aliyun,
    Custom,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub version: u32,

    #[serde(default, with = "encoded_cookies")]
    pub cookies: Option<Cookies>,

    #[serde(default)]
    pub room: Option<RoomId>,

    #[serde(default)]
    pub login: bool,

    #[serde(default)]
    pub merge: bool,

    #[serde(default)]
    pub merge_rooms: Vec<RoomId>,

    #[serde(default)]
    pub always_on_top: bool,

    #[serde(default = "default_max_detail_entry")]
    pub max_detail_entry: usize,

    #[serde(default)]
    pub guard_effect: bool,

    #[serde(default)]
    pub level_effect: bool,

    #[serde(default = "default_opacity")]
    pub opacity: f32,

    #[serde(default)]
    pub lite_mode: bool,

    #[serde(default = "default_medal_display")]
    pub medal_display: bool,

    #[serde(default)]
    pub interact_display: bool,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_font_size")]
    pub font_size: f32,

    #[serde(default)]
    pub plugin_list: Vec<String>,

    #[serde(default)]
    pub windows: HashMap<String, WindowConfig>,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default = "default_max_danmu_count")]
    pub max_danmu_count: usize,

    #[serde(default)]
    pub tts_provider: TtsProvider,

    #[serde(default)]
    pub tts_aliyun_app_key: String,

    #[serde(default)]
    pub tts_aliyun_access_key_id: String,

    #[serde(default)]
    pub tts_aliyun_access_key_secret: String,

    #[serde(default)]
    pub tts_custom_url: String,

    #[serde(default)]
    pub tts_enabled: bool,

    #[serde(default)]
    pub tts_gift_enabled: bool,

    #[serde(default)]
    pub tts_sc_enabled: bool,

    #[serde(default = "default_tts_volume")]
    pub tts_volume: f32,

    #[serde(default = "default_auto_update_check")]
    pub auto_update_check: bool,

    // Extra fields for extensibility
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

fn default_auto_update_check() -> bool {
    true
}

fn default_max_detail_entry() -> usize {
    100
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_danmu_count() -> usize {
    200
}

fn default_opacity() -> f32 {
    1.0
}

fn default_medal_display() -> bool {
    true
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_font_size() -> f32 {
    14.0
}

fn default_tts_volume() -> f32 {
    1.0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            cookies: None,
            room: None,
            login: false,
            merge: false,
            merge_rooms: Vec::new(),
            always_on_top: false,
            max_detail_entry: default_max_detail_entry(),
            guard_effect: false,
            level_effect: false,
            opacity: default_opacity(),
            lite_mode: false,
            medal_display: default_medal_display(),
            interact_display: false,
            theme: default_theme(),
            font_size: default_font_size(),
            plugin_list: Vec::new(),
            windows: HashMap::new(),
            log_level: default_log_level(),
            max_danmu_count: default_max_danmu_count(),
            tts_provider: TtsProvider::None,
            tts_aliyun_app_key: String::new(),
            tts_aliyun_access_key_id: String::new(),
            tts_aliyun_access_key_secret: String::new(),
            tts_custom_url: String::new(),
            tts_enabled: false,
            tts_gift_enabled: false,
            tts_sc_enabled: false,
            tts_volume: default_tts_volume(),
            auto_update_check: default_auto_update_check(),
            extra: HashMap::new(),
        }
    }
}

/// Configuration change event
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    pub key: String,
    pub value: Value,
}

/// Configuration store with persistence
#[derive(Clone)]
pub struct ConfigStore {
    inner: Arc<ConfigStoreInner>,
}

struct ConfigStoreInner {
    config: RwLock<Config>,
    config_path: PathBuf,
    change_tx: broadcast::Sender<ConfigChangeEvent>,
}

impl ConfigStore {
    /// Create a new config store, loading from file if exists
    pub fn new() -> Result<Self> {
        let config_dir = directories::ProjectDirs::from("com", "jlivertool", "JLiverTool")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join(CONFIG_FILENAME);

        let config = if config_path.exists() {
            info!("Loading config from {:?}", config_path);
            let content = std::fs::read_to_string(&config_path)?;
            match serde_json::from_str(&content) {
                Ok(cfg) => {
                    debug!("Config loaded successfully");
                    cfg
                }
                Err(e) => {
                    warn!("Failed to parse config, using defaults: {}", e);
                    Config::default()
                }
            }
        } else {
            info!("No config file found, using defaults");
            Config::default()
        };

        let (change_tx, _) = broadcast::channel(100);

        Ok(Self {
            inner: Arc::new(ConfigStoreInner {
                config: RwLock::new(config),
                config_path,
                change_tx,
            }),
        })
    }

    /// Create a config store with a custom path
    pub fn with_path(path: PathBuf) -> Result<Self> {
        let config = if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Config::default()
        };

        let (change_tx, _) = broadcast::channel(100);

        Ok(Self {
            inner: Arc::new(ConfigStoreInner {
                config: RwLock::new(config),
                config_path: path,
                change_tx,
            }),
        })
    }

    /// Get the entire config
    pub fn get_config(&self) -> Config {
        self.inner.config.read().clone()
    }

    /// Get a specific value by key (supports dot notation)
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let config = self.inner.config.read();
        let value = serde_json::to_value(&*config).ok()?;

        let mut current = &value;
        for part in key.split('.') {
            current = current.get(part)?;
        }

        serde_json::from_value(current.clone()).ok()
    }

    /// Get a value as JSON Value
    pub fn get_value(&self, key: &str) -> Option<Value> {
        let config = self.inner.config.read();
        let value = serde_json::to_value(&*config).ok()?;

        let mut current = &value;
        for part in key.split('.') {
            current = current.get(part)?;
        }

        Some(current.clone())
    }

    /// Set a value by key and persist
    pub fn set<T: Serialize>(&self, key: &str, value: T) -> Result<()> {
        let json_value = serde_json::to_value(&value)?;

        {
            let mut config = self.inner.config.write();

            // Convert config to JSON, update, and convert back
            let mut config_value = serde_json::to_value(&*config)?;

            // Navigate to the parent and set the value
            let parts: Vec<&str> = key.split('.').collect();
            if parts.len() == 1 {
                if let Some(obj) = config_value.as_object_mut() {
                    obj.insert(key.to_string(), json_value.clone());
                }
            } else {
                let mut current = &mut config_value;
                for (i, part) in parts.iter().enumerate() {
                    if i == parts.len() - 1 {
                        if let Some(obj) = current.as_object_mut() {
                            obj.insert(part.to_string(), json_value.clone());
                        }
                    } else {
                        current = current
                            .as_object_mut()
                            .and_then(|obj| obj.get_mut(*part))
                            .ok_or_else(|| anyhow::anyhow!("Invalid key path: {}", key))?;
                    }
                }
            }

            *config = serde_json::from_value(config_value)?;
        }

        // Persist to file
        self.save()?;

        // Notify listeners
        let _ = self.inner.change_tx.send(ConfigChangeEvent {
            key: key.to_string(),
            value: json_value,
        });

        Ok(())
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config = self.inner.config.read();
        let content = serde_json::to_string_pretty(&*config)?;
        std::fs::write(&self.inner.config_path, content)?;
        debug!("Config saved to {:?}", self.inner.config_path);
        Ok(())
    }

    /// Subscribe to config changes
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.inner.change_tx.subscribe()
    }

    // Convenience methods for common config values

    /// Get cookies
    pub fn get_cookies(&self) -> Option<Cookies> {
        self.inner.config.read().cookies.clone()
    }

    /// Set cookies
    pub fn set_cookies(&self, cookies: Option<Cookies>) -> Result<()> {
        {
            let mut config = self.inner.config.write();
            config.cookies = cookies.clone();
            config.login = cookies.is_some();
        }
        self.save()
    }

    /// Get current room
    pub fn get_room(&self) -> Option<RoomId> {
        self.inner.config.read().room.clone()
    }

    /// Set current room
    pub fn set_room(&self, room: RoomId) -> Result<()> {
        {
            let mut config = self.inner.config.write();
            config.room = Some(room);
        }
        self.save()
    }

    /// Get opacity directly (avoids JSON serialization)
    pub fn get_opacity(&self) -> f32 {
        self.inner.config.read().opacity
    }

    /// Get font size directly (avoids JSON serialization)
    pub fn get_font_size(&self) -> f32 {
        self.inner.config.read().font_size
    }

    /// Get theme directly (avoids JSON serialization)
    pub fn get_theme(&self) -> String {
        self.inner.config.read().theme.clone()
    }

    /// Get lite mode directly (avoids JSON serialization)
    pub fn get_lite_mode(&self) -> bool {
        self.inner.config.read().lite_mode
    }

    /// Get medal display directly (avoids JSON serialization)
    pub fn get_medal_display(&self) -> bool {
        self.inner.config.read().medal_display
    }

    /// Get interact display directly (avoids JSON serialization)
    pub fn get_interact_display(&self) -> bool {
        self.inner.config.read().interact_display
    }

    /// Get guard effect directly (avoids JSON serialization)
    pub fn get_guard_effect(&self) -> bool {
        self.inner.config.read().guard_effect
    }

    /// Get level effect directly (avoids JSON serialization)
    pub fn get_level_effect(&self) -> bool {
        self.inner.config.read().level_effect
    }

    /// Get window config
    pub fn get_window_config(&self, window_type: WindowType) -> WindowConfig {
        let key = format!("{:?}", window_type).to_lowercase();
        self.inner
            .config
            .read()
            .windows
            .get(&key)
            .cloned()
            .unwrap_or_default()
    }

    /// Set window config
    pub fn set_window_config(&self, window_type: WindowType, config: WindowConfig) -> Result<()> {
        let key = format!("{:?}", window_type).to_lowercase();
        {
            let mut cfg = self.inner.config.write();
            cfg.windows.insert(key, config);
        }
        self.save()
    }

    /// Check if always on top is enabled
    pub fn is_always_on_top(&self) -> bool {
        self.inner.config.read().always_on_top
    }

    /// Set always on top
    pub fn set_always_on_top(&self, value: bool) -> Result<()> {
        {
            let mut config = self.inner.config.write();
            config.always_on_top = value;
        }
        self.save()
    }

    /// Check if merge rooms is enabled
    pub fn is_merge_enabled(&self) -> bool {
        self.inner.config.read().merge
    }

    /// Get merge rooms
    pub fn get_merge_rooms(&self) -> Vec<RoomId> {
        self.inner.config.read().merge_rooms.clone()
    }

    /// Get plugin list
    pub fn get_plugin_list(&self) -> Vec<String> {
        self.inner.config.read().plugin_list.clone()
    }

    /// Get the data directory path
    pub fn data_dir(&self) -> PathBuf {
        self.inner.config_path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new().expect("Failed to create config store")
    }
}
