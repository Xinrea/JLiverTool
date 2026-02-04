//! JLiverTool - Bilibili Live Tool
//!
//! A desktop application for viewing Bilibili live stream danmaku, gifts, and more.

// Hide console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use jlivertool_core::bilibili::api::{BiliApi, QrCodeStatus};
use jlivertool_core::bilibili::ws::{BiliWebSocket, WsEvent, WsInfo};
use jlivertool_core::config::ConfigStore;
use jlivertool_core::database::Database;
use jlivertool_core::events::Event;
use jlivertool_core::messages::{
    DanmuMessage, EntryEffectMessage, GiftMessage, GuardMessage, InteractMessage,
    OnlineRankCountMessage, RoomChangeMessage, SuperChatMessage,
};
use jlivertool_core::tts::{TtsEnabled, TtsManager, TtsMessage};
use jlivertool_core::types::RoomId;
use jlivertool_plugin::PluginManager;
use jlivertool_ui::{run_app_with_plugins, PluginInfo, UiCommand};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use tokio::sync::mpsc as tokio_mpsc;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Get the data directory for the application
fn get_data_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "jlivertool", "JLiverTool")
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Initialize logging with both console and file output
fn init_logging(data_dir: &PathBuf) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    // Create logs directory
    let logs_dir = data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)?;

    // Create a rolling file appender (daily rotation, keep 7 days)
    let file_appender = tracing_appender::rolling::daily(&logs_dir, "jlivertool.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Set up the subscriber with both console and file output
    let env_filter = EnvFilter::from_default_env()
        .add_directive("jlivertool=info".parse()?)
        .add_directive("jlivertool_core=info".parse()?)
        .add_directive("jlivertool_ui=info".parse()?)
        .add_directive("jlivertool_plugin=info".parse()?);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    Ok(guard)
}

/// Backend control commands
enum BackendCommand {
    /// Change to a new room
    ChangeRoom(RoomId),
    /// Stop the backend (reserved for future use)
    #[allow(dead_code)]
    Stop,
}

/// Event sender wrapper that sets a flag when events are sent
/// Also broadcasts events to plugins if a plugin event sender is set
#[derive(Clone)]
struct EventSender {
    tx: mpsc::Sender<Event>,
    has_events: Arc<AtomicBool>,
    plugin_tx: Option<tokio::sync::broadcast::Sender<jlivertool_plugin::PluginEvent>>,
}

impl EventSender {
    fn new(tx: mpsc::Sender<Event>, has_events: Arc<AtomicBool>) -> Self {
        Self { tx, has_events, plugin_tx: None }
    }

    fn with_plugin_sender(mut self, plugin_tx: tokio::sync::broadcast::Sender<jlivertool_plugin::PluginEvent>) -> Self {
        self.plugin_tx = Some(plugin_tx);
        self
    }

    fn send(&self, event: Event) -> Result<(), mpsc::SendError<Event>> {
        // Broadcast to plugins if sender is available
        if let Some(ref plugin_tx) = self.plugin_tx {
            if let Some(plugin_event) = jlivertool_plugin::PluginEvent::from_core_event(&event) {
                let _ = plugin_tx.send(plugin_event);
            }
        }

        let result = self.tx.send(event);
        if result.is_ok() {
            self.has_events.store(true, Ordering::Relaxed);
        }
        result
    }
}

fn main() -> Result<()> {
    // Get data directory and initialize logging
    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir)?;

    // Initialize logging with file output
    // Keep the guard alive for the duration of the program
    let _log_guard = init_logging(&data_dir)?;

    info!("========================================");
    info!("JLiverTool v{}", env!("CARGO_PKG_VERSION"));
    info!("OS: {} ({})", std::env::consts::OS, std::env::consts::ARCH);
    info!("Data directory: {:?}", data_dir);
    info!("========================================");

    // Create channels for communication
    let (event_tx, event_rx) = mpsc::channel::<Event>();
    let (command_tx, command_rx) = mpsc::channel::<UiCommand>();

    // Create flag for pending events (shared between sender and UI)
    let has_events = Arc::new(AtomicBool::new(false));

    // Shared API client and config
    let config = Arc::new(RwLock::new(ConfigStore::new()?));
    let api = Arc::new(RwLock::new(BiliApi::new()?));

    // Initialize plugin manager (note: plugins are loaded but webview windows
    // need to be created separately due to thread safety constraints)
    let plugin_manager = Arc::new(parking_lot::Mutex::new(PluginManager::new()));
    let plugins_dir = config.read().data_dir().join("plugins");
    match plugin_manager.lock().scan_plugins_dir(&plugins_dir) {
        Ok(loaded) => {
            if !loaded.is_empty() {
                info!("Loaded {} plugins: {:?}", loaded.len(), loaded);
            }
        }
        Err(e) => {
            warn!("Failed to scan plugins directory: {}", e);
        }
    }
    // Convert plugins to UI-compatible format
    let ui_plugins: Vec<PluginInfo> = plugin_manager
        .lock()
        .get_plugins_with_paths()
        .into_iter()
        .map(|(meta, path)| PluginInfo {
            id: meta.id,
            name: meta.name,
            author: meta.author,
            desc: meta.desc,
            version: meta.version,
            enabled: true, // All loaded plugins are enabled by default
            path,
        })
        .collect();

    // Start WebSocket server for plugin communication (always start, even if no plugins yet)
    let (ws_port, plugin_event_tx) = {
        let plugin_manager_clone = plugin_manager.clone();
        let (port_tx, port_rx) = std::sync::mpsc::channel::<Option<(u16, tokio::sync::broadcast::Sender<jlivertool_plugin::PluginEvent>)>>();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime for plugin WS server");

            runtime.block_on(async move {
                // Start the WebSocket server
                let result = {
                    let mut pm = plugin_manager_clone.lock();
                    pm.start_ws_server().await
                };

                match result {
                    Ok(port) => {
                        info!("Plugin WebSocket server started on port {}", port);
                        // Get the event sender
                        let event_sender = {
                            let pm = plugin_manager_clone.lock();
                            pm.get_event_sender()
                        };
                        if let Some(sender) = event_sender {
                            let _ = port_tx.send(Some((port, sender)));
                        } else {
                            let _ = port_tx.send(None);
                        }
                        // Keep the runtime alive to maintain the WebSocket server
                        // Wait indefinitely (server runs in background)
                        loop {
                            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                        }
                    }
                    Err(e) => {
                        error!("Failed to start plugin WebSocket server: {}", e);
                        let _ = port_tx.send(None);
                    }
                }
            });
        });
        // Wait for the port and event sender to be available
        match port_rx.recv().unwrap_or(None) {
            Some((port, sender)) => (Some(port), Some(sender)),
            None => (None, None),
        }
    };

    // Create event sender with optional plugin broadcasting
    let event_sender = {
        let sender = EventSender::new(event_tx.clone(), has_events.clone());
        if let Some(plugin_tx) = plugin_event_tx {
            sender.with_plugin_sender(plugin_tx)
        } else {
            sender
        }
    };

    // Initialize database
    let db_path = config.read().data_dir().join("jlivertool.db");
    let database = Arc::new(Database::new(&db_path)?);
    info!("Database initialized at {:?}", db_path);

    // Initialize TTS manager
    let tts_manager = Arc::new(TtsManager::new());
    {
        let config_read = config.read();
        let cfg = config_read.get_config();
        tts_manager.set_enabled(TtsEnabled {
            danmu: cfg.tts_enabled,
            gift: cfg.tts_gift_enabled,
            superchat: cfg.tts_sc_enabled,
        });
        tts_manager.set_volume(cfg.tts_volume);
    }
    info!("TTS manager initialized");

    // Set cookies if available
    {
        let config_read = config.read();
        if let Some(cookies) = config_read.get_cookies() {
            api.write().set_cookies(Some(cookies));
        }
    }

    // Create channel for backend control
    let (backend_cmd_tx, backend_cmd_rx) = tokio_mpsc::unbounded_channel::<BackendCommand>();

    // Start backend in separate thread
    let event_sender_clone = event_sender.clone();
    let config_clone = config.clone();
    let api_clone = api.clone();
    let db_clone = database.clone();
    let tts_clone = tts_manager.clone();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        runtime.block_on(async move {
            if let Err(e) = run_backend(event_sender_clone, config_clone, api_clone, db_clone, tts_clone, backend_cmd_rx).await {
                error!("Backend error: {}", e);
            }
        });
    });

    // Start command handler in separate thread
    let event_sender_clone = event_sender.clone();
    let config_clone = config.clone();
    let api_clone = api.clone();
    let tts_clone = tts_manager.clone();
    let plugin_manager_clone = plugin_manager.clone();
    let db_clone_for_commands = database.clone();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        runtime.block_on(async move {
            handle_commands(command_rx, event_sender_clone, config_clone, api_clone, tts_clone, plugin_manager_clone, db_clone_for_commands, backend_cmd_tx).await;
        });
    });

    // Check initial login status in async context
    let event_sender_login = event_sender.clone();
    let config_login = config.clone();
    let api_login = api.clone();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        runtime.block_on(async move {
            check_initial_login(event_sender_login, config_login, api_login).await;
        });
    });

    // Send initial config to UI
    {
        let config_read = config.read();
        let cfg = config_read.get_config();

        // Set initial theme
        jlivertool_ui::set_theme(&cfg.theme);

        let _ = event_sender.send(Event::ConfigLoaded {
            always_on_top: cfg.always_on_top,
            guard_effect: cfg.guard_effect,
            level_effect: cfg.level_effect,
            opacity: cfg.opacity,
            lite_mode: cfg.lite_mode,
            medal_display: cfg.medal_display,
            interact_display: cfg.interact_display,
            theme: cfg.theme.clone(),
            font_size: cfg.font_size,
            tts_enabled: cfg.tts_enabled,
            tts_gift_enabled: cfg.tts_gift_enabled,
            tts_sc_enabled: cfg.tts_sc_enabled,
            tts_volume: cfg.tts_volume,
            max_danmu_count: cfg.max_danmu_count,
            log_level: cfg.log_level.clone(),
            auto_update_check: cfg.auto_update_check,
        });

        // Auto-check for updates on startup if enabled
        if cfg.auto_update_check {
            let event_tx = event_sender.clone();
            let current_version = env!("CARGO_PKG_VERSION").to_string();
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime");

                runtime.block_on(async move {
                    match jlivertool_core::check_for_update(&current_version).await {
                        Ok(update_info) => {
                            let _ = event_tx.send(Event::UpdateCheckResult {
                                has_update: update_info.has_update,
                                current_version: update_info.current_version,
                                latest_version: update_info.latest_version,
                                release_url: update_info.release_url,
                                error: None,
                            });
                        }
                        Err(e) => {
                            tracing::warn!("Auto update check failed: {}", e);
                        }
                    }
                });
            });
        }
    }

    // Run UI on main thread
    run_app_with_plugins(event_rx, command_tx, Some(database), Some(config), has_events, ui_plugins, ws_port);

    Ok(())
}

/// Check initial login status and fetch user info
async fn check_initial_login(
    event_tx: EventSender,
    config: Arc<RwLock<ConfigStore>>,
    api: Arc<RwLock<BiliApi>>,
) {
    let cookies = config.read().get_cookies();

    if cookies.is_some() {
        info!("Found saved cookies, checking login status...");

        // Use nav() to verify login and get user info
        let api_read = api.read().clone();
        match api_read.nav().await {
            Ok(nav_data) => {
                if nav_data.is_login {
                    // Build UserInfoData from NavData
                    let user_info = jlivertool_core::bilibili::api::UserInfoData {
                        mid: nav_data.mid.unwrap_or(0),
                        name: nav_data.uname.unwrap_or_default(),
                        face: nav_data.face.unwrap_or_default(),
                        sign: String::new(),
                        level: 0,
                        sex: String::new(),
                        birthday: String::new(),
                        top_photo: String::new(),
                        fans_badge: false,
                        official: Default::default(),
                        vip: Default::default(),
                        live_room: None,
                    };
                    info!("Login verified for user: {}", user_info.name);
                    let _ = event_tx.send(Event::LoginStatusChanged {
                        logged_in: true,
                        user_info: Some(user_info),
                    });
                } else {
                    warn!("Cookies exist but not logged in, clearing...");
                    // Clear invalid cookies
                    if let Err(e) = config.write().set_cookies(None) {
                        error!("Failed to clear cookies: {}", e);
                    }
                    api.write().set_cookies(None);
                    let _ = event_tx.send(Event::LoginStatusChanged {
                        logged_in: false,
                        user_info: None,
                    });
                }
            }
            Err(e) => {
                warn!("Failed to verify login: {}", e);
                let _ = event_tx.send(Event::LoginStatusChanged {
                    logged_in: false,
                    user_info: None,
                });
            }
        }
    }
}

/// Handle UI commands
async fn handle_commands(
    command_rx: mpsc::Receiver<UiCommand>,
    event_tx: EventSender,
    config: Arc<RwLock<ConfigStore>>,
    api: Arc<RwLock<BiliApi>>,
    tts_manager: Arc<TtsManager>,
    plugin_manager: Arc<parking_lot::Mutex<PluginManager>>,
    database: Arc<Database>,
    backend_cmd_tx: tokio_mpsc::UnboundedSender<BackendCommand>,
) {
    while let Ok(command) = command_rx.recv() {
        match command {
            UiCommand::RequestQrLogin => {
                info!("Starting QR login...");
                let api_read = api.read().clone();

                // Generate QR code
                match api_read.qr_generate().await {
                    Ok(qr_data) => {
                        info!("QR code generated: {}", qr_data.url);
                        let _ = event_tx.send(Event::QrCodeGenerated {
                            url: qr_data.url.clone(),
                            qrcode_key: qr_data.qrcode_key.clone(),
                        });

                        // Start polling for QR login status
                        let qrcode_key = qr_data.qrcode_key;
                        let event_tx = event_tx.clone();
                        let config = config.clone();
                        let api = api.clone();

                        tokio::spawn(async move {
                            poll_qr_login(qrcode_key, event_tx, config, api).await;
                        });
                    }
                    Err(e) => {
                        error!("Failed to generate QR code: {}", e);
                    }
                }
            }
            UiCommand::RequestLogout => {
                info!("Logging out...");
                let api_read = api.read().clone();

                // Try to logout on server
                if let Err(e) = api_read.logout().await {
                    warn!("Logout API failed: {}", e);
                }

                // Clear local cookies
                if let Err(e) = config.write().set_cookies(None) {
                    error!("Failed to clear cookies: {}", e);
                }

                // Clear API cookies
                api.write().set_cookies(None);

                // Notify UI
                let _ = event_tx.send(Event::LoginStatusChanged {
                    logged_in: false,
                    user_info: None,
                });

                info!("Logged out successfully");
            }
            UiCommand::ChangeRoom(room_id) => {
                info!("Changing room to {}", room_id);
                let api_read = api.read().clone();

                // Validate and get room info
                match api_read.get_room(room_id).await {
                    Ok(room) => {
                        // Save room to config
                        if let Err(e) = config.write().set_room(room.clone()) {
                            error!("Failed to save room config: {}", e);
                        }

                        // Get room info for display
                        match api_read.get_room_info(room.real_id()).await {
                            Ok(room_info) => {
                                // Clear danmu list first
                                let _ = event_tx.send(Event::ClearDanmuList);

                                // Notify UI about room update
                                let _ = event_tx.send(Event::UpdateRoom {
                                    room_id: room.clone(),
                                    title: room_info.title,
                                    live_status: room_info.live_status,
                                });

                                // Signal backend to change room
                                let _ = backend_cmd_tx.send(BackendCommand::ChangeRoom(room));

                                info!("Room changed successfully");
                            }
                            Err(e) => {
                                warn!("Failed to get room info: {}", e);
                                // Still update with basic info
                                let _ = event_tx.send(Event::ClearDanmuList);
                                let _ = event_tx.send(Event::UpdateRoom {
                                    room_id: room.clone(),
                                    title: String::new(),
                                    live_status: 0,
                                });
                                let _ = backend_cmd_tx.send(BackendCommand::ChangeRoom(room));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Invalid room ID {}: {}", room_id, e);
                    }
                }
            }
            UiCommand::SendDanmu { room_id, message } => {
                info!("Sending danmu to room {}: {}", room_id, message);
                let api_read = api.read().clone();

                match api_read.send_danmu(room_id, &message, 1, 16777215, 25).await {
                    Ok(_) => {
                        info!("Danmu sent successfully");
                    }
                    Err(e) => {
                        error!("Failed to send danmu: {}", e);
                    }
                }
            }
            UiCommand::UpdateRoomTitle { room_id, title } => {
                info!("Updating room {} title to: {}", room_id, title);
                let api_read = api.read().clone();

                match api_read.update_room_title(room_id, &title).await {
                    Ok(_) => {
                        info!("Room title updated successfully");
                    }
                    Err(e) => {
                        error!("Failed to update room title: {}", e);
                    }
                }
            }
            UiCommand::StopLive { room_id } => {
                info!("Stopping live for room {}", room_id);
                let api_read = api.read().clone();

                match api_read.stop_live(room_id).await {
                    Ok(_) => {
                        info!("Live stopped successfully");
                    }
                    Err(e) => {
                        error!("Failed to stop live: {}", e);
                    }
                }
            }
            UiCommand::StartLive { room_id, area_v2 } => {
                info!("Starting live for room {} with area {}", room_id, area_v2);
                let api_read = api.read().clone();

                match api_read.start_live(room_id, area_v2).await {
                    Ok(data) => {
                        info!("Live started successfully");

                        if let Some(rtmp) = data.get("rtmp") {
                            if let (Some(addr), Some(code)) = (
                                rtmp.get("addr").and_then(|v| v.as_str()),
                                rtmp.get("code").and_then(|v| v.as_str()),
                            ) {
                                let _ = event_tx.send(Event::RtmpInfo {
                                    addr: addr.to_string(),
                                    code: code.to_string(),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to start live: {}", e);
                    }
                }
            }
            UiCommand::UpdateWindowSettings {
                lite_mode,
                medal_display,
                interact_display,
                guard_effect,
                level_effect,
            } => {
                info!(
                    "Updating window settings: lite_mode={}, medal_display={}, interact_display={}, guard_effect={}, level_effect={}",
                    lite_mode, medal_display, interact_display, guard_effect, level_effect
                );

                let config_write = config.write();
                if let Err(e) = config_write.set("lite_mode", lite_mode) {
                    error!("Failed to save lite_mode: {}", e);
                }
                if let Err(e) = config_write.set("medal_display", medal_display) {
                    error!("Failed to save medal_display: {}", e);
                }
                if let Err(e) = config_write.set("interact_display", interact_display) {
                    error!("Failed to save interact_display: {}", e);
                }
                if let Err(e) = config_write.set("guard_effect", guard_effect) {
                    error!("Failed to save guard_effect: {}", e);
                }
                if let Err(e) = config_write.set("level_effect", level_effect) {
                    error!("Failed to save level_effect: {}", e);
                }
            }
            UiCommand::UpdateOpacity(opacity) => {
                info!("Updating opacity to {}", opacity);

                let config_write = config.write();
                if let Err(e) = config_write.set("opacity", opacity) {
                    error!("Failed to save opacity: {}", e);
                }
            }
            UiCommand::UpdateTheme(theme) => {
                info!("Updating theme to {}", theme);

                // Update the UI theme
                jlivertool_ui::set_theme(&theme);

                let config_write = config.write();
                if let Err(e) = config_write.set("theme", &theme) {
                    error!("Failed to save theme: {}", e);
                }
            }
            UiCommand::UpdateFontSize(font_size) => {
                info!("Updating font size to {}", font_size);

                let config_write = config.write();
                if let Err(e) = config_write.set("font_size", font_size) {
                    error!("Failed to save font_size: {}", e);
                }
            }
            UiCommand::UpdateAlwaysOnTop(always_on_top) => {
                info!("Updating always on top to {}", always_on_top);

                let config_write = config.write();
                if let Err(e) = config_write.set("always_on_top", always_on_top) {
                    error!("Failed to save always_on_top: {}", e);
                }
            }
            UiCommand::FetchUserInfo { uid } => {
                info!("Fetching user info for UID {}", uid);
                let api_read = api.read().clone();
                let event_tx = event_tx.clone();

                tokio::spawn(async move {
                    match api_read.get_user_info(uid).await {
                        Ok(user_info) => {
                            info!("User info fetched for {}: {}", uid, user_info.name);
                            let _ = event_tx.send(Event::UserInfoFetched { uid, user_info });
                        }
                        Err(e) => {
                            error!("Failed to fetch user info for {}: {}", uid, e);
                        }
                    }
                });
            }
            UiCommand::FetchAudienceList { room_id, ruid } => {
                info!("Fetching audience list for room {}", room_id);
                let api_read = api.read().clone();
                let event_tx = event_tx.clone();

                tokio::spawn(async move {
                    match api_read.get_online_gold_rank(room_id, ruid, 1, 50).await {
                        Ok(data) => {
                            info!("Audience list fetched: {} users", data.item.len());
                            let _ = event_tx.send(Event::AudienceListFetched { list: data.item });
                        }
                        Err(e) => {
                            error!("Failed to fetch audience list: {}", e);
                        }
                    }
                });
            }
            UiCommand::FetchGuardList { room_id, ruid, page } => {
                info!("Fetching guard list for room {} page {}", room_id, page);
                let api_read = api.read().clone();
                let event_tx = event_tx.clone();

                tokio::spawn(async move {
                    match api_read.get_guard_list(room_id, ruid, page).await {
                        Ok(data) => {
                            // Combine top3 and list
                            let mut combined_list = data.top3;
                            combined_list.extend(data.list);
                            info!("Guard list fetched: {} guards (total: {})", combined_list.len(), data.info.num);
                            let _ = event_tx.send(Event::GuardListFetched {
                                list: combined_list,
                                total: data.info.num,
                                page,
                            });
                        }
                        Err(e) => {
                            error!("Failed to fetch guard list: {}", e);
                        }
                    }
                });
            }
            UiCommand::SaveWindowBounds {
                window_type,
                x,
                y,
                width,
                height,
            } => {
                let window_config = jlivertool_core::config::WindowConfig {
                    x,
                    y,
                    width,
                    height,
                };
                if let Err(e) = config.write().set_window_config(window_type, window_config) {
                    error!("Failed to save window bounds: {}", e);
                }
            }
            UiCommand::UpdateTtsEnabled { danmu, gift, superchat } => {
                info!("Updating TTS enabled: danmu={}, gift={}, superchat={}", danmu, gift, superchat);
                tts_manager.set_enabled(TtsEnabled {
                    danmu,
                    gift,
                    superchat,
                });
                // Save to config
                let config_write = config.write();
                if let Err(e) = config_write.set("tts_enabled", danmu) {
                    error!("Failed to save tts_enabled: {}", e);
                }
                if let Err(e) = config_write.set("tts_gift_enabled", gift) {
                    error!("Failed to save tts_gift_enabled: {}", e);
                }
                if let Err(e) = config_write.set("tts_sc_enabled", superchat) {
                    error!("Failed to save tts_sc_enabled: {}", e);
                }
            }
            UiCommand::UpdateTtsVolume(volume) => {
                info!("Updating TTS volume to {}", volume);
                tts_manager.set_volume(volume);
                if let Err(e) = config.write().set("tts_volume", volume) {
                    error!("Failed to save tts_volume: {}", e);
                }
            }
            UiCommand::UpdateTtsProvider(provider) => {
                info!("Updating TTS provider to {}", provider);
                if let Err(e) = config.write().set("tts_provider", &provider) {
                    error!("Failed to save tts_provider: {}", e);
                }
            }
            UiCommand::TestTts => {
                info!("Testing TTS");
                tts_manager.test();
            }
            UiCommand::RefreshPlugins => {
                info!("Refreshing plugins list");
                let plugins_dir = config.read().data_dir().join("plugins");
                info!("Plugins directory: {:?}", plugins_dir);

                // Rescan plugins directory
                let pm = plugin_manager.lock();
                // Clear existing plugins and rescan
                pm.clear_plugins();
                match pm.scan_plugins_dir(&plugins_dir) {
                    Ok(loaded) => {
                        info!("Refreshed plugins: {:?}", loaded);
                    }
                    Err(e) => {
                        warn!("Failed to scan plugins directory: {}", e);
                    }
                }

                // Send updated plugin list to UI
                let plugin_events: Vec<jlivertool_core::events::PluginInfoEvent> = pm
                    .get_plugins_with_paths()
                    .into_iter()
                    .map(|(meta, path)| jlivertool_core::events::PluginInfoEvent {
                        id: meta.id,
                        name: meta.name,
                        author: meta.author,
                        desc: meta.desc,
                        version: meta.version,
                        path,
                    })
                    .collect();
                info!("Sending {} plugins to UI", plugin_events.len());

                let _ = event_tx.send(Event::PluginsRefreshed { plugins: plugin_events });
            }
            UiCommand::ImportPlugin(github_url) => {
                info!("Importing plugin from GitHub: {}", github_url);
                let plugins_dir = config.read().data_dir().join("plugins");
                let pm = plugin_manager.clone();
                let event_tx = event_tx.clone();

                tokio::spawn(async move {
                    // Download plugin files (this is the async part)
                    let result = jlivertool_plugin::PluginManager::download_plugin_from_github(
                        &github_url,
                        &plugins_dir,
                    )
                    .await;

                    match result {
                        Ok(plugin_dir) => {
                            // Load the plugin (sync, needs lock)
                            let load_result = pm.lock().load_plugin(plugin_dir);
                            match load_result {
                                Ok(plugin_id) => {
                                    info!("Successfully imported plugin: {}", plugin_id);
                                    let _ = event_tx.send(Event::PluginImportResult {
                                        success: true,
                                        message: format!("插件 {} 导入成功", plugin_id),
                                    });

                                    // Refresh plugins list
                                    let pm = pm.lock();
                                    let plugin_events: Vec<jlivertool_core::events::PluginInfoEvent> = pm
                                        .get_plugins_with_paths()
                                        .into_iter()
                                        .map(|(meta, path)| jlivertool_core::events::PluginInfoEvent {
                                            id: meta.id,
                                            name: meta.name,
                                            author: meta.author,
                                            desc: meta.desc,
                                            version: meta.version,
                                            path,
                                        })
                                        .collect();
                                    let _ = event_tx.send(Event::PluginsRefreshed { plugins: plugin_events });
                                }
                                Err(e) => {
                                    error!("Failed to load plugin: {}", e);
                                    let _ = event_tx.send(Event::PluginImportResult {
                                        success: false,
                                        message: format!("加载插件失败: {}", e),
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to download plugin: {}", e);
                            let _ = event_tx.send(Event::PluginImportResult {
                                success: false,
                                message: format!("下载失败: {}", e),
                            });
                        }
                    }
                });
            }
            UiCommand::RemovePlugin { plugin_id, plugin_path } => {
                info!("Removing plugin: {} at {:?}", plugin_id, plugin_path);
                let pm = plugin_manager.lock();

                match pm.remove_plugin(&plugin_id, &plugin_path) {
                    Ok(()) => {
                        info!("Successfully removed plugin: {}", plugin_id);

                        // Send updated plugin list to UI
                        let plugin_events: Vec<jlivertool_core::events::PluginInfoEvent> = pm
                            .get_plugins_with_paths()
                            .into_iter()
                            .map(|(meta, path)| jlivertool_core::events::PluginInfoEvent {
                                id: meta.id,
                                name: meta.name,
                                author: meta.author,
                                desc: meta.desc,
                                version: meta.version,
                                path,
                            })
                            .collect();
                        let _ = event_tx.send(Event::PluginsRefreshed { plugins: plugin_events });
                    }
                    Err(e) => {
                        error!("Failed to remove plugin: {}", e);
                    }
                }
            }
            UiCommand::UpdateAdvancedSettings { max_danmu, log_level } => {
                info!("Updating advanced settings: max_danmu={}, log_level={}", max_danmu, log_level);
                let config_write = config.write();
                if let Err(e) = config_write.set("max_danmu_count", max_danmu) {
                    error!("Failed to save max_danmu_count: {}", e);
                }
                if let Err(e) = config_write.set("log_level", &log_level) {
                    error!("Failed to save log_level: {}", e);
                }
            }
            UiCommand::ClearAllData => {
                info!("Clearing all data");
                // Get current room ID from config
                let room_id = config.read().get_room().map(|r| r.real_id());
                if let Some(room_id) = room_id {
                    // Clear all data for the current room
                    if let Err(e) = database.clear_room_data(room_id) {
                        error!("Failed to clear room data: {}", e);
                    } else {
                        info!("Room data cleared successfully");
                        let _ = event_tx.send(Event::DataCleared);
                    }
                }
            }
            UiCommand::OpenDataFolder => {
                info!("Opening data folder");
                let data_dir = config.read().data_dir();
                // Open the folder using the system's default file manager
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open")
                        .arg(&data_dir)
                        .spawn();
                }
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer")
                        .arg(&data_dir)
                        .spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&data_dir)
                        .spawn();
                }
            }
            UiCommand::CheckForUpdate => {
                info!("Checking for updates...");
                let event_tx = event_tx.clone();
                let current_version = env!("CARGO_PKG_VERSION").to_string();
                tokio::spawn(async move {
                    match jlivertool_core::check_for_update(&current_version).await {
                        Ok(update_info) => {
                            let _ = event_tx.send(Event::UpdateCheckResult {
                                has_update: update_info.has_update,
                                current_version: update_info.current_version,
                                latest_version: update_info.latest_version,
                                release_url: update_info.release_url,
                                error: None,
                            });
                        }
                        Err(e) => {
                            let _ = event_tx.send(Event::UpdateCheckResult {
                                has_update: false,
                                current_version,
                                latest_version: String::new(),
                                release_url: String::new(),
                                error: Some(e.to_string()),
                            });
                        }
                    }
                });
            }
            UiCommand::UpdateAutoUpdateCheck(enabled) => {
                info!("Updating auto update check setting: {}", enabled);
                if let Err(e) = config.write().set("auto_update_check", enabled) {
                    error!("Failed to save auto_update_check: {}", e);
                }
            }
        }
    }
}

/// Poll QR login status
async fn poll_qr_login(
    qrcode_key: String,
    event_tx: EventSender,
    config: Arc<RwLock<ConfigStore>>,
    api: Arc<RwLock<BiliApi>>,
) {
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 60; // 2 minutes timeout

    loop {
        attempts += 1;
        if attempts > MAX_ATTEMPTS {
            let _ = event_tx.send(Event::QrLoginStatus {
                status: QrCodeStatus::Expired,
            });
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let api_read = api.read().clone();
        match api_read.qr_poll(&qrcode_key).await {
            Ok((status, cookies_opt)) => {
                let _ = event_tx.send(Event::QrLoginStatus { status });

                match status {
                    QrCodeStatus::Success => {
                        if let Some(cookies) = cookies_opt {
                            info!("QR login successful!");

                            // Save cookies
                            if let Err(e) = config.write().set_cookies(Some(cookies.clone())) {
                                error!("Failed to save cookies: {}", e);
                            }

                            // Update API cookies
                            api.write().set_cookies(Some(cookies.clone()));

                            // Fetch user info using nav() with new cookies
                            let api_with_cookies = api.read().clone();
                            match api_with_cookies.nav().await {
                                Ok(nav_data) => {
                                    if nav_data.is_login {
                                        let user_info =
                                            jlivertool_core::bilibili::api::UserInfoData {
                                                mid: nav_data.mid.unwrap_or(0),
                                                name: nav_data.uname.unwrap_or_default(),
                                                face: nav_data.face.unwrap_or_default(),
                                                sign: String::new(),
                                                level: 0,
                                                sex: String::new(),
                                                birthday: String::new(),
                                                top_photo: String::new(),
                                                fans_badge: false,
                                                official: Default::default(),
                                                vip: Default::default(),
                                                live_room: None,
                                            };
                                        let _ = event_tx.send(Event::LoginStatusChanged {
                                            logged_in: true,
                                            user_info: Some(user_info),
                                        });
                                    } else {
                                        let _ = event_tx.send(Event::LoginStatusChanged {
                                            logged_in: true,
                                            user_info: None,
                                        });
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to get user info: {}", e);
                                    let _ = event_tx.send(Event::LoginStatusChanged {
                                        logged_in: true,
                                        user_info: None,
                                    });
                                }
                            }
                        }
                        break;
                    }
                    QrCodeStatus::Expired | QrCodeStatus::Error => {
                        break;
                    }
                    _ => {
                        // Continue polling
                    }
                }
            }
            Err(e) => {
                error!("Failed to poll QR status: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }
}

/// Run the backend service
async fn run_backend(
    event_tx: EventSender,
    config: Arc<RwLock<ConfigStore>>,
    api: Arc<RwLock<BiliApi>>,
    database: Arc<Database>,
    tts_manager: Arc<TtsManager>,
    mut backend_cmd_rx: tokio_mpsc::UnboundedReceiver<BackendCommand>,
) -> Result<()> {
    // Get initial room to connect
    let initial_room = config
        .read()
        .get_room()
        .unwrap_or_else(|| RoomId::new(0, 21484828, 0)); // Default room

    let mut current_room = initial_room;

    loop {
        // Connect to current room
        info!("Connecting to room {}", current_room.real_id());

        let api_read = api.read().clone();

        // Get room info
        match api_read.get_room_info(current_room.real_id()).await {
            Ok(room_info) => {
                info!(
                    "Room: {} ({}), Status: {}",
                    room_info.title,
                    current_room.real_id(),
                    room_info.live_status
                );

                // Emit room update event
                let _ = event_tx.send(Event::UpdateRoom {
                    room_id: current_room.clone(),
                    title: room_info.title.clone(),
                    live_status: room_info.live_status,
                });
            }
            Err(e) => {
                warn!("Failed to get room info: {}", e);
            }
        }

        // Get danmu info for WebSocket connection
        let danmu_info = match api_read.get_danmu_info(current_room.real_id()).await {
            Ok(info) => info,
            Err(e) => {
                error!("Failed to get danmu info: {}", e);
                // Wait and retry
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        // Select best host
        let host = match danmu_info.host_list.first() {
            Some(h) => h,
            None => {
                error!("No WebSocket host available");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        let ws_info = WsInfo {
            server: format!("wss://{}:{}/sub", host.host, host.wss_port),
            room_id: current_room.real_id(),
            uid: config
                .read()
                .get_cookies()
                .map(|c| c.dede_user_id.parse().unwrap_or(0))
                .unwrap_or(0),
            token: danmu_info.token,
        };

        // Create WebSocket connection
        let (ws, mut ws_event_rx) = BiliWebSocket::new(ws_info);

        // Spawn WebSocket connection task
        let ws_handle = tokio::spawn(async move {
            if let Err(e) = ws.connect().await {
                error!("WebSocket connection error: {}", e);
            }
        });

        // Handle WebSocket events and backend commands
        let event_tx_clone = event_tx.clone();
        let db_clone = database.clone();
        let tts_clone = tts_manager.clone();
        let room_id = current_room.real_id();

        let mut should_reconnect = false;
        let mut new_room: Option<RoomId> = None;

        loop {
            tokio::select! {
                // Handle WebSocket events
                event = ws_event_rx.recv() => {
                    match event {
                        Some(WsEvent::Connected) => {
                            info!("WebSocket connected");
                            let _ = event_tx_clone.send(Event::ConnectionStatus { connected: true });
                        }
                        Some(WsEvent::Authenticated) => {
                            info!("WebSocket authenticated");
                        }
                        Some(WsEvent::HeartbeatReply(count)) => {
                            // Only update online count from heartbeat if it's a reasonable value
                            // Heartbeat can return 1 when there's no valid data
                            if count > 1 {
                                let _ = event_tx_clone.send(Event::UpdateOnline {
                                    count: count as u64,
                                });
                            }
                        }
                        Some(WsEvent::Message(body)) => {
                            if let Some(cmd) = body.get("cmd").and_then(|v| v.as_str()) {
                                handle_message(cmd, &body, room_id, &event_tx_clone, &db_clone, &tts_clone);
                            }
                        }
                        Some(WsEvent::Disconnected) => {
                            info!("WebSocket disconnected");
                            let _ = event_tx_clone.send(Event::ConnectionStatus { connected: false });
                            // Reconnect to same room
                            should_reconnect = true;
                            break;
                        }
                        Some(WsEvent::Error(err)) => {
                            error!("WebSocket error: {}", err);
                        }
                        None => {
                            // Channel closed, reconnect
                            should_reconnect = true;
                            break;
                        }
                    }
                }
                // Handle backend commands
                cmd = backend_cmd_rx.recv() => {
                    match cmd {
                        Some(BackendCommand::ChangeRoom(room)) => {
                            info!("Received room change command to {}", room.real_id());
                            new_room = Some(room);
                            break;
                        }
                        Some(BackendCommand::Stop) => {
                            info!("Received stop command");
                            ws_handle.abort();
                            return Ok(());
                        }
                        None => {
                            // Command channel closed, stop
                            ws_handle.abort();
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Abort the WebSocket task
        ws_handle.abort();

        // Update current room if changed
        if let Some(room) = new_room {
            current_room = room;
        } else if should_reconnect {
            // Wait before reconnecting
            info!("Reconnecting in 5 seconds...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}

/// Handle incoming WebSocket message
fn handle_message(
    cmd: &str,
    body: &serde_json::Value,
    room_id: u64,
    event_tx: &EventSender,
    database: &Arc<Database>,
    tts_manager: &Arc<TtsManager>,
) {
    let base_cmd = cmd.split(':').next().unwrap_or(cmd);

    match base_cmd {
        "DANMU_MSG" => {
            if let Some(danmu) = DanmuMessage::from_raw(body, None) {
                // Store in database (only non-generated messages)
                if !danmu.is_generated {
                    if let Err(e) = database.insert_danmu(room_id, &danmu) {
                        warn!("Failed to store danmu: {}", e);
                    }
                }
                // TTS for danmu
                tts_manager.speak(TtsMessage::danmu(&danmu.sender.uname, &danmu.content));
                let _ = event_tx.send(Event::NewDanmu(danmu));
            }
        }
        "SEND_GIFT" => {
            if let Some(gift) = GiftMessage::from_raw(body, room_id) {
                // Store in database
                if let Err(e) = database.insert_gift(&gift) {
                    warn!("Failed to store gift: {}", e);
                }
                // TTS for gift
                tts_manager.speak(TtsMessage::gift(&gift.sender.uname, &gift.gift_info.name, gift.num));
                let _ = event_tx.send(Event::NewGift(gift));
            }
        }
        "USER_TOAST_MSG" => {
            if let Some(guard) = GuardMessage::from_raw(body, room_id) {
                info!(
                    "New guard: {} bought {} x{} (¥{:.2})",
                    guard.sender.uname,
                    jlivertool_core::types::guard_level_name(guard.guard_level),
                    guard.num,
                    guard.price as f64 / 1000.0
                );
                // Store in database
                if let Err(e) = database.insert_guard(&guard) {
                    warn!("Failed to store guard: {}", e);
                }
                let _ = event_tx.send(Event::NewGuard(guard));
            }
        }
        "SUPER_CHAT_MESSAGE" => {
            if let Some(sc) = SuperChatMessage::from_raw(body, room_id) {
                info!(
                    "New superchat: {} sent ¥{} - {}",
                    sc.sender.uname,
                    sc.price,
                    sc.message
                );
                // Store in database
                if let Err(e) = database.insert_superchat(&sc) {
                    warn!("Failed to store superchat: {}", e);
                }
                // TTS for superchat
                tts_manager.speak(TtsMessage::superchat(&sc.sender.uname, sc.price, &sc.message));
                let _ = event_tx.send(Event::NewSuperChat(sc));
            }
        }
        "INTERACT_WORD" => {
            if let Some(interact) = InteractMessage::from_raw(body) {
                let _ = event_tx.send(Event::NewInteract(interact));
            }
        }
        "ENTRY_EFFECT" => {
            if let Some(entry) = EntryEffectMessage::from_raw(body) {
                let _ = event_tx.send(Event::NewEntryEffect(entry));
            }
        }
        "ROOM_CHANGE" => {
            if let Some(room_change) = RoomChangeMessage::from_raw(body) {
                info!("Room changed: title={}, area={}/{}", room_change.title, room_change.parent_area_name, room_change.area_name);
                let _ = event_tx.send(Event::RoomChange(room_change));
            }
        }
        "ONLINE_RANK_COUNT" | "ONLINE_RANK_V2" => {
            if let Some(rank) = OnlineRankCountMessage::from_raw(body) {
                let _ = event_tx.send(Event::UpdateOnline { count: rank.count });
            }
        }
        "LIVE" => {
            let _ = event_tx.send(Event::LiveStart);
        }
        "PREPARING" => {
            let _ = event_tx.send(Event::LiveEnd);
        }
        _ => {}
    }
}
