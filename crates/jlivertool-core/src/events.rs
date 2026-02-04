//! Event system for JLiverTool
//!
//! Provides a type-safe event bus for communication between components.

use crate::bilibili::api::{GuardListItem, OnlineGoldRankItem, UserInfoData};
use crate::messages::{
    DanmuMessage, EntryEffectMessage, GiftMessage, GuardMessage, InteractMessage,
    RoomChangeMessage, SuperChatMessage,
};
use crate::types::{DetailInfo, RoomId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Event types
#[derive(Debug, Clone)]
pub enum Event {
    /// Room info updated
    UpdateRoom {
        room_id: RoomId,
        title: String,
        live_status: u8,
    },

    /// Online count updated
    UpdateOnline { count: u64 },

    /// New danmaku message
    NewDanmu(DanmuMessage),

    /// New gift message
    NewGift(GiftMessage),

    /// New guard purchase
    NewGuard(GuardMessage),

    /// New superchat
    NewSuperChat(SuperChatMessage),

    /// User interaction (enter/follow)
    NewInteract(InteractMessage),

    /// Entry effect (guard entry)
    NewEntryEffect(EntryEffectMessage),

    /// Room change (title/area change)
    RoomChange(RoomChangeMessage),

    /// Config value changed
    ConfigChanged { key: String, value: serde_json::Value },

    /// Initial config loaded
    ConfigLoaded {
        always_on_top: bool,
        guard_effect: bool,
        level_effect: bool,
        opacity: f32,
        lite_mode: bool,
        medal_display: bool,
        interact_display: bool,
        theme: String,
        font_size: f32,
        tts_enabled: bool,
        tts_gift_enabled: bool,
        tts_sc_enabled: bool,
        tts_volume: f32,
        max_danmu_count: usize,
        log_level: String,
        auto_update_check: bool,
    },

    /// Detail window data updated
    DetailUpdate(DetailInfo),

    /// Live stream started
    LiveStart,

    /// Live stream ended
    LiveEnd,

    /// Connection status changed
    ConnectionStatus { connected: bool },

    /// Login status changed
    LoginStatusChanged {
        logged_in: bool,
        user_info: Option<UserInfoData>,
    },

    /// Request to start QR login
    RequestQrLogin,

    /// QR code generated for login
    QrCodeGenerated { url: String, qrcode_key: String },

    /// QR code login status updated
    QrLoginStatus {
        status: crate::bilibili::api::QrCodeStatus,
    },

    /// Request to logout
    RequestLogout,

    /// RTMP info received from start live
    RtmpInfo {
        addr: String,
        code: String,
    },

    /// Clear danmu list (used when changing rooms)
    ClearDanmuList,

    /// User info fetched from API
    UserInfoFetched {
        uid: u64,
        user_info: UserInfoData,
    },

    /// Audience list fetched (online gold rank)
    AudienceListFetched {
        list: Vec<OnlineGoldRankItem>,
    },

    /// Guard list fetched
    GuardListFetched {
        list: Vec<GuardListItem>,
        total: u32,
        page: u32,
    },

    /// Plugins list refreshed
    PluginsRefreshed {
        plugins: Vec<PluginInfoEvent>,
    },

    /// Plugin import result
    PluginImportResult {
        success: bool,
        message: String,
    },

    /// All data cleared
    DataCleared,

    /// Update check result
    UpdateCheckResult {
        has_update: bool,
        current_version: String,
        latest_version: String,
        release_url: String,
        error: Option<String>,
    },
}

/// Plugin info for events (simplified version)
#[derive(Debug, Clone)]
pub struct PluginInfoEvent {
    pub id: String,
    pub name: String,
    pub author: String,
    pub desc: String,
    pub version: String,
    pub path: std::path::PathBuf,
}

/// Event handler callback type
pub type EventHandler = Box<dyn Fn(&Event) + Send + Sync>;

/// Event bus for distributing events to subscribers
#[derive(Clone)]
pub struct EventBus {
    inner: Arc<EventBusInner>,
}

struct EventBusInner {
    sender: broadcast::Sender<Event>,
    handlers: RwLock<HashMap<String, Vec<EventHandler>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);

        Self {
            inner: Arc::new(EventBusInner {
                sender,
                handlers: RwLock::new(HashMap::new()),
            }),
        }
    }

    /// Subscribe to all events
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.inner.sender.subscribe()
    }

    /// Emit an event
    pub fn emit(&self, event: Event) {
        // Send to broadcast subscribers
        let _ = self.inner.sender.send(event.clone());

        // Call registered handlers
        let event_type = Self::event_type_name(&event);
        let handlers = self.inner.handlers.read();

        // Call type-specific handlers
        if let Some(type_handlers) = handlers.get(event_type) {
            for handler in type_handlers {
                handler(&event);
            }
        }

        // Call wildcard handlers
        if let Some(all_handlers) = handlers.get("*") {
            for handler in all_handlers {
                handler(&event);
            }
        }
    }

    /// Register a handler for a specific event type
    pub fn on<F>(&self, event_type: &str, handler: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        let mut handlers = self.inner.handlers.write();
        handlers
            .entry(event_type.to_string())
            .or_default()
            .push(Box::new(handler));
    }

    /// Register a handler for all events
    pub fn on_all<F>(&self, handler: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.on("*", handler);
    }

    /// Get the type name for an event
    fn event_type_name(event: &Event) -> &'static str {
        match event {
            Event::UpdateRoom { .. } => "update_room",
            Event::UpdateOnline { .. } => "update_online",
            Event::NewDanmu(_) => "new_danmu",
            Event::NewGift(_) => "new_gift",
            Event::NewGuard(_) => "new_guard",
            Event::NewSuperChat(_) => "new_superchat",
            Event::NewInteract(_) => "new_interact",
            Event::NewEntryEffect(_) => "new_entry_effect",
            Event::RoomChange(_) => "room_change",
            Event::ConfigChanged { .. } => "config_changed",
            Event::ConfigLoaded { .. } => "config_loaded",
            Event::DetailUpdate(_) => "detail_update",
            Event::LiveStart => "live_start",
            Event::LiveEnd => "live_end",
            Event::ConnectionStatus { .. } => "connection_status",
            Event::LoginStatusChanged { .. } => "login_status_changed",
            Event::RequestQrLogin => "request_qr_login",
            Event::QrCodeGenerated { .. } => "qr_code_generated",
            Event::QrLoginStatus { .. } => "qr_login_status",
            Event::RequestLogout => "request_logout",
            Event::RtmpInfo { .. } => "rtmp_info",
            Event::ClearDanmuList => "clear_danmu_list",
            Event::UserInfoFetched { .. } => "user_info_fetched",
            Event::AudienceListFetched { .. } => "audience_list_fetched",
            Event::GuardListFetched { .. } => "guard_list_fetched",
            Event::PluginsRefreshed { .. } => "plugins_refreshed",
            Event::PluginImportResult { .. } => "plugin_import_result",
            Event::DataCleared => "data_cleared",
            Event::UpdateCheckResult { .. } => "update_check_result",
        }
    }
}

/// Event types enum for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    UpdateRoom,
    UpdateOnline,
    NewDanmu,
    NewGift,
    NewGuard,
    NewSuperChat,
    NewInteract,
    NewEntryEffect,
    RoomChange,
    ConfigChanged,
    DetailUpdate,
    LiveStart,
    LiveEnd,
    ConnectionStatus,
    LoginStatusChanged,
    RequestQrLogin,
    QrCodeGenerated,
    QrLoginStatus,
    RequestLogout,
    ClearDanmuList,
    UserInfoFetched,
    AudienceListFetched,
    GuardListFetched,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UpdateRoom => "update_room",
            Self::UpdateOnline => "update_online",
            Self::NewDanmu => "new_danmu",
            Self::NewGift => "new_gift",
            Self::NewGuard => "new_guard",
            Self::NewSuperChat => "new_superchat",
            Self::NewInteract => "new_interact",
            Self::NewEntryEffect => "new_entry_effect",
            Self::RoomChange => "room_change",
            Self::ConfigChanged => "config_changed",
            Self::DetailUpdate => "detail_update",
            Self::LiveStart => "live_start",
            Self::LiveEnd => "live_end",
            Self::ConnectionStatus => "connection_status",
            Self::LoginStatusChanged => "login_status_changed",
            Self::RequestQrLogin => "request_qr_login",
            Self::QrCodeGenerated => "qr_code_generated",
            Self::QrLoginStatus => "qr_login_status",
            Self::RequestLogout => "request_logout",
            Self::ClearDanmuList => "clear_danmu_list",
            Self::UserInfoFetched => "user_info_fetched",
            Self::AudienceListFetched => "audience_list_fetched",
            Self::GuardListFetched => "guard_list_fetched",
        }
    }
}

/// Helper macro for matching events
#[macro_export]
macro_rules! match_event {
    ($event:expr, $($pattern:pat => $handler:expr),+ $(,)?) => {
        match $event {
            $($pattern => $handler),+
            _ => {}
        }
    };
}
