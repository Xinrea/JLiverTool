//! System tray icon support
//!
//! This module provides cross-platform system tray functionality using the `tray-icon` crate.

mod icon;
mod menu;

pub use icon::load_icon;
pub use menu::{build_menu, MenuIds};

use muda::{Menu, MenuEvent};
use std::sync::mpsc;
use tray_icon::{TrayIcon, TrayIconBuilder};
use tracing::{error, info};

/// Commands that can be triggered from the tray menu
#[derive(Debug, Clone)]
pub enum TrayCommand {
    /// Toggle main window visibility
    ToggleWindow,
    /// Open settings window
    OpenSettings,
    /// Start live streaming (room_id, area_id)
    StartLive(u64, u64),
    /// Stop live streaming (room_id)
    StopLive(u64),
    /// Toggle click-through mode
    ToggleClickThrough,
    /// Quit the application
    Quit,
}

/// Current state of the tray icon
#[derive(Debug, Clone, Default)]
pub struct TrayState {
    pub room_id: Option<u64>,
    pub room_title: String,
    pub area_id: u64,
    pub live_status: u8,
    pub logged_in: bool,
    pub is_room_owner: bool,
    pub connected: bool,
    pub window_visible: bool,
    pub click_through: bool,
}

/// Manages the system tray icon and menu
pub struct TrayManager {
    tray_icon: TrayIcon,
    #[allow(dead_code)]
    menu: Menu,
    menu_ids: MenuIds,
    state: TrayState,
    command_tx: mpsc::Sender<TrayCommand>,
}

impl TrayManager {
    /// Create a new TrayManager
    ///
    /// This must be called on the main thread before the GPUI event loop starts.
    pub fn new() -> Result<(Self, mpsc::Receiver<TrayCommand>), Box<dyn std::error::Error>> {
        let (command_tx, command_rx) = mpsc::channel();

        // Load the default icon
        let icon = load_icon()?;

        // Build the menu
        let (menu, menu_ids) = build_menu(&TrayState::default())?;

        // Create the tray icon
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("JLiverTool")
            .with_icon(icon)
            .build()?;

        info!("Tray icon created successfully");

        Ok((
            Self {
                tray_icon,
                menu,
                menu_ids,
                state: TrayState::default(),
                command_tx,
            },
            command_rx,
        ))
    }

    /// Update the tray state and refresh the menu
    pub fn update_state(&mut self, state: TrayState) {
        // Preserve click_through state since it's managed by TrayManager
        let click_through = self.state.click_through;
        self.state = state.clone();
        self.state.click_through = click_through;

        // Update menu items based on state
        self.update_menu();

        // Update tooltip
        let tooltip = if let Some(room_id) = state.room_id {
            let status = if state.live_status == 1 { "直播中" } else { "未开播" };
            format!("JLiverTool - {} ({})", room_id, status)
        } else {
            "JLiverTool".to_string()
        };

        if let Err(e) = self.tray_icon.set_tooltip(Some(&tooltip)) {
            error!("Failed to set tray tooltip: {}", e);
        }
    }

    /// Update the menu based on current state
    fn update_menu(&mut self) {
        // Update Show/Hide Window text
        let show_hide_text = if self.state.window_visible {
            "隐藏窗口"
        } else {
            "显示窗口"
        };
        self.menu_ids.show_hide.set_text(show_hide_text);

        // Update click-through text
        let click_through_text = if self.state.click_through {
            "关闭鼠标穿透"
        } else {
            "开启鼠标穿透"
        };
        self.menu_ids.click_through.set_text(click_through_text);

        // Update room info
        let room_info = if let Some(room_id) = self.state.room_id {
            if self.state.room_title.is_empty() {
                format!("房间: {}", room_id)
            } else {
                format!("房间: {} - {}", room_id, self.state.room_title)
            }
        } else {
            "房间: 未连接".to_string()
        };
        self.menu_ids.room_info.set_text(&room_info);

        // Update live status
        let live_status = if self.state.live_status == 1 {
            "状态: 直播中"
        } else {
            "状态: 未开播"
        };
        self.menu_ids.live_status.set_text(live_status);

        // Update start/stop live visibility (only for room owner)
        if self.state.logged_in && self.state.is_room_owner {
            if self.state.live_status == 1 {
                self.menu_ids.start_live.set_enabled(false);
                self.menu_ids.stop_live.set_enabled(true);
            } else {
                self.menu_ids.start_live.set_enabled(true);
                self.menu_ids.stop_live.set_enabled(false);
            }
        } else {
            self.menu_ids.start_live.set_enabled(false);
            self.menu_ids.stop_live.set_enabled(false);
        }
    }

    /// Update the tray icon based on live status
    pub fn update_icon(&mut self, is_live: bool) {
        match icon::load_icon_for_status(is_live) {
            Ok(icon) => {
                if let Err(e) = self.tray_icon.set_icon(Some(icon)) {
                    error!("Failed to set tray icon: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to load tray icon: {}", e);
            }
        }
    }

    /// Process menu events and send commands
    ///
    /// This should be called periodically to handle menu clicks.
    pub fn process_events(&self) {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            let command = if event.id == self.menu_ids.show_hide.id() {
                Some(TrayCommand::ToggleWindow)
            } else if event.id == self.menu_ids.settings.id() {
                Some(TrayCommand::OpenSettings)
            } else if event.id == self.menu_ids.click_through.id() {
                Some(TrayCommand::ToggleClickThrough)
            } else if event.id == self.menu_ids.start_live.id() {
                self.state.room_id.map(|room_id| {
                    TrayCommand::StartLive(room_id, self.state.area_id)
                })
            } else if event.id == self.menu_ids.stop_live.id() {
                self.state.room_id.map(|room_id| {
                    TrayCommand::StopLive(room_id)
                })
            } else if event.id == self.menu_ids.quit.id() {
                Some(TrayCommand::Quit)
            } else {
                None
            };

            if let Some(cmd) = command {
                if let Err(e) = self.command_tx.send(cmd) {
                    error!("Failed to send tray command: {}", e);
                }
            }
        }
    }

    /// Get a clone of the command sender
    pub fn command_sender(&self) -> mpsc::Sender<TrayCommand> {
        self.command_tx.clone()
    }

    /// Toggle click-through state and update menu
    pub fn toggle_click_through(&mut self) -> bool {
        self.state.click_through = !self.state.click_through;
        self.update_menu();
        self.state.click_through
    }

    /// Get current click-through state
    pub fn click_through_enabled(&self) -> bool {
        self.state.click_through
    }
}
