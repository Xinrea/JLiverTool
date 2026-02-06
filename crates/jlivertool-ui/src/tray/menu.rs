//! Tray menu construction

use super::TrayState;
use muda::{Menu, MenuItem, PredefinedMenuItem};

/// Menu item IDs for event handling
pub struct MenuIds {
    pub show_hide: MenuItem,
    pub room_info: MenuItem,
    pub live_status: MenuItem,
    pub start_live: MenuItem,
    pub stop_live: MenuItem,
    pub settings: MenuItem,
    pub quit: MenuItem,
}

/// Build the tray context menu
pub fn build_menu(state: &TrayState) -> Result<(Menu, MenuIds), Box<dyn std::error::Error>> {
    let menu = Menu::new();

    // Show/Hide Window
    let show_hide_text = if state.window_visible {
        "隐藏窗口"
    } else {
        "显示窗口"
    };
    let show_hide = MenuItem::new(show_hide_text, true, None);
    menu.append(&show_hide)?;

    menu.append(&PredefinedMenuItem::separator())?;

    // Room Info (disabled label)
    let room_info_text = if let Some(room_id) = state.room_id {
        if state.room_title.is_empty() {
            format!("房间: {}", room_id)
        } else {
            format!("房间: {} - {}", room_id, state.room_title)
        }
    } else {
        "房间: 未连接".to_string()
    };
    let room_info = MenuItem::new(&room_info_text, false, None);
    menu.append(&room_info)?;

    // Live Status (disabled label)
    let live_status_text = if state.live_status == 1 {
        "状态: 直播中"
    } else {
        "状态: 未开播"
    };
    let live_status = MenuItem::new(live_status_text, false, None);
    menu.append(&live_status)?;

    menu.append(&PredefinedMenuItem::separator())?;

    // Start Live (only enabled if logged in and not live)
    let start_live_enabled = state.logged_in && state.live_status != 1;
    let start_live = MenuItem::new("开始直播", start_live_enabled, None);
    menu.append(&start_live)?;

    // Stop Live (only enabled if logged in and live)
    let stop_live_enabled = state.logged_in && state.live_status == 1;
    let stop_live = MenuItem::new("停止直播", stop_live_enabled, None);
    menu.append(&stop_live)?;

    menu.append(&PredefinedMenuItem::separator())?;

    // Settings
    let settings = MenuItem::new("设置", true, None);
    menu.append(&settings)?;

    menu.append(&PredefinedMenuItem::separator())?;

    // Quit
    let quit = MenuItem::new("退出", true, None);
    menu.append(&quit)?;

    let menu_ids = MenuIds {
        show_hide,
        room_info,
        live_status,
        start_live,
        stop_live,
        settings,
        quit,
    };

    Ok((menu, menu_ids))
}
