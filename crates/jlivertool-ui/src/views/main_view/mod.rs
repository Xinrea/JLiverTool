//! Main window view
//!
//! This module contains the main window view and its components:
//! - `content_rendering` - BV link rendering, guard icons, DisplayMessage
//! - `user_info_card` - User info card popup component
//! - `danmu_list_item` - Unified danmu list item rendering
//! - `event_processing` - Event handling logic
//! - `render` - Render methods and Render trait implementation

mod content_rendering;
mod danmu_list_item;
mod event_processing;
mod render;
mod user_info_card;

pub use content_rendering::{render_content_with_links, DisplayMessage, RenderRow};
use content_rendering::{estimate_danmu_prefix_width, estimate_text_width, split_content_to_lines};
use danmu_list_item::DanmuListItemView;
use user_info_card::{SelectedUserState, UserInfoCard};

use crate::app::UiCommand;
use crate::tray::{TrayManager, TrayState};
use crate::views::AudienceView;
use crate::views::GiftView;
use crate::views::SettingView;
use crate::views::StatisticsView;
use crate::views::SuperChatView;
use gpui::*;
use jlivertool_core::database::Database;
use jlivertool_core::events::Event;
use jlivertool_core::types::RoomId;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

const MAX_DANMU_COUNT: usize = 200;

/// Available commands for the command autocomplete
pub const AVAILABLE_COMMANDS: &[(&str, &str)] = &[
    ("/title", "修改直播间标题"),
    ("/bye", "关闭直播"),
    #[cfg(debug_assertions)]
    ("/debug", "调试: sc/gift/guard/danmu"),
];

/// Main window view state
pub struct MainView {
    event_rx: mpsc::Receiver<Event>,
    #[allow(dead_code)] // Used indirectly through closures
    command_tx: mpsc::Sender<UiCommand>,
    /// Flag indicating there are pending events to process (used by timer)
    #[allow(dead_code)]
    has_events: Arc<AtomicBool>,
    room: Option<RoomId>,
    room_title: String,
    live_status: u8,
    area_id: u64,
    online_count: u64,
    connected: bool,
    danmu_list: VecDeque<DisplayMessage>,
    /// Flattened render rows for the uniform_list (1 source message → 1-2 rows)
    render_rows: Rc<Vec<RenderRow>>,
    /// Last window width used to compute render_rows (for change detection)
    last_render_width: f32,
    /// Number of source danmu_list items processed into render_rows (for incremental append)
    render_rows_source_count: usize,
    /// Deferred scroll-to-bottom (applied after render_rows are rebuilt)
    pending_scroll_to_bottom: bool,
    // Settings view entity
    setting_view: Entity<SettingView>,
    // Gift view entity
    gift_view: Entity<GiftView>,
    // SuperChat view entity
    superchat_view: Entity<SuperChatView>,
    // Statistics view entity
    statistics_view: Entity<StatisticsView>,
    // Audience view entity
    audience_view: Entity<AudienceView>,
    // Database reference for statistics
    database: Option<Arc<Database>>,
    // Config store reference for window bounds
    config: Option<Arc<parking_lot::RwLock<jlivertool_core::config::ConfigStore>>>,
    // Window opacity
    opacity: f32,
    // Danmu font size
    font_size: f32,
    // Window display settings
    lite_mode: bool,
    medal_display: bool,
    interact_display: bool,
    guard_effect: bool,
    level_effect: bool,
    // Always on top
    always_on_top: bool,
    // Flag to apply always_on_top on next render
    pending_always_on_top: Option<bool>,
    // Click-through mode
    click_through: bool,
    // Pending click-through change from tray (Arc for thread-safe sharing)
    pending_click_through: Arc<AtomicBool>,
    // Scroll handle for danmu list
    scroll_handle: UniformListScrollHandle,
    // Window handles for single-instance windows
    settings_window: Option<AnyWindowHandle>,
    gift_window: Option<AnyWindowHandle>,
    superchat_window: Option<AnyWindowHandle>,
    statistics_window: Option<AnyWindowHandle>,
    audience_window: Option<AnyWindowHandle>,
    // Input state for danmu input (lazily initialized)
    input_state: Option<Entity<gpui_component::input::InputState>>,
    // Subscription for input events (must be kept alive)
    _input_subscription: Option<Subscription>,
    // Flag to clear input on next render (using Rc<Cell> for sharing with callback)
    pending_input_clear: Rc<Cell<bool>>,
    // Selected user for info card popup
    selected_user: SelectedUserState,
    // Last saved window bounds (to avoid saving on every frame)
    last_saved_bounds: Option<(i32, i32, u32, u32)>,
    // WebSocket port for plugin communication
    ws_port: Option<u16>,
    // HTTP port for serving plugin files
    http_port: Option<u16>,
    // Command autocomplete state
    show_command_popup: Rc<Cell<bool>>,
    selected_command_index: Rc<Cell<usize>>,
    // Pending command to insert (set by subscription, applied in render)
    pending_command_insert: Rc<RefCell<Option<String>>>,
    // Update dialog state
    show_update_dialog: bool,
    update_info: Option<UpdateDialogInfo>,
    // Tray manager for system tray integration
    tray_manager: Option<Arc<parking_lot::Mutex<TrayManager>>>,
    // Login status for tray
    logged_in: bool,
    // Logged-in user's UID (to check if user owns the room)
    logged_in_uid: Option<u64>,
    // Pending tray command to open settings (Arc for thread-safe sharing)
    pending_open_settings: Arc<AtomicBool>,
}

/// Update dialog information
#[derive(Clone)]
pub struct UpdateDialogInfo {
    pub latest_version: String,
    pub release_url: String,
}

impl MainView {
    pub fn new(
        event_rx: mpsc::Receiver<Event>,
        command_tx: mpsc::Sender<UiCommand>,
        has_events: Arc<AtomicBool>,
        cx: &mut Context<Self>,
    ) -> Self {
        let setting_view = cx.new(SettingView::new);
        let gift_view = cx.new(GiftView::new);
        let superchat_view = cx.new(SuperChatView::new);
        let statistics_view = cx.new(StatisticsView::new);
        let audience_view = cx.new(AudienceView::new);

        // Setup callbacks for setting view
        let tx_login = command_tx.clone();
        let tx_logout = command_tx.clone();
        let tx_room = command_tx.clone();
        let tx_opacity = command_tx.clone();

        // Get entity for opacity callback
        let entity = cx.entity().downgrade();
        let entity_gift = entity.clone();
        let entity_sc = entity.clone();
        let entity_stats = entity.clone();

        setting_view.update(cx, |view, _cx| {
            view.on_qr_login(move |_window, _cx| {
                let _ = tx_login.send(UiCommand::RequestQrLogin);
            });

            view.on_logout(move |_window, _cx| {
                let _ = tx_logout.send(UiCommand::RequestLogout);
            });

            view.on_change_room(move |room_id, _window, _cx| {
                let _ = tx_room.send(UiCommand::ChangeRoom(room_id));
            });

            view.on_opacity_change({
                let entity = entity.clone();
                move |opacity, _window, cx| {
                    // Send command to persist opacity
                    let _ = tx_opacity.send(UiCommand::UpdateOpacity(opacity));
                    let _ = entity.update(cx, |view, cx| {
                        view.opacity = opacity;
                        // Update opacity for all secondary views
                        view.gift_view
                            .update(cx, |v, cx| v.set_opacity(opacity, cx));
                        view.superchat_view
                            .update(cx, |v, cx| v.set_opacity(opacity, cx));
                        view.statistics_view
                            .update(cx, |v, cx| v.set_opacity(opacity, cx));
                        view.audience_view
                            .update(cx, |v, cx| v.set_opacity(opacity, cx));
                        cx.notify();
                    });
                }
            });

            view.on_open_gift_window(move |window, cx| {
                let _ = entity_gift.update(cx, |view, cx| {
                    view.open_gift_window(window, cx);
                });
            });

            view.on_open_superchat_window(move |window, cx| {
                let _ = entity_sc.update(cx, |view, cx| {
                    view.open_superchat_window(window, cx);
                });
            });

            view.on_open_statistics_window(move |window, cx| {
                let _ = entity_stats.update(cx, |view, cx| {
                    view.open_statistics_window(window, cx);
                });
            });

            view.on_start_live({
                let tx = command_tx.clone();
                move |room_id, area_id, _window, _cx| {
                    let _ = tx.send(UiCommand::StartLive {
                        room_id,
                        area_v2: area_id,
                    });
                }
            });

            view.on_stop_live({
                let tx = command_tx.clone();
                move |room_id, _window, _cx| {
                    let _ = tx.send(UiCommand::StopLive { room_id });
                }
            });

            view.on_window_settings_change({
                let tx = command_tx.clone();
                let entity = entity.clone();
                move |lite_mode,
                      medal_display,
                      interact_display,
                      guard_effect,
                      level_effect,
                      _window,
                      cx| {
                    let _ = tx.send(UiCommand::UpdateWindowSettings {
                        lite_mode,
                        medal_display,
                        interact_display,
                        guard_effect,
                        level_effect,
                    });
                    // Update local settings and clean up messages if settings are disabled
                    let _ = entity.update(cx, |view, cx| {
                        let old_interact_display = view.interact_display;
                        let old_guard_effect = view.guard_effect;
                        let old_level_effect = view.level_effect;

                        view.lite_mode = lite_mode;
                        view.medal_display = medal_display;
                        view.interact_display = interact_display;
                        view.guard_effect = guard_effect;
                        view.level_effect = level_effect;

                        // Force rebuild of render rows on layout-affecting changes
                        view.last_render_width = 0.0;
                        view.render_rows_source_count = 0;

                        // Remove interact messages if interact_display was disabled
                        if old_interact_display && !interact_display {
                            view.danmu_list
                                .retain(|msg| !matches!(msg, DisplayMessage::Interact(_)));
                        }

                        // Remove entry effect messages based on settings changes
                        if (old_guard_effect && !guard_effect)
                            || (old_level_effect && !level_effect)
                        {
                            view.danmu_list.retain(|msg| {
                                if let DisplayMessage::EntryEffect(entry) = msg {
                                    let is_guard =
                                        entry.privilege_type >= 1 && entry.privilege_type <= 3;
                                    if is_guard {
                                        guard_effect // Keep if guard_effect is still enabled
                                    } else {
                                        level_effect // Keep if level_effect is still enabled
                                    }
                                } else {
                                    true // Keep non-entry-effect messages
                                }
                            });
                        }

                        cx.notify();
                    });
                }
            });

            view.on_theme_change({
                let tx = command_tx.clone();
                move |theme, _window, _cx| {
                    let _ = tx.send(UiCommand::UpdateTheme(theme));
                }
            });

            view.on_font_size_change({
                let tx = command_tx.clone();
                let entity = entity.clone();
                move |font_size, _window, cx| {
                    let _ = tx.send(UiCommand::UpdateFontSize(font_size));
                    // Update local font size
                    let _ = entity.update(cx, |view, cx| {
                        view.font_size = font_size;
                        view.last_render_width = 0.0; // Force rebuild of render rows
                        cx.notify();
                    });
                }
            });

            view.on_tts_enabled_change({
                let tx = command_tx.clone();
                move |danmu, gift, superchat, _window, _cx| {
                    let _ = tx.send(UiCommand::UpdateTtsEnabled {
                        danmu,
                        gift,
                        superchat,
                    });
                }
            });

            view.on_tts_volume_change({
                let tx = command_tx.clone();
                move |volume, _window, _cx| {
                    let _ = tx.send(UiCommand::UpdateTtsVolume(volume));
                }
            });

            view.on_tts_test({
                let tx = command_tx.clone();
                move |_window, _cx| {
                    let _ = tx.send(UiCommand::TestTts);
                }
            });

            view.on_plugin_open({
                let entity = entity.clone();
                move |plugin_id, plugin_name, plugin_path, window, cx| {
                    let _ = entity.update(cx, |view, cx| {
                        view.open_plugin_in_browser(plugin_id, plugin_name, plugin_path, window, cx);
                    });
                }
            });

            view.on_open_plugins_folder({
                let entity = entity.clone();
                move |_window, cx| {
                    let _ = entity.update(cx, |view, _cx| {
                        if let Some(ref config) = view.config {
                            let plugins_dir = config.read().data_dir().join("plugins");
                            // Create the directory if it doesn't exist
                            let _ = std::fs::create_dir_all(&plugins_dir);
                            // Open the folder
                            let _ = open::that(&plugins_dir);
                        }
                    });
                }
            });

            view.on_refresh_plugins({
                let tx = command_tx.clone();
                move |_window, _cx| {
                    let _ = tx.send(UiCommand::RefreshPlugins);
                }
            });

            view.on_plugin_import({
                let tx = command_tx.clone();
                move |url, _window, _cx| {
                    let _ = tx.send(UiCommand::ImportPlugin(url));
                }
            });

            view.on_plugin_remove({
                let tx = command_tx.clone();
                move |plugin_id, plugin_path, _window, _cx| {
                    let _ = tx.send(UiCommand::RemovePlugin { plugin_id, plugin_path });
                }
            });

            view.on_advanced_settings_change({
                let tx = command_tx.clone();
                move |max_danmu, log_level, _window, _cx| {
                    let _ = tx.send(UiCommand::UpdateAdvancedSettings { max_danmu, log_level });
                }
            });

            view.on_clear_data({
                let tx = command_tx.clone();
                move |_window, _cx| {
                    let _ = tx.send(UiCommand::ClearAllData);
                }
            });

            view.on_open_data_folder({
                let tx = command_tx.clone();
                move |_window, _cx| {
                    let _ = tx.send(UiCommand::OpenDataFolder);
                }
            });

            view.on_room_title_change({
                let tx = command_tx.clone();
                move |room_id, title, _window, _cx| {
                    let _ = tx.send(UiCommand::UpdateRoomTitle { room_id, title });
                }
            });

            view.on_check_update({
                let tx = command_tx.clone();
                move |_window, _cx| {
                    let _ = tx.send(UiCommand::CheckForUpdate);
                }
            });

            view.on_auto_update_change({
                let tx = command_tx.clone();
                move |enabled, _window, _cx| {
                    let _ = tx.send(UiCommand::UpdateAutoUpdateCheck(enabled));
                }
            });

            view.on_plugin_port_change({
                let tx = command_tx.clone();
                move |ws_port, http_port, _window, _cx| {
                    let _ = tx.send(UiCommand::UpdatePluginPorts { ws_port, http_port });
                }
            });
        });

        let this = Self {
            event_rx,
            command_tx,
            has_events: has_events.clone(),
            room: None,
            room_title: String::new(),
            live_status: 0,
            area_id: 0,
            online_count: 0,
            connected: false,
            danmu_list: VecDeque::with_capacity(MAX_DANMU_COUNT),
            render_rows: Rc::new(Vec::new()),
            last_render_width: 0.0,
            render_rows_source_count: 0,
            pending_scroll_to_bottom: true,
            setting_view,
            gift_view,
            superchat_view,
            statistics_view,
            audience_view,
            database: None,
            config: None,
            opacity: 1.0,
            font_size: 14.0,
            lite_mode: false,
            medal_display: true,
            interact_display: false,
            guard_effect: true,
            level_effect: false,
            always_on_top: false,
            pending_always_on_top: None,
            click_through: false,
            pending_click_through: Arc::new(AtomicBool::new(false)),
            scroll_handle: UniformListScrollHandle::new(),
            settings_window: None,
            gift_window: None,
            superchat_window: None,
            statistics_window: None,
            audience_window: None,
            input_state: None,
            _input_subscription: None,
            pending_input_clear: Rc::new(Cell::new(false)),
            selected_user: Rc::new(RefCell::new(None)),
            last_saved_bounds: None,
            ws_port: None,
            http_port: None,
            show_command_popup: Rc::new(Cell::new(false)),
            selected_command_index: Rc::new(Cell::new(0)),
            pending_command_insert: Rc::new(RefCell::new(None)),
            show_update_dialog: false,
            update_info: None,
            tray_manager: None,
            logged_in: false,
            logged_in_uid: None,
            pending_open_settings: Arc::new(AtomicBool::new(false)),
        };

        // Start a timer to periodically check for new events from the backend
        // Only triggers re-render when has_events flag is set
        cx.spawn(async move |view: WeakEntity<Self>, cx: &mut AsyncApp| {
            loop {
                Timer::after(Duration::from_millis(200)).await;
                // Only notify if there are pending events
                if has_events.swap(false, Ordering::Relaxed) {
                    let result = cx.update(|cx| {
                        view.update(cx, |_, cx| {
                            cx.notify();
                        })
                    });
                    if result.is_err() {
                        break; // View was dropped
                    }
                }
            }
        })
        .detach();

        this
    }

    /// Get the setting view entity
    pub fn setting_view(&self) -> &Entity<SettingView> {
        &self.setting_view
    }

    /// Set the list of plugins in the settings view
    pub fn set_plugins(&mut self, plugins: Vec<crate::views::setting_view::PluginInfo>, cx: &mut Context<Self>) {
        self.setting_view.update(cx, |view, cx| {
            view.set_plugins(plugins, cx);
        });
    }

    /// Set plugin import status message
    pub fn set_plugin_import_status(&mut self, status: Option<String>, cx: &mut Context<Self>) {
        self.setting_view.update(cx, |view, cx| {
            view.set_plugin_import_status(status, cx);
        });
    }

    /// Set the WebSocket port for plugin communication
    pub fn set_ws_port(&mut self, port: u16, cx: &mut Context<Self>) {
        self.ws_port = Some(port);
        // Update setting view with the port
        if let Some(http_port) = self.http_port {
            self.setting_view.update(cx, |view, cx| {
                view.set_plugin_ports(port, http_port, cx);
            });
        }
    }

    /// Set the HTTP port for serving plugin files
    pub fn set_http_port(&mut self, port: u16, cx: &mut Context<Self>) {
        self.http_port = Some(port);
        // Update setting view with the port
        if let Some(ws_port) = self.ws_port {
            self.setting_view.update(cx, |view, cx| {
                view.set_plugin_ports(ws_port, port, cx);
            });
        }
    }

    /// Set the database reference
    pub fn set_database(&mut self, db: Arc<Database>) {
        self.database = Some(db.clone());
    }

    /// Set the config store reference
    pub fn set_config(
        &mut self,
        config: Arc<parking_lot::RwLock<jlivertool_core::config::ConfigStore>>,
    ) {
        self.config = Some(config);
    }

    /// Set the tray manager reference
    pub fn set_tray_manager(&mut self, tray: Arc<parking_lot::Mutex<TrayManager>>) {
        self.tray_manager = Some(tray);
    }

    /// Set the pending_open_settings flag (shared with tray command handler)
    pub fn set_pending_open_settings_flag(&mut self, flag: Arc<AtomicBool>) {
        self.pending_open_settings = flag;
    }

    /// Set the pending_click_through flag (shared with tray command handler)
    pub fn set_pending_click_through_flag(&mut self, flag: Arc<AtomicBool>) {
        self.pending_click_through = flag;
    }

    /// Get the pending_open_settings flag for tray command handling
    pub fn pending_open_settings_flag(&self) -> Arc<AtomicBool> {
        self.pending_open_settings.clone()
    }

    /// Update the tray state based on current view state
    fn update_tray_state(&self) {
        if let Some(ref tray) = self.tray_manager {
            // Check if logged-in user owns the current room
            let is_room_owner = match (self.logged_in_uid, self.room.as_ref()) {
                (Some(uid), Some(room)) => uid == room.owner_uid(),
                _ => false,
            };

            let state = TrayState {
                room_id: self.room.as_ref().map(|r| r.real_id()),
                room_title: self.room_title.clone(),
                area_id: self.area_id,
                live_status: self.live_status,
                logged_in: self.logged_in,
                is_room_owner,
                connected: self.connected,
                window_visible: true, // We don't track this in MainView currently
                click_through: false, // Managed by TrayManager directly
            };
            tray.lock().update_state(state);

            // Update icon based on live status
            tray.lock().update_icon(self.live_status == 1);
        }
    }

    /// Check if scroll is at or near the bottom
    fn is_at_bottom(&self) -> bool {
        if self.render_rows.len() <= 1 {
            return true;
        }

        let scroll_state = self.scroll_handle.0.borrow();
        let base_handle = &scroll_state.base_handle;
        let offset = base_handle.offset();
        let max_offset = base_handle.max_offset();
        let threshold = px(50.0);
        offset.y <= -max_offset.height + threshold
    }

    /// Scroll to the bottom of the danmu list (deferred until render_rows are rebuilt)
    fn scroll_to_bottom(&mut self) {
        self.pending_scroll_to_bottom = true;
    }

    /// Apply pending scroll-to-bottom after render_rows have been rebuilt
    fn apply_pending_scroll(&mut self) {
        if self.pending_scroll_to_bottom && !self.render_rows.is_empty() {
            self.pending_scroll_to_bottom = false;
            let last_index = self.render_rows.len().saturating_sub(1);
            self.scroll_handle
                .scroll_to_item(last_index, ScrollStrategy::Bottom);
        }
    }

    /// Handle debug commands (only available in debug builds)
    #[cfg(debug_assertions)]
    pub(super) fn handle_debug_command(&mut self, args: &str) {
        use jlivertool_core::messages::{GiftInfo, GiftMessage, GuardMessage, SuperChatMessage};
        use jlivertool_core::types::{MedalInfo, Sender};

        let now = chrono::Utc::now().timestamp();
        let debug_id = format!("debug-{}", now);
        let fake_sender = Sender {
            uid: 12345678,
            uname: "测试用户".to_string(),
            face: String::new(),
            medal_info: MedalInfo {
                anchor_roomid: 0,
                anchor_uname: String::new(),
                guard_level: 0,
                medal_color: 0x6154c1,
                medal_color_border: 0x6154c1,
                medal_color_start: 0x6154c1,
                medal_color_end: 0x6154c1,
                medal_level: 20,
                medal_name: "测试".to_string(),
                is_lighted: true,
            },
        };

        let parts: Vec<&str> = args.splitn(2, ' ').collect();
        let sub_cmd = parts.first().copied().unwrap_or("");
        let content = parts.get(1).copied().unwrap_or("测试内容");

        match sub_cmd {
            "sc" => {
                let sc = SuperChatMessage {
                    id: debug_id.clone(),
                    room: 0,
                    sender: fake_sender,
                    message: content.to_string(),
                    price: 30,
                    timestamp: now,
                    start_time: now,
                    end_time: now + 60,
                    background_color: "#EDF5FF".to_string(),
                    background_bottom_color: "#2A60B2".to_string(),
                    archived: false,
                };
                self.danmu_list.push_back(DisplayMessage::SuperChat(sc));
            }
            "gift" => {
                let gift = GiftMessage {
                    id: debug_id.clone(),
                    room: 0,
                    gift_info: GiftInfo {
                        id: 1,
                        name: content.to_string(),
                        price: 1000,
                        coin_type: "gold".to_string(),
                        img_basic: String::new(),
                        img_dynamic: String::new(),
                        gif: String::new(),
                        webp: String::new(),
                    },
                    sender: fake_sender,
                    action: "投喂".to_string(),
                    num: 1,
                    timestamp: now,
                    archived: false,
                };
                self.danmu_list.push_back(DisplayMessage::Gift(gift));
            }
            "guard" => {
                let guard = GuardMessage {
                    id: debug_id.clone(),
                    room: 0,
                    sender: fake_sender,
                    num: 1,
                    unit: "月".to_string(),
                    guard_level: 3,
                    price: 198000,
                    timestamp: now,
                    archived: false,
                };
                self.danmu_list.push_back(DisplayMessage::Guard(guard));
            }
            "danmu" | _ => {
                let danmu = jlivertool_core::messages::DanmuMessage {
                    sender: fake_sender,
                    content: content.to_string(),
                    is_generated: false,
                    is_special: false,
                    emoji_content: None,
                    side_index: -1,
                    reply_uname: None,
                };
                self.danmu_list.push_back(DisplayMessage::Danmu(danmu));
            }
        }

        while self.danmu_list.len() > MAX_DANMU_COUNT {
            self.danmu_list.pop_front();
        }
        // Reset render rows so they get rebuilt on next render
        self.render_rows_source_count = 0;
        self.render_rows = Rc::new(Vec::new());
        self.scroll_to_bottom();
    }

    /// Update render rows: full rebuild if width changed, incremental append otherwise.
    pub(super) fn update_render_rows(&mut self, window_width: f32) {
        if (window_width - self.last_render_width).abs() > 1.0 || self.last_render_width == 0.0 {
            self.rebuild_render_rows(window_width);
        } else {
            self.append_new_render_rows(window_width);
        }
    }

    /// Fully rebuild render_rows from danmu_list for the given window width.
    fn rebuild_render_rows(&mut self, window_width: f32) {
        let mut rows = Vec::new();
        let available_width = window_width - 14.0; // scrollbar buffer
        for msg in self.danmu_list.iter() {
            Self::append_message_rows(
                &mut rows,
                msg,
                available_width,
                self.font_size,
                self.lite_mode,
                self.medal_display,
            );
        }
        self.render_rows = Rc::new(rows);
        self.last_render_width = window_width;
        self.render_rows_source_count = self.danmu_list.len();
    }

    /// Incrementally append new messages to render_rows.
    fn append_new_render_rows(&mut self, window_width: f32) {
        let current_source_len = self.danmu_list.len();

        // If items were removed from front (MAX_DANMU_COUNT), full rebuild needed
        if current_source_len < self.render_rows_source_count {
            self.rebuild_render_rows(window_width);
            return;
        }

        // Nothing new to add
        if current_source_len == self.render_rows_source_count {
            return;
        }

        let available_width = window_width - 14.0;
        let mut rows = Rc::try_unwrap(std::mem::replace(&mut self.render_rows, Rc::new(Vec::new())))
            .unwrap_or_else(|rc| (*rc).clone());

        let new_start = self.render_rows_source_count;
        for i in new_start..current_source_len {
            let msg = &self.danmu_list[i];
            Self::append_message_rows(
                &mut rows,
                msg,
                available_width,
                self.font_size,
                self.lite_mode,
                self.medal_display,
            );
        }

        self.render_rows = Rc::new(rows);
        self.render_rows_source_count = current_source_len;
    }

    /// Convert a single DisplayMessage into one or more RenderRows and append them.
    fn append_message_rows(
        rows: &mut Vec<RenderRow>,
        msg: &DisplayMessage,
        available_width: f32,
        font_size: f32,
        lite_mode: bool,
        medal_display: bool,
    ) {
        match msg {
            DisplayMessage::Danmu(danmu) => {
                // Skip wrapping for emoji danmu
                if danmu.emoji_content.is_some() {
                    rows.push(RenderRow::Full(msg.clone()));
                    return;
                }

                let prefix_width =
                    estimate_danmu_prefix_width(danmu, font_size, lite_mode, medal_display);
                let first_line_content_width = available_width - prefix_width;
                // Continuation line has only padding, no prefix
                let padding = if lite_mode { 4.0 * 2.0 } else { 8.0 * 2.0 };
                let continuation_content_width = available_width - padding;

                let content_width = estimate_text_width(&danmu.content, font_size);
                if content_width <= first_line_content_width || first_line_content_width <= 0.0 {
                    rows.push(RenderRow::Full(msg.clone()));
                } else {
                    let lines = split_content_to_lines(
                        &danmu.content,
                        font_size,
                        first_line_content_width,
                        continuation_content_width,
                    );

                    if lines.len() <= 1 {
                        rows.push(RenderRow::Full(msg.clone()));
                    } else {
                        rows.push(RenderRow::DanmuFirstLine {
                            danmu: danmu.clone(),
                            content_slice: lines[0].clone(),
                        });
                        for (i, line) in lines[1..].iter().enumerate() {
                            rows.push(RenderRow::DanmuContinuation {
                                danmu: danmu.clone(),
                                content_slice: line.clone(),
                                continuation_index: i,
                            });
                        }
                    }
                }
            }
            DisplayMessage::SuperChat(sc) => {
                // Always split superchat: header (avatar + price + username) + content rows
                rows.push(RenderRow::SuperChatHeader { sc: sc.clone() });

                if !sc.message.is_empty() {
                    let padding = if lite_mode { 4.0 * 2.0 } else { 8.0 * 2.0 };
                    // Account for border_l_4 (4px)
                    let content_line_width = available_width - padding - 4.0;
                    let content_width = estimate_text_width(&sc.message, font_size * 0.9);

                    if content_width <= content_line_width || content_line_width <= 0.0 {
                        rows.push(RenderRow::SuperChatContent {
                            sc: sc.clone(),
                            content_slice: sc.message.clone(),
                            continuation_index: 0,
                            is_last: true,
                        });
                    } else {
                        let lines = split_content_to_lines(
                            &sc.message,
                            font_size * 0.9,
                            content_line_width,
                            content_line_width,
                        );
                        let last_idx = lines.len().saturating_sub(1);
                        for (i, line) in lines.iter().enumerate() {
                            rows.push(RenderRow::SuperChatContent {
                                sc: sc.clone(),
                                content_slice: line.clone(),
                                continuation_index: i,
                                is_last: i == last_idx,
                            });
                        }
                    }
                }
            }
            _ => {
                rows.push(RenderRow::Full(msg.clone()));
            }
        }
    }

    /// Open settings window
    fn open_settings_window(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        use crate::views::WindowBoundsTracker;
        use gpui_component::Root;
        use jlivertool_core::types::WindowType;

        // Check if window is already open and focus it
        if let Some(handle) = &self.settings_window {
            if cx
                .update_window(*handle, |_, window, _cx| {
                    window.activate_window();
                })
                .is_ok()
            {
                return;
            }
            self.settings_window = None;
        }

        // Load saved bounds or use default
        let bounds = if let Some(ref config) = self.config {
            let saved = config.read().get_window_config(WindowType::Setting);
            if saved.width > 0 && saved.height > 0 {
                Bounds::new(
                    point(px(saved.x as f32), px(saved.y as f32)),
                    size(px(saved.width as f32), px(saved.height as f32)),
                )
            } else {
                Bounds::centered(None, size(px(800.0), px(700.0)), cx)
            }
        } else {
            Bounds::centered(None, size(px(800.0), px(700.0)), cx)
        };

        let setting_view = self.setting_view.clone();
        let always_on_top = self.always_on_top;
        let command_tx = self.command_tx.clone();

        if let Ok(handle) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("设置".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_background: WindowBackgroundAppearance::Transparent,
                window_min_size: Some(size(px(700.0), px(500.0))),
                ..Default::default()
            },
            |new_window, cx| {
                if always_on_top {
                    crate::platform::set_window_always_on_top(new_window, true);
                }
                let tracker = cx.new(|_| {
                    WindowBoundsTracker::new(setting_view, WindowType::Setting, command_tx)
                });
                cx.new(|cx| Root::new(tracker, new_window, cx))
            },
        ) {
            self.settings_window = Some(handle.into());
        }
    }

    /// Open settings window (deferred version for use outside render context)
    /// This is called from cx.spawn() to avoid opening windows during render
    fn open_settings_window_deferred(&mut self, cx: &mut Context<Self>) {
        use crate::views::WindowBoundsTracker;
        use gpui_component::Root;
        use jlivertool_core::types::WindowType;

        // Check if window is already open and focus it
        if let Some(handle) = &self.settings_window {
            if cx
                .update_window(*handle, |_, window, _cx| {
                    window.activate_window();
                })
                .is_ok()
            {
                return;
            }
            self.settings_window = None;
        }

        // Load saved bounds or use default
        let bounds = if let Some(ref config) = self.config {
            let saved = config.read().get_window_config(WindowType::Setting);
            if saved.width > 0 && saved.height > 0 {
                Bounds::new(
                    point(px(saved.x as f32), px(saved.y as f32)),
                    size(px(saved.width as f32), px(saved.height as f32)),
                )
            } else {
                Bounds::centered(None, size(px(800.0), px(700.0)), cx)
            }
        } else {
            Bounds::centered(None, size(px(800.0), px(700.0)), cx)
        };

        let setting_view = self.setting_view.clone();
        let always_on_top = self.always_on_top;
        let command_tx = self.command_tx.clone();

        if let Ok(handle) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("设置".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_background: WindowBackgroundAppearance::Transparent,
                window_min_size: Some(size(px(700.0), px(500.0))),
                ..Default::default()
            },
            |new_window, cx| {
                if always_on_top {
                    crate::platform::set_window_always_on_top(new_window, true);
                }
                let tracker = cx.new(|_| {
                    WindowBoundsTracker::new(setting_view, WindowType::Setting, command_tx)
                });
                cx.new(|cx| Root::new(tracker, new_window, cx))
            },
        ) {
            self.settings_window = Some(handle.into());
        }
    }

    /// Open gift window
    fn open_gift_window(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        use crate::views::WindowBoundsTracker;
        use gpui_component::Root;
        use jlivertool_core::types::WindowType;

        if let Some(handle) = &self.gift_window {
            if cx
                .update_window(*handle, |_, window, _cx| {
                    window.activate_window();
                })
                .is_ok()
            {
                return;
            }
            self.gift_window = None;
        }

        let bounds = if let Some(ref config) = self.config {
            let saved = config.read().get_window_config(WindowType::Gift);
            if saved.width > 0 && saved.height > 0 {
                Bounds::new(
                    point(px(saved.x as f32), px(saved.y as f32)),
                    size(px(saved.width as f32), px(saved.height as f32)),
                )
            } else {
                Bounds::centered(None, size(px(500.0), px(600.0)), cx)
            }
        } else {
            Bounds::centered(None, size(px(500.0), px(600.0)), cx)
        };

        let gift_view = self.gift_view.clone();
        let always_on_top = self.always_on_top;
        let click_through = self.click_through;
        let command_tx = self.command_tx.clone();

        if let Ok(handle) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("礼物记录".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_background: WindowBackgroundAppearance::Transparent,
                window_min_size: Some(size(px(400.0), px(300.0))),
                ..Default::default()
            },
            |new_window, cx| {
                if always_on_top {
                    crate::platform::set_window_always_on_top(new_window, true);
                }
                if click_through {
                    crate::platform::set_window_click_through(new_window, true);
                }
                let tracker =
                    cx.new(|_| WindowBoundsTracker::new(gift_view, WindowType::Gift, command_tx));
                cx.new(|cx| Root::new(tracker, new_window, cx))
            },
        ) {
            self.gift_window = Some(handle.into());
        }
    }

    /// Open superchat window
    fn open_superchat_window(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        use crate::views::WindowBoundsTracker;
        use gpui_component::Root;
        use jlivertool_core::types::WindowType;

        if let Some(handle) = &self.superchat_window {
            if cx
                .update_window(*handle, |_, window, _cx| {
                    window.activate_window();
                })
                .is_ok()
            {
                return;
            }
            self.superchat_window = None;
        }

        let bounds = if let Some(ref config) = self.config {
            let saved = config.read().get_window_config(WindowType::SuperChat);
            if saved.width > 0 && saved.height > 0 {
                Bounds::new(
                    point(px(saved.x as f32), px(saved.y as f32)),
                    size(px(saved.width as f32), px(saved.height as f32)),
                )
            } else {
                Bounds::centered(None, size(px(500.0), px(600.0)), cx)
            }
        } else {
            Bounds::centered(None, size(px(500.0), px(600.0)), cx)
        };

        let superchat_view = self.superchat_view.clone();
        let always_on_top = self.always_on_top;
        let click_through = self.click_through;
        let command_tx = self.command_tx.clone();

        if let Ok(handle) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("醒目留言".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_background: WindowBackgroundAppearance::Transparent,
                window_min_size: Some(size(px(400.0), px(300.0))),
                ..Default::default()
            },
            |new_window, cx| {
                if always_on_top {
                    crate::platform::set_window_always_on_top(new_window, true);
                }
                if click_through {
                    crate::platform::set_window_click_through(new_window, true);
                }
                let tracker = cx.new(|_| {
                    WindowBoundsTracker::new(superchat_view, WindowType::SuperChat, command_tx)
                });
                cx.new(|cx| Root::new(tracker, new_window, cx))
            },
        ) {
            self.superchat_window = Some(handle.into());
        }
    }

    /// Open statistics window
    fn open_statistics_window(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        use crate::views::WindowBoundsTracker;
        use gpui_component::Root;
        use jlivertool_core::types::WindowType;

        if let Some(handle) = &self.statistics_window {
            if cx
                .update_window(*handle, |_, window, _cx| {
                    window.activate_window();
                })
                .is_ok()
            {
                return;
            }
            self.statistics_window = None;
        }

        let bounds = if let Some(ref config) = self.config {
            let saved = config.read().get_window_config(WindowType::Detail);
            if saved.width > 0 && saved.height > 0 {
                Bounds::new(
                    point(px(saved.x as f32), px(saved.y as f32)),
                    size(px(saved.width as f32), px(saved.height as f32)),
                )
            } else {
                Bounds::centered(None, size(px(420.0), px(600.0)), cx)
            }
        } else {
            Bounds::centered(None, size(px(420.0), px(600.0)), cx)
        };

        let statistics_view = self.statistics_view.clone();
        let always_on_top = self.always_on_top;
        let command_tx = self.command_tx.clone();

        if let Some(db) = &self.database {
            self.statistics_view.update(cx, |view, cx| {
                view.set_database(db.clone());
                view.set_room_id(self.room.as_ref().map(|r| r.real_id()), cx);
            });
        }

        if let Ok(handle) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("数据统计".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_background: WindowBackgroundAppearance::Transparent,
                window_min_size: Some(size(px(350.0), px(300.0))),
                ..Default::default()
            },
            |new_window, cx| {
                if always_on_top {
                    crate::platform::set_window_always_on_top(new_window, true);
                }
                let tracker = cx.new(|_| {
                    WindowBoundsTracker::new(statistics_view, WindowType::Detail, command_tx)
                });
                cx.new(|cx| Root::new(tracker, new_window, cx))
            },
        ) {
            self.statistics_window = Some(handle.into());
        }
    }

    /// Open audience window
    fn open_audience_window(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        use crate::views::WindowBoundsTracker;
        use gpui_component::Root;
        use jlivertool_core::types::WindowType;

        if let Some(handle) = &self.audience_window {
            if cx
                .update_window(*handle, |_, window, _cx| {
                    window.activate_window();
                })
                .is_ok()
            {
                return;
            }
            self.audience_window = None;
        }

        let bounds = if let Some(ref config) = self.config {
            let saved = config.read().get_window_config(WindowType::Rank);
            if saved.width > 0 && saved.height > 0 {
                Bounds::new(
                    point(px(saved.x as f32), px(saved.y as f32)),
                    size(px(saved.width as f32), px(saved.height as f32)),
                )
            } else {
                Bounds::centered(None, size(px(400.0), px(500.0)), cx)
            }
        } else {
            Bounds::centered(None, size(px(400.0), px(500.0)), cx)
        };

        let audience_view = self.audience_view.clone();
        let always_on_top = self.always_on_top;
        let command_tx = self.command_tx.clone();

        if let Some(room) = &self.room {
            let room_id = room.real_id();
            let ruid = room.owner_uid();
            let tx_audience = command_tx.clone();
            let tx_guards = command_tx.clone();

            self.audience_view.update(cx, |view, _cx| {
                view.on_fetch_audience(move |_window, _cx| {
                    let _ = tx_audience.send(UiCommand::FetchAudienceList { room_id, ruid });
                });
                view.on_fetch_guards(move |page, _window, _cx| {
                    let _ = tx_guards.send(UiCommand::FetchGuardList {
                        room_id,
                        ruid,
                        page,
                    });
                });
            });
        }

        let command_tx_for_tracker = self.command_tx.clone();

        if let Ok(handle) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("观众列表".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_background: WindowBackgroundAppearance::Transparent,
                window_min_size: Some(size(px(350.0), px(300.0))),
                ..Default::default()
            },
            |new_window, cx| {
                if always_on_top {
                    crate::platform::set_window_always_on_top(new_window, true);
                }
                let tracker = cx.new(|_| {
                    WindowBoundsTracker::new(
                        audience_view,
                        WindowType::Rank,
                        command_tx_for_tracker,
                    )
                });
                cx.new(|cx| Root::new(tracker, new_window, cx))
            },
        ) {
            self.audience_window = Some(handle.into());

            if self.room.is_some() {
                let room = self.room.as_ref().unwrap();
                let room_id = room.real_id();
                let ruid = room.owner_uid();
                let _ = self
                    .command_tx
                    .send(UiCommand::FetchAudienceList { room_id, ruid });
                let _ = self.command_tx.send(UiCommand::FetchGuardList {
                    room_id,
                    ruid,
                    page: 1,
                });
            }
        }
    }

    /// Open plugin in the system's default browser
    fn open_plugin_in_browser(
        &self,
        _plugin_id: String,
        _plugin_name: String,
        plugin_path: PathBuf,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let http_port = match self.http_port {
            Some(port) => port,
            None => {
                tracing::error!("HTTP port not set, cannot open plugin in browser");
                return;
            }
        };

        let ws_port = match self.ws_port {
            Some(port) => port,
            None => {
                tracing::error!("WebSocket port not set, cannot open plugin in browser");
                return;
            }
        };

        // Get the folder name from the plugin path
        let folder_name = match plugin_path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                tracing::error!("Invalid plugin path: {:?}", plugin_path);
                return;
            }
        };

        // Build the URL for the plugin
        let url = format!(
            "http://127.0.0.1:{}/{}/index.html?ws_port={}",
            http_port, folder_name, ws_port
        );

        tracing::info!("Opening plugin in browser: {}", url);

        // Open in the system's default browser
        if let Err(e) = open::that(&url) {
            tracing::error!("Failed to open plugin in browser: {}", e);
        }
    }
}
