//! Application entry point

use crate::http::IsahcHttpClient;
use crate::theme::update_gpui_component_theme;
use crate::views::MainView;
use gpui::*;
use gpui_component::init;
use gpui_component::Root;
use jlivertool_core::config::{ConfigStore, WindowConfig};
use jlivertool_core::database::Database;
use jlivertool_core::events::Event;
use jlivertool_core::types::WindowType;
use parking_lot::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;

/// UI command that can be sent from UI to backend
#[derive(Debug, Clone)]
pub enum UiCommand {
    /// Request QR code login
    RequestQrLogin,
    /// Request logout
    RequestLogout,
    /// Change room
    ChangeRoom(u64),
    /// Send danmu message
    SendDanmu { room_id: u64, message: String },
    /// Update room title
    UpdateRoomTitle { room_id: u64, title: String },
    /// Start live streaming
    StartLive { room_id: u64, area_v2: u64 },
    /// Stop live streaming
    StopLive { room_id: u64 },
    /// Update window display settings
    UpdateWindowSettings {
        lite_mode: bool,
        medal_display: bool,
        interact_display: bool,
        guard_effect: bool,
        level_effect: bool,
    },
    /// Update opacity setting
    UpdateOpacity(f32),
    /// Update theme setting
    UpdateTheme(String),
    /// Update font size setting
    UpdateFontSize(f32),
    /// Update always on top setting
    UpdateAlwaysOnTop(bool),
    /// Fetch user info by UID
    FetchUserInfo { uid: u64 },
    /// Fetch audience list (online gold rank)
    FetchAudienceList { room_id: u64, ruid: u64 },
    /// Fetch guard list
    FetchGuardList { room_id: u64, ruid: u64, page: u32 },
    /// Save window bounds
    SaveWindowBounds {
        window_type: WindowType,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    /// Update TTS enabled settings
    UpdateTtsEnabled {
        danmu: bool,
        gift: bool,
        superchat: bool,
    },
    /// Update TTS volume
    UpdateTtsVolume(f32),
    /// Update TTS provider
    UpdateTtsProvider(String),
    /// Test TTS
    TestTts,
    /// Refresh plugins list
    RefreshPlugins,
    /// Import plugin from GitHub URL
    ImportPlugin(String),
    /// Remove a plugin by ID and path
    RemovePlugin { plugin_id: String, plugin_path: std::path::PathBuf },
    /// Update advanced settings (max danmu count, log level)
    UpdateAdvancedSettings { max_danmu: usize, log_level: String },
    /// Clear all data (danmu, gifts, superchats)
    ClearAllData,
    /// Open data folder
    OpenDataFolder,
    /// Check for updates
    CheckForUpdate,
    /// Update auto-update check setting
    UpdateAutoUpdateCheck(bool),
}

/// Wrapper for event receiver with a flag to indicate pending events
pub struct EventReceiver {
    pub rx: mpsc::Receiver<Event>,
    pub has_events: Arc<AtomicBool>,
}

impl EventReceiver {
    pub fn new(rx: mpsc::Receiver<Event>, has_events: Arc<AtomicBool>) -> Self {
        Self { rx, has_events }
    }
}

/// Run the GPUI application
pub fn run_app(
    event_rx: mpsc::Receiver<Event>,
    command_tx: mpsc::Sender<UiCommand>,
    database: Option<Arc<Database>>,
    config: Option<Arc<RwLock<ConfigStore>>>,
    has_events: Arc<AtomicBool>,
) {
    run_app_with_plugins(event_rx, command_tx, database, config, has_events, Vec::new(), None);
}

/// Run the GPUI application with plugin info
pub fn run_app_with_plugins(
    event_rx: mpsc::Receiver<Event>,
    command_tx: mpsc::Sender<UiCommand>,
    database: Option<Arc<Database>>,
    config: Option<Arc<RwLock<ConfigStore>>>,
    has_events: Arc<AtomicBool>,
    plugins: Vec<crate::views::setting_view::PluginInfo>,
    ws_port: Option<u16>,
) {
    // Get saved window bounds for main window
    let main_window_config = config
        .as_ref()
        .map(|c| c.read().get_window_config(WindowType::Main))
        .unwrap_or_default();

    Application::new().run(move |cx| {
        // Initialize gpui-component
        init(cx);

        // Configure gpui-component theme colors based on current theme
        update_gpui_component_theme(cx);

        // Setup HTTP client for loading remote images
        if let Ok(http_client) = IsahcHttpClient::new() {
            cx.set_http_client(http_client);
        }

        // Create main window bounds - use saved or default
        let bounds = if main_window_config.width > 0 && main_window_config.height > 0 {
            Bounds::new(
                point(px(main_window_config.x as f32), px(main_window_config.y as f32)),
                size(px(main_window_config.width as f32), px(main_window_config.height as f32)),
            )
        } else {
            Bounds::centered(None, size(px(450.0), px(800.0)), cx)
        };

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("JLiverTool".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_background: WindowBackgroundAppearance::Transparent,
                // Minimum window size
                window_min_size: Some(size(px(350.0), px(400.0))),
                ..Default::default()
            },
            |window, cx| {
                // Create MainView
                let main_view = cx.new(|cx| {
                    let mut view = MainView::new(event_rx, command_tx, has_events, cx);
                    if let Some(db) = database {
                        view.set_database(db);
                    }
                    if let Some(cfg) = config {
                        view.set_config(cfg);
                    }
                    // Set plugins if any
                    if !plugins.is_empty() {
                        view.set_plugins(plugins, cx);
                    }
                    // Set WebSocket port for plugin communication
                    if let Some(port) = ws_port {
                        view.set_ws_port(port);
                    }
                    view
                });
                // Wrap with Root to support gpui-component Input
                cx.new(|cx| Root::new(main_view, window, cx))
            },
        )
        .unwrap();
    });
}

/// Helper to create window bounds from WindowConfig or default
pub fn window_bounds_from_config(
    config: &WindowConfig,
    default_width: f32,
    default_height: f32,
    cx: &App,
) -> Bounds<Pixels> {
    if config.width > 0 && config.height > 0 {
        Bounds::new(
            point(px(config.x as f32), px(config.y as f32)),
            size(px(config.width as f32), px(config.height as f32)),
        )
    } else {
        Bounds::centered(None, size(px(default_width), px(default_height)), cx)
    }
}
