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

pub use content_rendering::{render_content_with_links, DisplayMessage};
use danmu_list_item::DanmuListItemView;
use user_info_card::{SelectedUserState, UserInfoCard};

use crate::app::UiCommand;
use crate::tray::{TrayManager, TrayState};
use crate::views::AudienceView;
use crate::views::GiftView;
use crate::views::PluginWindowView;
use crate::views::SettingView;
use crate::views::StatisticsView;
use crate::views::SuperChatView;
use gpui::*;
use jlivertool_core::database::Database;
use jlivertool_core::events::Event;
use jlivertool_core::types::RoomId;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
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
    /// Snapshot of danmu_list for efficient rendering (updated only when list changes)
    danmu_list_snapshot: Rc<Vec<DisplayMessage>>,
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
    // Plugin windows (plugin_id -> window handle)
    plugin_windows: HashMap<String, AnyWindowHandle>,
    // WebSocket port for plugin communication
    ws_port: Option<u16>,
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
                        view.open_plugin_window(plugin_id, plugin_name, plugin_path, window, cx);
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
            danmu_list_snapshot: Rc::new(Vec::new()),
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
            plugin_windows: HashMap::new(),
            ws_port: None,
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
    pub fn set_ws_port(&mut self, port: u16) {
        self.ws_port = Some(port);
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
            };
            tray.lock().update_state(state);

            // Update icon based on live status
            tray.lock().update_icon(self.live_status == 1);
        }
    }

    /// Check if scroll is at or near the bottom
    fn is_at_bottom(&self) -> bool {
        if self.danmu_list.len() <= 1 {
            return true;
        }

        let scroll_state = self.scroll_handle.0.borrow();
        let base_handle = &scroll_state.base_handle;
        let offset = base_handle.offset();
        let max_offset = base_handle.max_offset();
        let threshold = px(50.0);
        offset.y <= -max_offset.height + threshold
    }

    /// Scroll to the bottom of the danmu list
    fn scroll_to_bottom(&mut self) {
        let last_index = self.danmu_list.len().saturating_sub(1);
        self.scroll_handle
            .scroll_to_item(last_index, ScrollStrategy::Bottom);
    }

    /// Update the danmu list snapshot for efficient rendering
    fn update_snapshot(&mut self) {
        self.danmu_list_snapshot = Rc::new(self.danmu_list.iter().cloned().collect());
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

    /// Open plugin window
    fn open_plugin_window(
        &mut self,
        plugin_id: String,
        plugin_name: String,
        plugin_path: PathBuf,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use gpui_component::Root;

        // Check if window is already open and focus it
        if let Some(handle) = self.plugin_windows.get(&plugin_id) {
            if cx
                .update_window(*handle, |_, window, _cx| {
                    window.activate_window();
                })
                .is_ok()
            {
                return;
            }
            self.plugin_windows.remove(&plugin_id);
        }

        let ws_port = match self.ws_port {
            Some(port) => port,
            None => {
                tracing::error!("WebSocket port not set, cannot open plugin window");
                return;
            }
        };

        let bounds = Bounds::centered(None, size(px(600.0), px(500.0)), cx);
        let always_on_top = self.always_on_top;
        let plugin_id_clone = plugin_id.clone();

        if let Ok(handle) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(plugin_name.clone().into()),
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
                let plugin_view = cx.new(|cx| {
                    PluginWindowView::new(
                        plugin_id_clone,
                        plugin_name,
                        plugin_path,
                        ws_port,
                        new_window,
                        cx,
                    )
                });
                cx.new(|cx| Root::new(plugin_view, new_window, cx))
            },
        ) {
            self.plugin_windows.insert(plugin_id, handle.into());
        }
    }
}
