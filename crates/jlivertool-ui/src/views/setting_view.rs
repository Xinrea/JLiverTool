//! Settings view

use crate::components::QrCodeView;
use crate::theme::Colors;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    h_flex,
    slider::{Slider, SliderEvent, SliderState},
    switch::Switch,
    v_flex,
};
use jlivertool_core::bilibili::api::{QrCodeStatus, UserInfoData};
use parking_lot::RwLock;
use std::sync::Arc;

/// Type alias for simple callbacks (no parameters)
type SimpleCallback = Arc<dyn Fn(&mut Window, &mut App) + Send + Sync>;
/// Type alias for room-related callbacks (room_id parameter)
type RoomCallback = Arc<dyn Fn(u64, &mut Window, &mut App) + Send + Sync>;
/// Type alias for opacity/font size callbacks (f32 parameter)
type FloatCallback = Arc<dyn Fn(f32, &mut Window, &mut App) + Send + Sync>;
/// Type alias for window settings callback (5 bool parameters)
type WindowSettingsCallback = Arc<dyn Fn(bool, bool, bool, bool, bool, &mut Window, &mut App) + Send + Sync>;
/// Type alias for theme callback (String parameter)
type ThemeCallback = Arc<dyn Fn(String, &mut Window, &mut App) + Send + Sync>;
/// Type alias for TTS enabled callback (3 bool parameters: danmu, gift, superchat)
type TtsEnabledCallback = Arc<dyn Fn(bool, bool, bool, &mut Window, &mut App) + Send + Sync>;
/// Type alias for plugin open callback (plugin_id, plugin_name, plugin_path)
type PluginOpenCallback = Arc<dyn Fn(String, String, std::path::PathBuf, &mut Window, &mut App) + Send + Sync>;

/// Plugin info for display in settings
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub author: String,
    pub desc: String,
    pub version: String,
    pub enabled: bool,
    pub path: std::path::PathBuf,
}

/// Settings view state
pub struct SettingView {
    // Settings data
    settings: Arc<RwLock<SettingsData>>,
    // Room ID display
    room_id: Option<u64>,
    // Room owner UID (to check if user owns the room)
    room_owner_uid: Option<u64>,
    // Room editing state
    room_input: Arc<RwLock<RoomInputState>>,
    // Account info
    account: Arc<RwLock<AccountState>>,
    // QR code view
    qr_code_view: Entity<QrCodeView>,
    // Callbacks
    on_qr_login: Option<SimpleCallback>,
    on_logout: Option<SimpleCallback>,
    on_change_room: Option<RoomCallback>,
    on_opacity_change: Option<FloatCallback>,
    on_open_gift_window: Option<SimpleCallback>,
    on_open_superchat_window: Option<SimpleCallback>,
    on_open_statistics_window: Option<SimpleCallback>,
    on_start_live: Option<RoomCallback>,
    on_stop_live: Option<SimpleCallback>,
    on_window_settings_change: Option<WindowSettingsCallback>,
    on_theme_change: Option<ThemeCallback>,
    on_font_size_change: Option<FloatCallback>,
    rtmp_info: Arc<RwLock<Option<RtmpInfo>>>,
    // Merge settings
    merge_settings: Arc<RwLock<MergeSettings>>,
    // Active tab
    active_tab: usize,
    // TTS callbacks
    on_tts_enabled_change: Option<TtsEnabledCallback>,
    on_tts_volume_change: Option<FloatCallback>,
    on_tts_test: Option<SimpleCallback>,
    // Plugin management
    plugins: Arc<RwLock<Vec<PluginInfo>>>,
    on_plugin_open: Option<PluginOpenCallback>,
    on_open_plugins_folder: Option<SimpleCallback>,
    on_refresh_plugins: Option<SimpleCallback>,
}

/// Tab definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    Basic = 0,
    Window = 1,
    Appearance = 2,
    Tts = 3,
    Plugin = 4,
    Advanced = 5,
    About = 6,
}

impl SettingsTab {
    fn name(&self) -> &'static str {
        match self {
            Self::Basic => "基础设置",
            Self::Window => "窗口设置",
            Self::Appearance => "外观设置",
            Self::Tts => "TTS 设置",
            Self::Plugin => "插件管理",
            Self::Advanced => "高级设置",
            Self::About => "关于",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::Basic,
            Self::Window,
            Self::Appearance,
            Self::Tts,
            Self::Plugin,
            Self::Advanced,
            Self::About,
        ]
    }
}

#[derive(Clone, Default)]
pub struct RtmpInfo {
    pub addr: String,
    pub code: String,
}

/// Merge settings for danmu aggregation
#[derive(Clone, Default)]
pub struct MergeSettings {
    pub enabled: bool,
    pub rooms: Vec<u64>,
}

/// Room input state
#[derive(Clone, Default)]
pub struct RoomInputState {
    pub editing: bool,
    pub input_text: String,
    pub error: bool,
}

/// Account state
#[derive(Clone, Default)]
pub struct AccountState {
    pub logged_in: bool,
    pub user_info: Option<UserInfoData>,
    pub qr_dialog_open: bool,
    pub qr_url: Option<String>,
    pub qr_status: Option<QrCodeStatus>,
}

/// Configuration values for loading/setting config
#[derive(Debug, Clone)]
pub struct ConfigValues {
    pub always_on_top: bool,
    pub guard_effect: bool,
    pub level_effect: bool,
    pub opacity: f32,
    pub lite_mode: bool,
    pub medal_display: bool,
    pub interact_display: bool,
    pub theme: String,
    pub font_size: f32,
    pub tts_enabled: bool,
    pub tts_gift_enabled: bool,
    pub tts_sc_enabled: bool,
    pub tts_volume: f32,
}

/// Settings data that can be shared
#[derive(Clone)]
pub struct SettingsData {
    // Window display settings
    pub lite_mode: bool,
    pub medal_display: bool,
    pub interact_display: bool,
    pub guard_effect: bool,
    pub level_effect: bool,
    // General settings
    pub always_on_top: bool,
    pub opacity: f32,
    // Appearance settings
    pub theme: String,
    pub font_size: f32,
    // TTS settings
    pub tts_enabled: bool,
    pub gift_tts: bool,
    pub sc_tts: bool,
    pub tts_volume: f32,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            lite_mode: false,
            medal_display: true,
            interact_display: false,
            guard_effect: true,
            level_effect: false,
            always_on_top: false,
            opacity: 1.0,
            theme: "dark".to_string(),
            font_size: 14.0,
            tts_enabled: false,
            gift_tts: false,
            sc_tts: false,
            tts_volume: 1.0,
        }
    }
}

impl SettingView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let qr_code_view = cx.new(QrCodeView::new);

        Self {
            settings: Arc::new(RwLock::new(SettingsData::default())),
            room_id: None,
            room_owner_uid: None,
            room_input: Arc::new(RwLock::new(RoomInputState::default())),
            account: Arc::new(RwLock::new(AccountState::default())),
            qr_code_view,
            on_qr_login: None,
            on_logout: None,
            on_change_room: None,
            on_opacity_change: None,
            on_open_gift_window: None,
            on_open_superchat_window: None,
            on_open_statistics_window: None,
            on_start_live: None,
            on_stop_live: None,
            on_window_settings_change: None,
            on_theme_change: None,
            on_font_size_change: None,
            rtmp_info: Arc::new(RwLock::new(None)),
            merge_settings: Arc::new(RwLock::new(MergeSettings::default())),
            active_tab: 0,
            on_tts_enabled_change: None,
            on_tts_volume_change: None,
            on_tts_test: None,
            plugins: Arc::new(RwLock::new(Vec::new())),
            on_plugin_open: None,
            on_open_plugins_folder: None,
            on_refresh_plugins: None,
        }
    }

    /// Set room ID and owner UID to display
    pub fn set_room_id(&mut self, room_id: Option<u64>, owner_uid: Option<u64>, cx: &mut Context<Self>) {
        self.room_id = room_id;
        self.room_owner_uid = owner_uid;
        // Update input text if not editing
        if !self.room_input.read().editing {
            let mut input = self.room_input.write();
            input.input_text = room_id.map(|id| id.to_string()).unwrap_or_default();
            input.error = false;
        }
        cx.notify();
    }

    /// Check if the logged-in user owns the current room
    fn is_room_owner(&self) -> bool {
        let account = self.account.read();
        if !account.logged_in {
            return false;
        }
        if let (Some(user_info), Some(owner_uid)) = (&account.user_info, self.room_owner_uid) {
            return user_info.mid == owner_uid;
        }
        false
    }

    /// Set change room callback
    pub fn on_change_room<F>(&mut self, callback: F)
    where
        F: Fn(u64, &mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_change_room = Some(Arc::new(callback));
    }

    /// Set login callback
    pub fn on_qr_login<F>(&mut self, callback: F)
    where
        F: Fn(&mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_qr_login = Some(Arc::new(callback));
    }

    /// Set logout callback
    pub fn on_logout<F>(&mut self, callback: F)
    where
        F: Fn(&mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_logout = Some(Arc::new(callback));
    }

    /// Set opacity change callback
    pub fn on_opacity_change<F>(&mut self, callback: F)
    where
        F: Fn(f32, &mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_opacity_change = Some(Arc::new(callback));
    }

    pub fn on_open_gift_window<F>(&mut self, callback: F)
    where
        F: Fn(&mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_open_gift_window = Some(Arc::new(callback));
    }

    pub fn on_open_superchat_window<F>(&mut self, callback: F)
    where
        F: Fn(&mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_open_superchat_window = Some(Arc::new(callback));
    }

    pub fn on_open_statistics_window<F>(&mut self, callback: F)
    where
        F: Fn(&mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_open_statistics_window = Some(Arc::new(callback));
    }

    pub fn on_start_live<F>(&mut self, callback: F)
    where
        F: Fn(u64, &mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_start_live = Some(Arc::new(callback));
    }

    pub fn on_stop_live<F>(&mut self, callback: F)
    where
        F: Fn(&mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_stop_live = Some(Arc::new(callback));
    }

    /// Set callback for window settings changes
    /// Parameters: lite_mode, medal_display, interact_display, guard_effect, level_effect
    pub fn on_window_settings_change<F>(&mut self, callback: F)
    where
        F: Fn(bool, bool, bool, bool, bool, &mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_window_settings_change = Some(Arc::new(callback));
    }

    /// Set callback for theme changes
    pub fn on_theme_change<F>(&mut self, callback: F)
    where
        F: Fn(String, &mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_theme_change = Some(Arc::new(callback));
    }

    /// Set callback for font size changes
    pub fn on_font_size_change<F>(&mut self, callback: F)
    where
        F: Fn(f32, &mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_font_size_change = Some(Arc::new(callback));
    }

    /// Set callback for TTS enabled changes
    pub fn on_tts_enabled_change<F>(&mut self, callback: F)
    where
        F: Fn(bool, bool, bool, &mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_tts_enabled_change = Some(Arc::new(callback));
    }

    /// Set callback for TTS volume changes
    pub fn on_tts_volume_change<F>(&mut self, callback: F)
    where
        F: Fn(f32, &mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_tts_volume_change = Some(Arc::new(callback));
    }

    /// Set callback for TTS test
    pub fn on_tts_test<F>(&mut self, callback: F)
    where
        F: Fn(&mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_tts_test = Some(Arc::new(callback));
    }

    /// Set callback for opening plugin window
    pub fn on_plugin_open<F>(&mut self, callback: F)
    where
        F: Fn(String, String, std::path::PathBuf, &mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_plugin_open = Some(Arc::new(callback));
    }

    /// Set callback for opening plugins folder
    pub fn on_open_plugins_folder<F>(&mut self, callback: F)
    where
        F: Fn(&mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_open_plugins_folder = Some(Arc::new(callback));
    }

    /// Set callback for refreshing plugins list
    pub fn on_refresh_plugins<F>(&mut self, callback: F)
    where
        F: Fn(&mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_refresh_plugins = Some(Arc::new(callback));
    }

    /// Set the list of plugins
    pub fn set_plugins(&mut self, plugins: Vec<PluginInfo>, cx: &mut Context<Self>) {
        *self.plugins.write() = plugins;
        cx.notify();
    }

    /// Update a plugin's enabled state
    pub fn set_plugin_enabled(&mut self, plugin_id: &str, enabled: bool, cx: &mut Context<Self>) {
        let mut plugins = self.plugins.write();
        if let Some(plugin) = plugins.iter_mut().find(|p| p.id == plugin_id) {
            plugin.enabled = enabled;
        }
        drop(plugins);
        cx.notify();
    }

    /// Notify TTS enabled change
    fn notify_tts_enabled_change(&self, window: &mut Window, cx: &mut App) {
        if let Some(ref callback) = self.on_tts_enabled_change {
            let settings = self.settings.read();
            callback(
                settings.tts_enabled,
                settings.gift_tts,
                settings.sc_tts,
                window,
                cx,
            );
        }
    }

    /// Notify window settings change
    fn notify_window_settings_change(&self, window: &mut Window, cx: &mut App) {
        if let Some(ref callback) = self.on_window_settings_change {
            let settings = self.settings.read();
            callback(
                settings.lite_mode,
                settings.medal_display,
                settings.interact_display,
                settings.guard_effect,
                settings.level_effect,
                window,
                cx,
            );
        }
    }

    pub fn set_rtmp_info(&mut self, addr: String, code: String, cx: &mut Context<Self>) {
        *self.rtmp_info.write() = Some(RtmpInfo { addr, code });
        cx.notify();
    }

    pub fn clear_rtmp_info(&mut self, cx: &mut Context<Self>) {
        *self.rtmp_info.write() = None;
        cx.notify();
    }

    /// Update login status
    pub fn set_login_status(
        &mut self,
        logged_in: bool,
        user_info: Option<UserInfoData>,
        cx: &mut Context<Self>,
    ) {
        let mut account = self.account.write();
        account.logged_in = logged_in;
        account.user_info = user_info;
        if logged_in {
            account.qr_dialog_open = false;
            account.qr_url = None;
            account.qr_status = None;
        }
        drop(account);
        cx.notify();
    }

    /// Update QR code info
    pub fn set_qr_code(&mut self, url: String, cx: &mut Context<Self>) {
        // Update QR code view
        self.qr_code_view.update(cx, |view, cx| {
            view.set_data(url.clone(), cx);
        });

        let mut account = self.account.write();
        account.qr_url = Some(url);
        account.qr_dialog_open = true;
        account.qr_status = Some(QrCodeStatus::NeedScan);
        drop(account);
        cx.notify();
    }

    /// Update QR code status
    pub fn set_qr_status(&mut self, status: QrCodeStatus, cx: &mut Context<Self>) {
        let mut account = self.account.write();
        account.qr_status = Some(status);
        if status == QrCodeStatus::Expired {
            account.qr_dialog_open = false;
        }
        drop(account);
        cx.notify();
    }

    /// Load config values
    pub fn load_config(&mut self, config: ConfigValues, cx: &mut Context<Self>) {
        let mut settings = self.settings.write();
        settings.always_on_top = config.always_on_top;
        settings.guard_effect = config.guard_effect;
        settings.level_effect = config.level_effect;
        settings.opacity = config.opacity;
        settings.lite_mode = config.lite_mode;
        settings.medal_display = config.medal_display;
        settings.interact_display = config.interact_display;
        settings.theme = config.theme;
        settings.font_size = config.font_size;
        settings.tts_enabled = config.tts_enabled;
        settings.gift_tts = config.tts_gift_enabled;
        settings.sc_tts = config.tts_sc_enabled;
        settings.tts_volume = config.tts_volume;
        drop(settings);
        cx.notify();
    }

    /// Set settings from config
    pub fn set_config(&mut self, config: ConfigValues, _cx: &mut Context<Self>) {
        let mut settings = self.settings.write();
        settings.always_on_top = config.always_on_top;
        settings.guard_effect = config.guard_effect;
        settings.level_effect = config.level_effect;
        settings.opacity = config.opacity;
        settings.lite_mode = config.lite_mode;
        settings.medal_display = config.medal_display;
        settings.interact_display = config.interact_display;
        settings.theme = config.theme;
        settings.font_size = config.font_size;
        settings.tts_enabled = config.tts_enabled;
        settings.gift_tts = config.tts_gift_enabled;
        settings.sc_tts = config.tts_sc_enabled;
        settings.tts_volume = config.tts_volume;
    }

    /// Set opacity
    pub fn set_opacity(&mut self, opacity: f32, cx: &mut Context<Self>) {
        self.settings.write().opacity = opacity;
        cx.notify();
    }

    /// Get opacity
    pub fn get_opacity(&self) -> f32 {
        self.settings.read().opacity
    }

    /// Get current settings
    pub fn get_settings(&self) -> SettingsData {
        self.settings.read().clone()
    }

    fn render_section_title(&self, title: &str) -> impl IntoElement {
        div()
            .w_full()
            .pb_3()
            .mb_4()
            .border_b_1()
            .border_color(Colors::bg_hover())
            .child(
                div()
                    .text_size(px(15.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(Colors::text_primary())
                    .child(title.to_string()),
            )
    }

    fn render_section_card(&self, content: impl IntoElement) -> impl IntoElement {
        div()
            .w_full()
            .p_5()
            .rounded(px(8.0))
            .bg(Colors::bg_secondary())
            .border_1()
            .border_color(Colors::bg_hover())
            .child(content)
    }

    fn render_setting_row(
        &self,
        label: &str,
        description: &str,
        control: impl IntoElement,
    ) -> impl IntoElement {
        h_flex()
            .w_full()
            .py_2()
            .items_center()
            .justify_between()
            .child(
                v_flex()
                    .flex_1()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(Colors::text_primary())
                            .child(label.to_string()),
                    )
                    .when(!description.is_empty(), |this| {
                        this.child(
                            div()
                                .text_size(px(11.0))
                                .text_color(Colors::text_muted())
                                .child(description.to_string()),
                        )
                    }),
            )
            .child(control)
    }

    fn render_account_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let account = self.account.read().clone();
        let entity = cx.entity().clone();

        self.render_section_card(
            v_flex()
                .w_full()
                .child(self.render_section_title("账号设置"))
                .child(if account.logged_in {
                // Logged in state
                let user_info = account.user_info.clone();
                let on_logout = self.on_logout.clone();

                h_flex()
                    .w_full()
                    .py_3()
                    .gap_4()
                    .items_center()
                    // User avatar
                    .child({
                        let avatar_url = user_info.as_ref().and_then(|u| {
                            if u.face.is_empty() {
                                None
                            } else {
                                Some(u.face.clone())
                            }
                        });

                        if let Some(url) = avatar_url {
                            img(url)
                                .size(px(56.0))
                                .rounded_full()
                                .object_fit(gpui::ObjectFit::Cover)
                                .into_any_element()
                        } else {
                            // Fallback to initial letter
                            div()
                                .size(px(56.0))
                                .rounded_full()
                                .bg(Colors::accent())
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(24.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(Colors::text_primary())
                                .child(
                                    user_info
                                        .as_ref()
                                        .map(|u| u.name.chars().next().unwrap_or('?').to_string())
                                        .unwrap_or_else(|| "?".to_string()),
                                )
                                .into_any_element()
                        }
                    })
                    // User info
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(Colors::text_primary())
                                    .child(
                                        user_info
                                            .as_ref()
                                            .map(|u| u.name.clone())
                                            .unwrap_or_else(|| "用户".to_string()),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(Colors::text_muted())
                                    .child(format!(
                                        "UID: {}",
                                        user_info.as_ref().map(|u| u.mid).unwrap_or(0)
                                    )),
                            )
                            .child(
                                div()
                                    .id("logout-link")
                                    .text_size(px(12.0))
                                    .text_color(Colors::accent())
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(Colors::error()))
                                    .child("注销登录")
                                    .on_click(move |_event, window, cx| {
                                        if let Some(ref callback) = on_logout {
                                            callback(window, cx);
                                        }
                                    }),
                            ),
                    )
                    .into_any_element()
            } else if account.qr_dialog_open {
                // QR code scanning state
                let status_text = match account.qr_status {
                    Some(QrCodeStatus::NeedScan) => "请使用 bilibili App 扫描二维码",
                    Some(QrCodeStatus::NeedConfirm) => "已扫码，请在手机上确认登录",
                    Some(QrCodeStatus::Success) => "登录成功！",
                    Some(QrCodeStatus::Expired) => "二维码已过期，请重新获取",
                    Some(QrCodeStatus::Error) => "登录出错，请重试",
                    None => "正在生成二维码...",
                };

                let status_color = match account.qr_status {
                    Some(QrCodeStatus::NeedConfirm) => Colors::warning(),
                    Some(QrCodeStatus::Success) => Colors::success(),
                    Some(QrCodeStatus::Expired) | Some(QrCodeStatus::Error) => Colors::error(),
                    _ => Colors::text_secondary(),
                };

                let on_qr_login = self.on_qr_login.clone();

                v_flex()
                    .w_full()
                    .gap_4()
                    .items_center()
                    // QR code display
                    .child(self.qr_code_view.clone())
                    // Status text
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(status_color)
                            .child(status_text),
                    )
                    // Action buttons
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .id("cancel-qr-btn")
                                    .px_4()
                                    .py_2()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .bg(Colors::bg_hover())
                                    .hover(|s| s.bg(Colors::bg_secondary()))
                                    .text_size(px(12.0))
                                    .text_color(Colors::text_secondary())
                                    .child("取消")
                                    .on_click({
                                        let account = self.account.clone();
                                        move |_event, _window, cx| {
                                            account.write().qr_dialog_open = false;
                                            entity.update(cx, |_, cx| cx.notify());
                                        }
                                    }),
                            )
                            .when(
                                matches!(
                                    account.qr_status,
                                    Some(QrCodeStatus::Expired) | Some(QrCodeStatus::Error)
                                ),
                                |this| {
                                    this.child(
                                        div()
                                            .id("refresh-qr-btn")
                                            .px_4()
                                            .py_2()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .bg(Colors::accent())
                                            .hover(|s| s.opacity(0.8))
                                            .text_size(px(12.0))
                                            .text_color(Colors::button_text())
                                            .child("重新获取")
                                            .on_click(move |_event, window, cx| {
                                                if let Some(ref callback) = on_qr_login {
                                                    callback(window, cx);
                                                }
                                            }),
                                    )
                                },
                            ),
                    )
                    .into_any_element()
            } else {
                // Not logged in state - show login prompt
                let on_qr_login = self.on_qr_login.clone();

                v_flex()
                    .w_full()
                    .py_4()
                    .gap_3()
                    .items_center()
                    // Icon placeholder
                    .child(
                        div()
                            .size(px(64.0))
                            .rounded_full()
                            .bg(Colors::bg_hover())
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(28.0))
                            .text_color(Colors::text_muted())
                            .child("👤"),
                    )
                    // Description
                    .child(
                        v_flex()
                            .gap_1()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .text_color(Colors::text_primary())
                                    .child("登录 bilibili 账号"),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(Colors::text_muted())
                                    .child("登录后可发送弹幕、查看更多信息"),
                            ),
                    )
                    // Login button
                    .child(
                        div()
                            .id("qr-login-btn")
                            .mt_2()
                            .px_6()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .bg(Colors::accent())
                            .hover(|s| s.opacity(0.8))
                            .text_size(px(13.0))
                            .text_color(Colors::button_text())
                            .child("扫码登录")
                            .on_click(move |_event, window, cx| {
                                if let Some(ref callback) = on_qr_login {
                                    callback(window, cx);
                                }
                            }),
                    )
                    .into_any_element()
            }),
        )
    }

    fn render_room_section(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let room_input_state = self.room_input.clone();
        let room_input = room_input_state.read().clone();
        let on_change_room = self.on_change_room.clone();
        let current_room_id = self.room_id;

        // Create InputState using keyed state - returns a wrapper struct
        struct RoomInputWrapper {
            input: Entity<gpui_component::input::InputState>,
        }

        let state =
            window.use_keyed_state(SharedString::from("room-input-state"), cx, |window, cx| {
                let initial_value = current_room_id.map(|id| id.to_string()).unwrap_or_default();

                let input = cx.new(|cx| {
                    gpui_component::input::InputState::new(window, cx)
                        .placeholder("输入房间号...")
                        .default_value(initial_value)
                });

                RoomInputWrapper { input }
            });

        let input_state = state.read(cx).input.clone();

        self.render_section_card(
            v_flex()
                .w_full()
                .child(self.render_section_title("直播间设置"))
            .child(
                v_flex()
                    .w_full()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(Colors::text_primary())
                            .child("房间号"),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(Colors::text_muted())
                            .child("输入直播间的数字 ID"),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2()
                            .items_center()
                            .child(div().flex_1().child(
                                gpui_component::input::Input::new(&input_state).cleanable(true),
                            ))
                            .child(
                                div()
                                    .id("confirm-room-btn")
                                    .px_4()
                                    .py(px(7.0))
                                    .rounded_md()
                                    .cursor_pointer()
                                    .bg(Colors::accent())
                                    .hover(|s| s.opacity(0.8))
                                    .text_size(px(13.0))
                                    .text_color(Colors::button_text())
                                    .child("确认")
                                    .on_click({
                                        let input_state = input_state.clone();
                                        move |_event, window, cx| {
                                            let text = input_state.read(cx).text().to_string();
                                            if let Ok(room_id) = text.parse::<u64>() {
                                                if let Some(ref callback) = on_change_room {
                                                    callback(room_id, window, cx);
                                                }
                                            }
                                        }
                                    }),
                            ),
                    )
                    .when(room_input.error, |this| {
                        this.child(
                            div()
                                .text_size(px(11.0))
                                .text_color(Colors::error())
                                .child("房间号无效，请检查输入"),
                        )
                    }),
            ),
        )
    }

    fn render_live_controls_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let on_start_live = self.on_start_live.clone();
        let on_stop_live = self.on_stop_live.clone();
        let rtmp_info = self.rtmp_info.read().clone();
        let room_id = self.room_id;
        let is_live = rtmp_info.is_some();

        self.render_section_card(
            v_flex()
                .w_full()
                .child(self.render_section_title("直播控制"))
                .child(
                    v_flex()
                        .w_full()
                        .gap_4()
                        // Status indicator
                        .child(
                            h_flex()
                                .w_full()
                                .px_4()
                                .py_3()
                                .rounded(px(8.0))
                                .bg(if is_live {
                                    Colors::success().opacity(0.1)
                                } else {
                                    Colors::bg_hover()
                                })
                                .border_1()
                                .border_color(if is_live {
                                    Colors::success().opacity(0.3)
                                } else {
                                    Colors::bg_hover()
                                })
                                .items_center()
                                .gap_3()
                                .child(
                                    // Status dot with pulse effect
                                    div()
                                        .size(px(10.0))
                                        .rounded_full()
                                        .bg(if is_live {
                                            Colors::success()
                                        } else {
                                            Colors::text_muted()
                                        }),
                                )
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .gap(px(2.0))
                                        .child(
                                            div()
                                                .text_size(px(13.0))
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(Colors::text_primary())
                                                .child(if is_live { "直播中" } else { "未开播" }),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(11.0))
                                                .text_color(Colors::text_muted())
                                                .child(if is_live {
                                                    "推流信息已就绪"
                                                } else {
                                                    "点击下方按钮开始直播"
                                                }),
                                        ),
                                ),
                        )
                        // Control buttons
                        .child(
                            h_flex()
                                .w_full()
                                .gap_3()
                                // Start button - outlined style when not live, solid when can start
                                .child(
                                    div()
                                        .id("start-live-btn")
                                        .flex_1()
                                        .px_4()
                                        .py(px(10.0))
                                        .rounded(px(8.0))
                                        .cursor_pointer()
                                        .border_1()
                                        .when(!is_live, |this| {
                                            this.bg(Colors::accent())
                                                .border_color(Colors::accent())
                                                .hover(|s| s.opacity(0.9))
                                        })
                                        .when(is_live, |this| {
                                            this.bg(gpui::transparent_black())
                                                .border_color(Colors::text_muted().opacity(0.3))
                                                .hover(|s| s.bg(Colors::bg_hover()))
                                        })
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(if is_live {
                                            Colors::text_muted()
                                        } else {
                                            Colors::text_primary()
                                        })
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .gap_2()
                                        // Play icon (circle outline)
                                        .child(
                                            div()
                                                .size(px(16.0))
                                                .rounded_full()
                                                .border_1()
                                                .border_color(if is_live {
                                                    Colors::text_muted()
                                                } else {
                                                    Colors::text_primary()
                                                })
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(
                                                    // Inner triangle approximation using a small rotated square
                                                    div()
                                                        .ml(px(1.0))
                                                        .w(px(5.0))
                                                        .h(px(6.0))
                                                        .bg(if is_live {
                                                            Colors::text_muted()
                                                        } else {
                                                            Colors::text_primary()
                                                        }),
                                                ),
                                        )
                                        .child("开始直播")
                                        .on_click(cx.listener(move |_view, _event, window, cx| {
                                            if let Some(ref callback) = on_start_live {
                                                if let Some(room_id) = room_id {
                                                    callback(room_id, window, cx);
                                                }
                                            }
                                        })),
                                )
                                // Stop button - outlined style when live, disabled when not
                                .child(
                                    div()
                                        .id("stop-live-btn")
                                        .flex_1()
                                        .px_4()
                                        .py(px(10.0))
                                        .rounded(px(8.0))
                                        .cursor_pointer()
                                        .border_1()
                                        .when(is_live, |this| {
                                            this.bg(gpui::transparent_black())
                                                .border_color(Colors::error().opacity(0.5))
                                                .text_color(Colors::error())
                                                .hover(|s| s.bg(Colors::error().opacity(0.1)))
                                        })
                                        .when(!is_live, |this| {
                                            this.bg(gpui::transparent_black())
                                                .border_color(Colors::text_muted().opacity(0.3))
                                                .text_color(Colors::text_muted())
                                                .hover(|s| s.bg(Colors::bg_hover()))
                                        })
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .gap_2()
                                        // Stop icon (square)
                                        .child(
                                            div()
                                                .size(px(10.0))
                                                .rounded(px(2.0))
                                                .bg(if is_live {
                                                    Colors::error()
                                                } else {
                                                    Colors::text_muted()
                                                }),
                                        )
                                        .child("停止直播")
                                        .on_click(cx.listener(move |_view, _event, window, cx| {
                                            if let Some(ref callback) = on_stop_live {
                                                callback(window, cx);
                                            }
                                        })),
                                ),
                        )
                        // RTMP info section - only shown when live
                        .when(rtmp_info.is_some(), |this| {
                            let info = rtmp_info.unwrap();
                            this.child(
                                v_flex()
                                    .w_full()
                                    .mt_1()
                                    .p_4()
                                    .rounded(px(8.0))
                                    .bg(Colors::bg_hover().opacity(0.5))
                                    .gap_4()
                                    // Header
                                    .child(
                                        h_flex()
                                            .w_full()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                // Broadcast icon
                                                div()
                                                    .size(px(16.0))
                                                    .rounded_full()
                                                    .bg(Colors::accent().opacity(0.2))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        div()
                                                            .size(px(6.0))
                                                            .rounded_full()
                                                            .bg(Colors::accent()),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(12.0))
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(Colors::text_primary())
                                                    .child("推流信息"),
                                            ),
                                    )
                                    // Server address
                                    .child(
                                        v_flex()
                                            .w_full()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_size(px(11.0))
                                                    .text_color(Colors::text_muted())
                                                    .child("服务器地址 (RTMP URL)"),
                                            )
                                            .child(
                                                h_flex()
                                                    .w_full()
                                                    .gap_2()
                                                    .items_center()
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .px_3()
                                                            .py_2()
                                                            .rounded(px(6.0))
                                                            .bg(Colors::bg_primary())
                                                            .border_1()
                                                            .border_color(Colors::bg_hover())
                                                            .text_size(px(11.0))
                                                            .text_color(Colors::text_secondary())
                                                            .overflow_hidden()
                                                            .text_ellipsis()
                                                            .child(info.addr.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .id("copy-rtmp-addr-btn")
                                                            .px_3()
                                                            .py_2()
                                                            .rounded(px(6.0))
                                                            .cursor_pointer()
                                                            .bg(gpui::transparent_black())
                                                            .border_1()
                                                            .border_color(Colors::accent().opacity(0.5))
                                                            .text_color(Colors::accent())
                                                            .hover(|s| s.bg(Colors::accent().opacity(0.1)))
                                                            .text_size(px(11.0))
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .child("复制")
                                                            .on_click({
                                                                let addr = info.addr.clone();
                                                                move |_event, _window, cx| {
                                                                    cx.write_to_clipboard(
                                                                        ClipboardItem::new_string(addr.clone()),
                                                                    );
                                                                }
                                                            }),
                                                    ),
                                            ),
                                    )
                                    // Stream key
                                    .child(
                                        v_flex()
                                            .w_full()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_size(px(11.0))
                                                    .text_color(Colors::text_muted())
                                                    .child("推流密钥 (Stream Key)"),
                                            )
                                            .child(
                                                h_flex()
                                                    .w_full()
                                                    .gap_2()
                                                    .items_center()
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .px_3()
                                                            .py_2()
                                                            .rounded(px(6.0))
                                                            .bg(Colors::bg_primary())
                                                            .border_1()
                                                            .border_color(Colors::bg_hover())
                                                            .text_size(px(11.0))
                                                            .text_color(Colors::text_secondary())
                                                            .overflow_hidden()
                                                            .text_ellipsis()
                                                            .child(info.code.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .id("copy-rtmp-code-btn")
                                                            .px_3()
                                                            .py_2()
                                                            .rounded(px(6.0))
                                                            .cursor_pointer()
                                                            .bg(gpui::transparent_black())
                                                            .border_1()
                                                            .border_color(Colors::accent().opacity(0.5))
                                                            .text_color(Colors::accent())
                                                            .hover(|s| s.bg(Colors::accent().opacity(0.1)))
                                                            .text_size(px(11.0))
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .child("复制")
                                                            .on_click({
                                                                let code = info.code.clone();
                                                                move |_event, _window, cx| {
                                                                    cx.write_to_clipboard(
                                                                        ClipboardItem::new_string(code.clone()),
                                                                    );
                                                                }
                                                            }),
                                                    ),
                                            ),
                                    ),
                            )
                        }),
                ),
        )
    }

    fn render_tab_content(
        &self,
        tab_index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tab = match tab_index {
            0 => SettingsTab::Basic,
            1 => SettingsTab::Window,
            2 => SettingsTab::Appearance,
            3 => SettingsTab::Tts,
            4 => SettingsTab::Plugin,
            5 => SettingsTab::Advanced,
            6 => SettingsTab::About,
            _ => SettingsTab::Basic,
        };

        match tab {
            SettingsTab::Basic => self.render_basic_tab(window, cx).into_any_element(),
            SettingsTab::Window => self.render_window_tab(cx).into_any_element(),
            SettingsTab::Appearance => self.render_appearance_tab(window, cx).into_any_element(),
            SettingsTab::Tts => self.render_tts_tab(cx, window).into_any_element(),
            SettingsTab::Plugin => self.render_plugin_tab(cx).into_any_element(),
            SettingsTab::Advanced => self.render_advanced_tab(cx).into_any_element(),
            SettingsTab::About => self.render_about_tab(cx).into_any_element(),
        }
    }

    fn render_basic_tab(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_owner = self.is_room_owner();

        v_flex()
            .w_full()
            .p_6()
            .gap_4()
            .child(self.render_account_section(cx))
            .child(self.render_room_section(window, cx))
            .when(is_owner, |this| {
                this.child(self.render_live_controls_section(cx))
            })
            .child(self.render_merge_section(window, cx))
    }

    fn render_merge_section(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let merge_settings = self.merge_settings.clone();
        let merge_enabled = merge_settings.read().enabled;
        let merge_rooms = merge_settings.read().rooms.clone();
        let entity = cx.entity().clone();

        // Room colors for visual distinction
        let room_colors = [
            hsla(0.6, 0.7, 0.5, 1.0),   // Blue
            hsla(0.3, 0.7, 0.5, 1.0),   // Green
            hsla(0.08, 0.7, 0.5, 1.0),  // Orange
            hsla(0.85, 0.7, 0.5, 1.0),  // Pink
            hsla(0.5, 0.7, 0.5, 1.0),   // Cyan
        ];

        // Create input state for adding rooms
        struct MergeRoomInputWrapper {
            input: Entity<gpui_component::input::InputState>,
        }

        let state = window.use_keyed_state(
            SharedString::from("merge-room-input-state"),
            cx,
            |window, cx| {
                let input = cx.new(|cx| {
                    gpui_component::input::InputState::new(window, cx)
                        .placeholder("输入房间号...")
                });
                MergeRoomInputWrapper { input }
            },
        );

        let input_state = state.read(cx).input.clone();

        self.render_section_card(
            v_flex()
                .w_full()
                .child(self.render_section_title("弹幕聚合"))
                .child(
                    v_flex()
                        .w_full()
                        .gap_3()
                        .child(self.render_setting_row(
                            "启用弹幕聚合",
                            "聚合多个直播间的弹幕到主窗口",
                            Switch::new("merge_enabled").checked(merge_enabled).on_click({
                                let merge_settings = merge_settings.clone();
                                let entity = entity.clone();
                                move |checked: &bool, _window, cx| {
                                    merge_settings.write().enabled = *checked;
                                    entity.update(cx, |_, cx| cx.notify());
                                }
                            }),
                        ))
                        .when(merge_enabled, |this| {
                            this.child(
                                v_flex()
                                    .w_full()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .text_color(Colors::text_muted())
                                            .child("副房间列表（最多5个）"),
                                    )
                                    // Room list
                                    .child(
                                        v_flex()
                                            .w_full()
                                            .gap_1()
                                            .children(merge_rooms.iter().enumerate().map(|(idx, room_id)| {
                                                let color = room_colors[idx % room_colors.len()];
                                                let room_id_str = room_id.to_string();
                                                let merge_settings = merge_settings.clone();
                                                let entity = entity.clone();
                                                let room_to_remove = *room_id;

                                                h_flex()
                                                    .w_full()
                                                    .px_3()
                                                    .py_2()
                                                    .rounded(px(6.0))
                                                    .bg(Colors::bg_secondary())
                                                    .items_center()
                                                    .justify_between()
                                                    .child(
                                                        h_flex()
                                                            .gap_2()
                                                            .items_center()
                                                            .child(
                                                                div()
                                                                    .size(px(8.0))
                                                                    .rounded_full()
                                                                    .bg(color),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_size(px(13.0))
                                                                    .text_color(Colors::text_primary())
                                                                    .child(room_id_str),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .id(SharedString::from(format!("remove-merge-room-{}", idx)))
                                                            .px_2()
                                                            .py_1()
                                                            .rounded(px(4.0))
                                                            .cursor_pointer()
                                                            .text_size(px(11.0))
                                                            .text_color(Colors::error())
                                                            .hover(|s| s.bg(Colors::error().opacity(0.1)))
                                                            .child("移除")
                                                            .on_click(move |_event, _window, cx| {
                                                                let mut settings = merge_settings.write();
                                                                settings.rooms.retain(|&r| r != room_to_remove);
                                                                entity.update(cx, |_, cx| cx.notify());
                                                            }),
                                                    )
                                            })),
                                    )
                                    // Add room input
                                    .when(merge_rooms.len() < 5, |this| {
                                        let merge_settings = merge_settings.clone();
                                        let entity = entity.clone();
                                        let input_state_for_click = input_state.clone();

                                        this.child(
                                            h_flex()
                                                .w_full()
                                                .gap_2()
                                                .items_center()
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .child(gpui_component::input::Input::new(&input_state).cleanable(true)),
                                                )
                                                .child(
                                                    div()
                                                        .id("add-merge-room-btn")
                                                        .px_3()
                                                        .py(px(7.0))
                                                        .rounded(px(6.0))
                                                        .cursor_pointer()
                                                        .bg(Colors::accent())
                                                        .hover(|s| s.opacity(0.8))
                                                        .text_size(px(12.0))
                                                        .text_color(Colors::button_text())
                                                        .child("添加")
                                                        .on_click(move |_event, _window, cx| {
                                                            let text = input_state_for_click.read(cx).text().to_string();
                                                            if let Ok(room_id) = text.parse::<u64>() {
                                                                let mut settings = merge_settings.write();
                                                                if settings.rooms.len() < 5 && !settings.rooms.contains(&room_id) {
                                                                    settings.rooms.push(room_id);
                                                                }
                                                            }
                                                            entity.update(cx, |_, cx| cx.notify());
                                                        }),
                                                ),
                                        )
                                    }),
                            )
                        }),
                ),
        )
    }

    fn render_window_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = self.settings.clone();
        let lite_mode = settings.read().lite_mode;
        let medal_display = settings.read().medal_display;
        let interact_display = settings.read().interact_display;
        let guard_effect = settings.read().guard_effect;
        let level_effect = settings.read().level_effect;

        let entity = cx.entity().clone();
        let entity2 = cx.entity().clone();
        let entity3 = cx.entity().clone();
        let entity4 = cx.entity().clone();
        let entity5 = cx.entity().clone();

        v_flex()
            .w_full()
            .p_6()
            .gap_4()
            .child(
                self.render_section_card(
                    v_flex()
                        .w_full()
                        .child(self.render_section_title("弹幕窗口设置"))
                        .child(self.render_setting_row(
                            "精简模式",
                            "简化弹幕窗口显示",
                            Switch::new("lite_mode").checked(lite_mode).on_click({
                                let settings = settings.clone();
                                move |checked: &bool, window, cx| {
                                    settings.write().lite_mode = *checked;
                                    entity.update(cx, |view, cx| {
                                        view.notify_window_settings_change(window, cx);
                                        cx.notify();
                                    });
                                }
                            }),
                        ))
                        .child(self.render_setting_row(
                            "勋章显示",
                            "显示用户的粉丝勋章",
                            Switch::new("medal_display").checked(medal_display).on_click({
                                let settings = settings.clone();
                                move |checked: &bool, window, cx| {
                                    settings.write().medal_display = *checked;
                                    entity2.update(cx, |view, cx| {
                                        view.notify_window_settings_change(window, cx);
                                        cx.notify();
                                    });
                                }
                            }),
                        ))
                        .child(self.render_setting_row(
                            "交互信息",
                            "显示普通用户进入和关注通知",
                            Switch::new("interact_display").checked(interact_display).on_click({
                                let settings = settings.clone();
                                move |checked: &bool, window, cx| {
                                    settings.write().interact_display = *checked;
                                    entity3.update(cx, |view, cx| {
                                        view.notify_window_settings_change(window, cx);
                                        cx.notify();
                                    });
                                }
                            }),
                        )),
                ),
            )
            .child(
                self.render_section_card(
                    v_flex()
                        .w_full()
                        .child(self.render_section_title("入场特效"))
                        .child(self.render_setting_row(
                            "舰长特效",
                            "显示大航海入场特效",
                            Switch::new("guard_effect").checked(guard_effect).on_click({
                                let settings = settings.clone();
                                move |checked: &bool, window, cx| {
                                    settings.write().guard_effect = *checked;
                                    entity4.update(cx, |view, cx| {
                                        view.notify_window_settings_change(window, cx);
                                        cx.notify();
                                    });
                                }
                            }),
                        ))
                        .child(self.render_setting_row(
                            "荣耀等级特效",
                            "显示荣耀等级入场特效",
                            Switch::new("level_effect").checked(level_effect).on_click({
                                let settings = settings.clone();
                                move |checked: &bool, window, cx| {
                                    settings.write().level_effect = *checked;
                                    entity5.update(cx, |view, cx| {
                                        view.notify_window_settings_change(window, cx);
                                        cx.notify();
                                    });
                                }
                            }),
                        )),
                ),
            )
    }

    fn render_appearance_tab(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .w_full()
            .p_6()
            .gap_4()
            .child(self.render_danmu_style_section(window, cx))
            .child(self.render_window_appearance_section(window, cx))
    }

    fn render_danmu_style_section(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let settings = self.settings.clone();
        let current_font_size = settings.read().font_size;

        // Create font size slider state
        struct FontSizeSliderWrapper {
            slider: Entity<SliderState>,
            subscribed: bool,
        }

        let initial_font_size = current_font_size;
        let font_size_slider_state = window.use_keyed_state(
            SharedString::from("font-size-slider-state"),
            cx,
            move |_window, cx| {
                let slider = cx.new(|_| {
                    SliderState::new()
                        .min(8.0)
                        .max(64.0)
                        .step(1.0)
                        .default_value(initial_font_size)
                });
                FontSizeSliderWrapper {
                    slider,
                    subscribed: false,
                }
            },
        );

        let slider_state = font_size_slider_state.read(cx).slider.clone();
        let is_subscribed = font_size_slider_state.read(cx).subscribed;

        // Sync slider value with current font size
        slider_state.update(cx, |state, cx| {
            let slider_value = state.value().start();
            if (slider_value - current_font_size).abs() > 0.1 {
                state.set_value(current_font_size, window, cx);
            }
        });

        // Subscribe to slider change events only once
        if !is_subscribed {
            let settings_for_slider = self.settings.clone();
            let on_font_size_change = self.on_font_size_change.clone();

            cx.subscribe_in(
                &slider_state,
                window,
                move |_view, _, event: &SliderEvent, window, cx| {
                    let SliderEvent::Change(value) = event;
                    let font_size = value.start();
                    settings_for_slider.write().font_size = font_size;
                    if let Some(ref callback) = on_font_size_change {
                        callback(font_size, window, cx);
                    }
                    cx.notify();
                },
            )
            .detach();

            font_size_slider_state.write(
                cx,
                FontSizeSliderWrapper {
                    slider: slider_state.clone(),
                    subscribed: true,
                },
            );
        }

        self.render_section_card(
            v_flex()
                .w_full()
                .child(self.render_section_title("弹幕设置"))
                .child(
                    v_flex()
                        .w_full()
                        .py_2()
                        .gap_4()
                        // Font size setting
                        .child(
                            v_flex()
                                .w_full()
                                .gap_2()
                                .child(
                                    h_flex()
                                        .w_full()
                                        .justify_between()
                                        .child(
                                            v_flex()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .text_size(px(13.0))
                                                        .text_color(Colors::text_primary())
                                                        .child("字体大小"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(11.0))
                                                        .text_color(Colors::text_muted())
                                                        .child("调整弹幕字体大小 (8px - 64px)"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(12.0))
                                                .text_color(Colors::text_secondary())
                                                .child(format!("{}px", current_font_size as i32)),
                                        ),
                                )
                                .child(div().w_full().child(Slider::new(&slider_state))),
                        ),
                ),
        )
    }

    fn render_window_appearance_section(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let settings = self.settings.clone();
        let current_theme = settings.read().theme.clone();
        let current_opacity = settings.read().opacity;
        let entity = cx.entity().clone();
        let on_theme_change = self.on_theme_change.clone();

        let themes = vec![
            ("light", "浅色"),
            ("dark", "深色"),
            ("dracula", "Dracula"),
            ("catppuccin", "Catppuccin"),
            ("blueberry", "蓝莓"),
            ("ayu-light", "Ayu 浅色"),
            ("ayu-dark", "Ayu 深色"),
        ];

        // Create opacity slider state
        struct OpacitySliderWrapper {
            slider: Entity<SliderState>,
            subscribed: bool,
        }

        let initial_opacity = current_opacity;
        let opacity_slider_state = window.use_keyed_state(
            SharedString::from("appearance-opacity-slider-state"),
            cx,
            move |_window, cx| {
                let slider = cx.new(|_| {
                    SliderState::new()
                        .min(0.0)
                        .max(1.0)
                        .step(0.01)
                        .default_value(initial_opacity)
                });
                OpacitySliderWrapper {
                    slider,
                    subscribed: false,
                }
            },
        );

        let slider_state = opacity_slider_state.read(cx).slider.clone();
        let is_subscribed = opacity_slider_state.read(cx).subscribed;

        // Sync slider value with current opacity
        slider_state.update(cx, |state, cx| {
            let slider_value = state.value().start();
            if (slider_value - current_opacity).abs() > 0.001 {
                state.set_value(current_opacity, window, cx);
            }
        });

        // Subscribe to slider change events only once
        if !is_subscribed {
            let settings_for_slider = self.settings.clone();
            let on_opacity_change = self.on_opacity_change.clone();

            cx.subscribe_in(
                &slider_state,
                window,
                move |_view, _, event: &SliderEvent, window, cx| {
                    let SliderEvent::Change(value) = event;
                    let opacity = value.start();
                    settings_for_slider.write().opacity = opacity;
                    if let Some(ref callback) = on_opacity_change {
                        callback(opacity, window, cx);
                    }
                    cx.notify();
                },
            )
            .detach();

            opacity_slider_state.write(
                cx,
                OpacitySliderWrapper {
                    slider: slider_state.clone(),
                    subscribed: true,
                },
            );
        }

        self.render_section_card(
            v_flex()
                .w_full()
                .child(self.render_section_title("窗口设置"))
                // Theme selection
                .child(
                    v_flex()
                        .w_full()
                        .py_2()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(Colors::text_primary())
                                .child("主题"),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(Colors::text_muted())
                                .child("选择窗口主题颜色"),
                        )
                        .child(
                            h_flex()
                                .w_full()
                                .gap_2()
                                .flex_wrap()
                                .children(themes.into_iter().map(|(id, name)| {
                                    let is_selected = id == current_theme;
                                    let settings = settings.clone();
                                    let entity = entity.clone();
                                    let theme_id = id.to_string();
                                    let on_theme_change = on_theme_change.clone();

                                    div()
                                        .id(SharedString::from(format!("theme-{}", id)))
                                        .px_3()
                                        .py_2()
                                        .rounded(px(6.0))
                                        .cursor_pointer()
                                        .border_1()
                                        .when(is_selected, |this| {
                                            this.border_color(Colors::accent())
                                                .bg(Colors::accent().opacity(0.1))
                                        })
                                        .when(!is_selected, |this| {
                                            this.border_color(Colors::bg_hover())
                                                .hover(|s| s.bg(Colors::bg_hover()))
                                        })
                                        .text_size(px(12.0))
                                        .text_color(Colors::text_primary())
                                        .child(name)
                                        .on_click(move |_event, _window, cx| {
                                            settings.write().theme = theme_id.clone();
                                            // Apply theme immediately
                                            crate::theme::set_theme(&theme_id);
                                            crate::theme::update_gpui_component_theme(cx);
                                            if let Some(ref callback) = on_theme_change {
                                                callback(theme_id.clone(), _window, cx);
                                            }
                                            entity.update(cx, |_, cx| cx.notify());
                                        })
                                })),
                        ),
                )
                // Opacity slider
                .child(
                    v_flex()
                        .w_full()
                        .py_2()
                        .gap_2()
                        .child(
                            h_flex()
                                .w_full()
                                .justify_between()
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_size(px(13.0))
                                                .text_color(Colors::text_primary())
                                                .child("透明度"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(11.0))
                                                .text_color(Colors::text_muted())
                                                .child("除设置窗口外，其他窗口的透明度"),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(Colors::text_secondary())
                                        .child(format!("{:.2}", current_opacity)),
                                ),
                        )
                        .child(div().w_full().child(Slider::new(&slider_state))),
                ),
        )
    }

    fn render_tts_tab(&self, cx: &mut Context<Self>, window: &mut Window) -> impl IntoElement {
        let settings = self.settings.clone();
        let tts_enabled = settings.read().tts_enabled;
        let gift_tts = settings.read().gift_tts;
        let sc_tts = settings.read().sc_tts;
        let current_volume = settings.read().tts_volume;
        let entity = cx.entity().clone();
        let entity2 = cx.entity().clone();
        let entity3 = cx.entity().clone();

        let providers = vec![
            ("system", "系统"),
            ("aliyun", "阿里云"),
            ("custom", "自定义"),
        ];

        // Clone callbacks for use in closures
        let on_tts_test = self.on_tts_test.clone();

        // Create volume slider state
        struct VolumeSliderWrapper {
            slider: Entity<SliderState>,
            subscribed: bool,
        }

        let initial_volume = current_volume;
        let volume_slider_state = window.use_keyed_state(
            SharedString::from("tts-volume-slider-state"),
            cx,
            move |_window, cx| {
                let slider = cx.new(|_| {
                    SliderState::new()
                        .min(0.0)
                        .max(1.0)
                        .step(0.01)
                        .default_value(initial_volume)
                });
                VolumeSliderWrapper {
                    slider,
                    subscribed: false,
                }
            },
        );

        let slider_state = volume_slider_state.read(cx).slider.clone();
        let is_subscribed = volume_slider_state.read(cx).subscribed;

        // Sync slider value with current volume
        slider_state.update(cx, |state, cx| {
            let slider_value = state.value().start();
            if (slider_value - current_volume).abs() > 0.001 {
                state.set_value(current_volume, window, cx);
            }
        });

        // Subscribe to slider change events only once
        if !is_subscribed {
            let settings_for_slider = self.settings.clone();
            let on_tts_volume_change = self.on_tts_volume_change.clone();

            cx.subscribe_in(
                &slider_state,
                window,
                move |_view, _, event: &SliderEvent, window, cx| {
                    let SliderEvent::Change(value) = event;
                    let volume = value.start();
                    settings_for_slider.write().tts_volume = volume;
                    if let Some(ref callback) = on_tts_volume_change {
                        callback(volume, window, cx);
                    }
                    cx.notify();
                },
            )
            .detach();

            volume_slider_state.write(
                cx,
                VolumeSliderWrapper {
                    slider: slider_state.clone(),
                    subscribed: true,
                },
            );
        }

        let volume_percent = format!("{}%", (current_volume * 100.0).round() as i32);

        v_flex()
            .w_full()
            .p_6()
            .gap_4()
            .child(
                self.render_section_card(
                    v_flex()
                        .w_full()
                        .child(self.render_section_title("TTS 配置"))
                        .child(
                            v_flex()
                                .w_full()
                                .py_2()
                                .gap_3()
                                .child(
                                    v_flex()
                                        .w_full()
                                        .gap_2()
                                        .child(
                                            h_flex()
                                                .w_full()
                                                .justify_between()
                                                .items_center()
                                                .child(
                                                    v_flex()
                                                        .gap_1()
                                                        .child(
                                                            div()
                                                                .text_size(px(13.0))
                                                                .text_color(Colors::text_primary())
                                                                .child("TTS 音量"),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_size(px(11.0))
                                                                .text_color(Colors::text_muted())
                                                                .child("调整语音播报音量"),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(13.0))
                                                        .text_color(Colors::text_secondary())
                                                        .child(volume_percent),
                                                ),
                                        )
                                        .child(
                                            div().w_full().child(Slider::new(&slider_state)),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("tts-test-btn")
                                        .w_full()
                                        .px_3()
                                        .py_2()
                                        .rounded(px(6.0))
                                        .cursor_pointer()
                                        .bg(Colors::accent())
                                        .hover(|s| s.opacity(0.8))
                                        .text_size(px(13.0))
                                        .text_color(Colors::button_text())
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child("测试 TTS")
                                        .on_click(move |_event, window, cx| {
                                            tracing::info!("TTS test clicked");
                                            if let Some(ref callback) = on_tts_test {
                                                callback(window, cx);
                                            }
                                        }),
                                )
                                .child(
                                    v_flex()
                                        .w_full()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_size(px(13.0))
                                                .text_color(Colors::text_primary())
                                                .child("TTS 声源"),
                                        )
                                        .child(
                                            h_flex()
                                                .w_full()
                                                .gap_2()
                                                .children(providers.into_iter().map(|(id, name)| {
                                                    let is_selected = id == "system";
                                                    div()
                                                        .id(SharedString::from(format!("tts-provider-{}", id)))
                                                        .px_3()
                                                        .py_2()
                                                        .rounded(px(6.0))
                                                        .cursor_pointer()
                                                        .border_1()
                                                        .when(is_selected, |this| {
                                                            this.border_color(Colors::accent())
                                                                .bg(Colors::accent().opacity(0.1))
                                                        })
                                                        .when(!is_selected, |this| {
                                                            this.border_color(Colors::bg_hover())
                                                                .hover(|s| s.bg(Colors::bg_hover()))
                                                        })
                                                        .text_size(px(12.0))
                                                        .text_color(Colors::text_primary())
                                                        .child(name)
                                                        .on_click(cx.listener(move |_this, _event, _window, cx| {
                                                            tracing::info!("TTS provider selected: {}", id);
                                                            cx.notify();
                                                        }))
                                                })),
                                        ),
                                ),
                        ),
                ),
            )
            .child(
                self.render_section_card(
                    v_flex()
                        .w_full()
                        .child(self.render_section_title("语音合成"))
                        .child(self.render_setting_row(
                            "弹幕语音",
                            "朗读弹幕内容",
                            Switch::new("tts_enabled").checked(tts_enabled).on_click({
                                let settings = settings.clone();
                                move |checked: &bool, _window, cx| {
                                    settings.write().tts_enabled = *checked;
                                    entity.update(cx, |this, cx| {
                                        this.notify_tts_enabled_change(_window, cx);
                                        cx.notify();
                                    });
                                }
                            }),
                        ))
                        .child(self.render_setting_row(
                            "礼物语音",
                            "朗读礼物通知",
                            Switch::new("gift_tts").checked(gift_tts).on_click({
                                let settings = settings.clone();
                                move |checked: &bool, _window, cx| {
                                    settings.write().gift_tts = *checked;
                                    entity2.update(cx, |this, cx| {
                                        this.notify_tts_enabled_change(_window, cx);
                                        cx.notify();
                                    });
                                }
                            }),
                        ))
                        .child(self.render_setting_row(
                            "醒目留言语音",
                            "朗读醒目留言内容",
                            Switch::new("sc_tts").checked(sc_tts).on_click({
                                let settings = settings.clone();
                                move |checked: &bool, _window, cx| {
                                    settings.write().sc_tts = *checked;
                                    entity3.update(cx, |this, cx| {
                                        this.notify_tts_enabled_change(_window, cx);
                                        cx.notify();
                                    });
                                }
                            }),
                        )),
                ),
            )
    }

    fn render_plugin_tab(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let plugins = self.plugins.read().clone();
        let on_plugin_open = self.on_plugin_open.clone();
        let on_open_plugins_folder = self.on_open_plugins_folder.clone();
        let on_refresh_plugins = self.on_refresh_plugins.clone();

        v_flex()
            .w_full()
            .p_6()
            .gap_4()
            .child(
                self.render_section_card(
                    v_flex()
                        .w_full()
                        .child(
                            h_flex()
                                .w_full()
                                .justify_between()
                                .items_center()
                                .child(self.render_section_title("已安装插件"))
                                .child(
                                    div()
                                        .id("refresh-plugins-btn")
                                        .px_2()
                                        .py_1()
                                        .rounded(px(4.0))
                                        .cursor_pointer()
                                        .text_size(px(11.0))
                                        .text_color(Colors::text_secondary())
                                        .hover(|s| s.bg(Colors::bg_hover()))
                                        .child("刷新")
                                        .on_click({
                                            let callback = on_refresh_plugins.clone();
                                            move |_event, window, cx| {
                                                if let Some(ref cb) = callback {
                                                    cb(window, cx);
                                                }
                                            }
                                        }),
                                ),
                        )
                        .child(
                            v_flex()
                                .w_full()
                                .py_2()
                                .gap_3()
                                .when(plugins.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .w_full()
                                            .py_8()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                v_flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(
                                                        div()
                                                            .text_size(px(14.0))
                                                            .text_color(Colors::text_muted())
                                                            .child("暂无已安装的插件"),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_size(px(12.0))
                                                            .text_color(Colors::text_muted())
                                                            .child("将插件文件夹放入 plugins 目录后重启应用"),
                                                    ),
                                            ),
                                    )
                                })
                                .children(plugins.iter().map(|plugin| {
                                    let plugin_id = plugin.id.clone();
                                    let plugin_name = plugin.name.clone();
                                    let plugin_path = plugin.path.clone();
                                    let on_open = on_plugin_open.clone();

                                    div()
                                        .w_full()
                                        .p_4()
                                        .rounded(px(8.0))
                                        .bg(Colors::bg_secondary())
                                        .child(
                                            h_flex()
                                                .w_full()
                                                .justify_between()
                                                .items_start()
                                                .child(
                                                    v_flex()
                                                        .flex_1()
                                                        .gap_1()
                                                        .child(
                                                            h_flex()
                                                                .gap_2()
                                                                .items_center()
                                                                .child(
                                                                    div()
                                                                        .text_size(px(14.0))
                                                                        .font_weight(FontWeight::MEDIUM)
                                                                        .text_color(Colors::text_primary())
                                                                        .child(plugin.name.clone()),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .px_2()
                                                                        .py(px(2.0))
                                                                        .rounded(px(4.0))
                                                                        .bg(Colors::accent().opacity(0.1))
                                                                        .text_size(px(10.0))
                                                                        .text_color(Colors::accent())
                                                                        .child(format!("v{}", plugin.version)),
                                                                ),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_size(px(12.0))
                                                                .text_color(Colors::text_muted())
                                                                .child(plugin.desc.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_size(px(11.0))
                                                                .text_color(Colors::text_muted())
                                                                .child(format!("作者: {}", plugin.author)),
                                                        ),
                                                )
                                                .child(
                                                    h_flex()
                                                        .gap_3()
                                                        .items_center()
                                                        .child(
                                                            div()
                                                                .id(SharedString::from(format!("plugin-open-{}", plugin_id.clone())))
                                                                .px_3()
                                                                .py(px(6.0))
                                                                .rounded(px(4.0))
                                                                .cursor_pointer()
                                                                .bg(Colors::accent())
                                                                .text_size(px(12.0))
                                                                .text_color(Colors::bg_primary())
                                                                .hover(|s| s.bg(Colors::accent().opacity(0.8)))
                                                                .child("打开")
                                                                .on_click({
                                                                    let plugin_id = plugin_id.clone();
                                                                    let plugin_name = plugin_name.clone();
                                                                    let plugin_path = plugin_path.clone();
                                                                    move |_event, window, cx| {
                                                                        if let Some(ref callback) = on_open {
                                                                            callback(plugin_id.clone(), plugin_name.clone(), plugin_path.clone(), window, cx);
                                                                        }
                                                                    }
                                                                }),
                                                        ),
                                                ),
                                        )
                                })),
                        ),
                ),
            )
            .child(
                self.render_section_card(
                    v_flex()
                        .w_full()
                        .child(self.render_section_title("插件说明"))
                        .child(
                            v_flex()
                                .w_full()
                                .py_2()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(Colors::text_secondary())
                                        .child("插件是基于 HTML/JavaScript 的扩展，可以订阅弹幕、礼物等事件。"),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(Colors::text_secondary())
                                        .child("每个插件需要包含 meta.json 配置文件和入口 HTML 文件。"),
                                )
                                .child(
                                    div()
                                        .pt_2()
                                        .child(
                                            div()
                                                .id("open-plugins-folder-btn")
                                                .px_3()
                                                .py(px(6.0))
                                                .rounded(px(4.0))
                                                .cursor_pointer()
                                                .bg(Colors::bg_hover())
                                                .text_size(px(12.0))
                                                .text_color(Colors::text_primary())
                                                .hover(|s| s.bg(Colors::accent().opacity(0.2)))
                                                .child("打开插件目录")
                                                .on_click({
                                                    let callback = on_open_plugins_folder.clone();
                                                    move |_event, window, cx| {
                                                        if let Some(ref cb) = callback {
                                                            cb(window, cx);
                                                        }
                                                    }
                                                }),
                                        ),
                                ),
                        ),
                ),
            )
    }

    fn render_advanced_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let log_levels = vec![("info", "Info"), ("debug", "Debug")];

        v_flex()
            .w_full()
            .p_6()
            .gap_4()
            .child(
                self.render_section_card(
                    v_flex()
                        .w_full()
                        .child(self.render_section_title("弹幕数量限制"))
                        .child(
                            v_flex()
                                .w_full()
                                .py_2()
                                .gap_3()
                                .child(
                                    h_flex()
                                        .w_full()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            v_flex()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .text_size(px(13.0))
                                                        .text_color(Colors::text_primary())
                                                        .child("主窗口最大弹幕数"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(11.0))
                                                        .text_color(Colors::text_muted())
                                                        .child("范围: 50 - 5000"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(13.0))
                                                .text_color(Colors::text_secondary())
                                                .child("200"),
                                        ),
                                )
                                .child(
                                    h_flex()
                                        .w_full()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            v_flex()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .text_size(px(13.0))
                                                        .text_color(Colors::text_primary())
                                                        .child("详情窗口最大弹幕数"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(11.0))
                                                        .text_color(Colors::text_muted())
                                                        .child("范围: 50 - 5000"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(13.0))
                                                .text_color(Colors::text_secondary())
                                                .child("100"),
                                        ),
                                ),
                        ),
                ),
            )
            .child(
                self.render_section_card(
                    v_flex()
                        .w_full()
                        .child(self.render_section_title("日志设置"))
                        .child(
                            v_flex()
                                .w_full()
                                .py_2()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .text_color(Colors::text_primary())
                                        .child("日志等级"),
                                )
                                .child(
                                    h_flex()
                                        .w_full()
                                        .gap_2()
                                        .children(log_levels.into_iter().map(|(id, name)| {
                                            let is_selected = id == "info";
                                            div()
                                                .id(SharedString::from(format!("log-level-{}", id)))
                                                .px_3()
                                                .py_2()
                                                .rounded(px(6.0))
                                                .cursor_pointer()
                                                .border_1()
                                                .when(is_selected, |this| {
                                                    this.border_color(Colors::accent())
                                                        .bg(Colors::accent().opacity(0.1))
                                                })
                                                .when(!is_selected, |this| {
                                                    this.border_color(Colors::bg_hover())
                                                        .hover(|s| s.bg(Colors::bg_hover()))
                                                })
                                                .text_size(px(12.0))
                                                .text_color(Colors::text_primary())
                                                .child(name)
                                                .on_click(cx.listener(move |_this, _event, _window, cx| {
                                                    tracing::info!("Log level selected: {}", id);
                                                    cx.notify();
                                                }))
                                        })),
                                ),
                        ),
                ),
            )
    }

    fn render_about_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .p_6()
            .gap_4()
            // App info section
            .child(
                self.render_section_card(
                    v_flex()
                        .w_full()
                        .child(self.render_section_title("关于"))
                        .child(
                            v_flex()
                                .w_full()
                                .py_2()
                                .gap_4()
                                // App logo and name
                                .child(
                                    h_flex()
                                        .w_full()
                                        .gap_4()
                                        .items_center()
                                        .child(
                                            div()
                                                .size(px(64.0))
                                                .rounded(px(12.0))
                                                .bg(Colors::accent())
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_size(px(28.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(Colors::text_primary())
                                                .child("J"),
                                        )
                                        .child(
                                            v_flex()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .text_size(px(18.0))
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(Colors::text_primary())
                                                        .child("JLiverTool"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(12.0))
                                                        .text_color(Colors::text_muted())
                                                        .child("Bilibili 直播弹幕工具"),
                                                ),
                                        ),
                                )
                                // Version info
                                .child(
                                    v_flex()
                                        .w_full()
                                        .gap_2()
                                        .child(
                                            h_flex()
                                                .w_full()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_size(px(13.0))
                                                        .text_color(Colors::text_secondary())
                                                        .child("版本"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(13.0))
                                                        .text_color(Colors::text_primary())
                                                        .child(env!("CARGO_PKG_VERSION")),
                                                ),
                                        )
                                        .child(
                                            h_flex()
                                                .w_full()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_size(px(13.0))
                                                        .text_color(Colors::text_secondary())
                                                        .child("框架"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(13.0))
                                                        .text_color(Colors::text_primary())
                                                        .child("GPUI (Rust)"),
                                                ),
                                        ),
                                ),
                        ),
                ),
            )
            // Links section
            .child(
                self.render_section_card(
                    v_flex()
                        .w_full()
                        .child(self.render_section_title("链接"))
                        .child(
                            v_flex()
                                .w_full()
                                .py_2()
                                .gap_2()
                                .child(
                                    div()
                                        .id("github-link")
                                        .w_full()
                                        .px_3()
                                        .py_2()
                                        .rounded(px(6.0))
                                        .cursor_pointer()
                                        .bg(Colors::bg_secondary())
                                        .hover(|s| s.bg(Colors::bg_hover()))
                                        .on_click(cx.listener(|_this, _event, _window, cx| {
                                            cx.open_url("https://github.com/Xinrea/JLiverTool");
                                        }))
                                        .child(
                                            h_flex()
                                                .w_full()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_size(px(13.0))
                                                        .text_color(Colors::text_primary())
                                                        .child("GitHub 仓库"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(11.0))
                                                        .text_color(Colors::text_muted())
                                                        .child("查看源代码"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("issues-link")
                                        .w_full()
                                        .px_3()
                                        .py_2()
                                        .rounded(px(6.0))
                                        .cursor_pointer()
                                        .bg(Colors::bg_secondary())
                                        .hover(|s| s.bg(Colors::bg_hover()))
                                        .on_click(cx.listener(|_this, _event, _window, cx| {
                                            cx.open_url("https://github.com/Xinrea/JLiverTool/issues");
                                        }))
                                        .child(
                                            h_flex()
                                                .w_full()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_size(px(13.0))
                                                        .text_color(Colors::text_primary())
                                                        .child("问题反馈"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(11.0))
                                                        .text_color(Colors::text_muted())
                                                        .child("报告问题或建议"),
                                                ),
                                        ),
                                ),
                        ),
                ),
            )
            // Update section
            .child(
                self.render_section_card(
                    v_flex()
                        .w_full()
                        .child(self.render_section_title("更新"))
                        .child(
                            v_flex()
                                .w_full()
                                .py_2()
                                .gap_3()
                                .child(self.render_setting_row(
                                    "自动检查更新",
                                    "启动时检查新版本",
                                    Switch::new("auto_update").checked(true).on_click({
                                        let entity = cx.entity().clone();
                                        move |_checked: &bool, _window, cx| {
                                            entity.update(cx, |_, cx| cx.notify());
                                        }
                                    }),
                                ))
                                .child(
                                    div()
                                        .id("check-update-btn")
                                        .w_full()
                                        .px_3()
                                        .py_2()
                                        .rounded(px(6.0))
                                        .cursor_pointer()
                                        .bg(Colors::accent())
                                        .hover(|s| s.opacity(0.8))
                                        .text_size(px(13.0))
                                        .text_color(Colors::button_text())
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child("检查更新")
                                        .on_click(cx.listener(|_this, _event, _window, cx| {
                                            tracing::info!("Check update clicked");
                                            cx.notify();
                                        })),
                                ),
                        ),
                ),
            )
    }
}

impl Render for SettingView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // macOS: left padding for traffic light buttons (buttons are ~70px wide)
        #[cfg(target_os = "macos")]
        let left_padding = px(78.0);
        #[cfg(not(target_os = "macos"))]
        let left_padding = px(24.0);

        let active_tab = self.active_tab;

        v_flex()
            .size_full()
            .bg(Colors::bg_primary())
            .text_color(Colors::text_primary())
            .child(
                // Header - matches other window headers style
                h_flex()
                    .w_full()
                    .h(px(32.0))
                    .pl(left_padding)
                    .pr_2()
                    .items_center()
                    .bg(Colors::bg_secondary())
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(Colors::text_primary())
                            .child("设置"),
                    ),
            )
            .child(
                // Content area with tabs
                h_flex()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(
                        // Tab sidebar
                        v_flex()
                            .w(px(160.0))
                            .h_full()
                            .bg(Colors::bg_secondary())
                            .border_r_1()
                            .border_color(Colors::bg_hover())
                            .p_3()
                            .gap_1()
                            .children(
                                SettingsTab::all().into_iter().enumerate().map(|(idx, tab)| {
                                    let is_active = idx == active_tab;
                                    div()
                                        .id(SharedString::from(format!("tab-{}", idx)))
                                        .w_full()
                                        .px_3()
                                        .py_2()
                                        .rounded(px(6.0))
                                        .cursor_pointer()
                                        .when(is_active, |this| {
                                            this.bg(Colors::accent())
                                                .text_color(Colors::button_text())
                                        })
                                        .when(!is_active, |this| {
                                            this.text_color(Colors::text_secondary())
                                                .hover(|s| s.bg(Colors::bg_hover()))
                                        })
                                        .text_size(px(13.0))
                                        .child(tab.name())
                                        .on_click(cx.listener(move |this, _event, _window, cx| {
                                            this.active_tab = idx;
                                            cx.notify();
                                        }))
                                }),
                            ),
                    )
                    .child(
                        // Tab content
                        div()
                            .flex_1()
                            .h_full()
                            .overflow_y_hidden()
                            .child(
                                div()
                                    .id("tab-content")
                                    .size_full()
                                    .overflow_y_scroll()
                                    .child(self.render_tab_content(active_tab, window, cx)),
                            ),
                    ),
            )
    }
}
