//! Event processing for MainView

use super::{DisplayMessage, MainView, MAX_DANMU_COUNT};
use gpui::Context;
use jlivertool_core::events::Event;

impl MainView {
    pub(super) fn process_events(&mut self, cx: &mut Context<Self>) {
        let mut list_modified = false;
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                Event::UpdateRoom {
                    room_id,
                    title,
                    live_status,
                    area_id,
                } => {
                    let real_id = room_id.real_id();
                    let owner_uid = room_id.owner_uid();
                    let title_clone = title.clone();
                    self.setting_view.update(cx, |view, cx| {
                        view.set_room_id(Some(real_id), Some(owner_uid), cx);
                        view.set_room_title(title_clone, cx);
                        view.set_area_id(area_id, cx);
                    });

                    let is_new_room = self.room.as_ref().map(|r| r.real_id()) != Some(real_id);

                    if let Some(db) = &self.database {
                        self.statistics_view.update(cx, |view, cx| {
                            view.set_database(db.clone());
                            view.set_room_id(Some(real_id), cx);
                        });

                        if is_new_room {
                            let db_clone = db.clone();
                            self.gift_view.update(cx, |view, cx| {
                                view.load_from_database(&db_clone, real_id, cx);
                            });
                            let db_clone = db.clone();
                            self.superchat_view.update(cx, |view, cx| {
                                view.load_from_database(&db_clone, real_id, cx);
                            });

                            if let Ok(recent_danmus) = db.get_danmus_since(real_id, 30) {
                                for danmu in recent_danmus {
                                    self.danmu_list.push_back(DisplayMessage::Danmu(danmu));
                                }
                                while self.danmu_list.len() > MAX_DANMU_COUNT {
                                    self.danmu_list.pop_front();
                                }
                                list_modified = true;
                                self.scroll_to_bottom();
                            }
                        }
                    }

                    self.room = Some(room_id);
                    self.room_title = title;
                    self.area_id = area_id;

                    if is_new_room {
                        self.live_status = live_status;
                    }

                    // Update tray state
                    self.update_tray_state();
                }
                Event::UpdateOnline { count } => {
                    self.online_count = count;
                }
                Event::NewDanmu(danmu) => {
                    if !danmu.is_generated {
                        let should_auto_scroll = self.is_at_bottom();
                        self.danmu_list.push_back(DisplayMessage::Danmu(danmu));
                        while self.danmu_list.len() > MAX_DANMU_COUNT {
                            self.danmu_list.pop_front();
                        }
                        list_modified = true;
                        if should_auto_scroll {
                            self.scroll_to_bottom();
                        }
                    }
                }
                Event::NewInteract(interact) => {
                    if self.interact_display {
                        let should_auto_scroll = self.is_at_bottom();
                        self.danmu_list
                            .push_back(DisplayMessage::Interact(interact));
                        while self.danmu_list.len() > MAX_DANMU_COUNT {
                            self.danmu_list.pop_front();
                        }
                        list_modified = true;
                        if should_auto_scroll {
                            self.scroll_to_bottom();
                        }
                    }
                }
                Event::NewEntryEffect(entry) => {
                    let is_guard_entry = entry.privilege_type >= 1 && entry.privilege_type <= 3;
                    let should_display = if is_guard_entry {
                        self.guard_effect
                    } else {
                        self.level_effect
                    };

                    if should_display {
                        let should_auto_scroll = self.is_at_bottom();
                        self.danmu_list
                            .push_back(DisplayMessage::EntryEffect(entry));
                        while self.danmu_list.len() > MAX_DANMU_COUNT {
                            self.danmu_list.pop_front();
                        }
                        list_modified = true;
                        if should_auto_scroll {
                            self.scroll_to_bottom();
                        }
                    }
                }
                Event::NewGift(gift) => {
                    let should_auto_scroll = self.is_at_bottom();
                    self.danmu_list
                        .push_back(DisplayMessage::Gift(gift.clone()));
                    while self.danmu_list.len() > MAX_DANMU_COUNT {
                        self.danmu_list.pop_front();
                    }
                    list_modified = true;
                    if should_auto_scroll {
                        self.scroll_to_bottom();
                    }
                    self.gift_view.update(cx, |view, cx| {
                        view.add_gift(gift, cx);
                    });
                }
                Event::NewGuard(guard) => {
                    let should_auto_scroll = self.is_at_bottom();
                    self.danmu_list
                        .push_back(DisplayMessage::Guard(guard.clone()));
                    while self.danmu_list.len() > MAX_DANMU_COUNT {
                        self.danmu_list.pop_front();
                    }
                    list_modified = true;
                    if should_auto_scroll {
                        self.scroll_to_bottom();
                    }
                    self.gift_view.update(cx, |view, cx| {
                        view.add_guard(guard, cx);
                    });
                }
                Event::NewSuperChat(sc) => {
                    let should_auto_scroll = self.is_at_bottom();
                    self.danmu_list
                        .push_back(DisplayMessage::SuperChat(sc.clone()));
                    while self.danmu_list.len() > MAX_DANMU_COUNT {
                        self.danmu_list.pop_front();
                    }
                    list_modified = true;
                    if should_auto_scroll {
                        self.scroll_to_bottom();
                    }
                    self.superchat_view.update(cx, |view, cx| {
                        view.add_superchat(sc, cx);
                    });
                }
                Event::ConnectionStatus { connected } => {
                    self.connected = connected;
                    self.update_tray_state();
                }
                Event::LiveStart => {
                    self.live_status = 1;
                    self.update_tray_state();
                }
                Event::LiveEnd => {
                    self.live_status = 0;
                    self.setting_view.update(cx, |view, cx| {
                        view.clear_rtmp_info(cx);
                    });
                    self.update_tray_state();
                }
                Event::LoginStatusChanged {
                    logged_in,
                    user_info,
                } => {
                    self.logged_in = logged_in;
                    // Store the logged-in user's UID
                    self.logged_in_uid = if logged_in {
                        user_info.as_ref().map(|u| u.mid)
                    } else {
                        None
                    };
                    self.setting_view.update(cx, |view, cx| {
                        view.set_login_status(logged_in, user_info, cx);
                    });
                    self.update_tray_state();
                }
                Event::QrCodeGenerated { url, qrcode_key: _ } => {
                    self.setting_view.update(cx, |view, cx| {
                        view.set_qr_code(url, cx);
                    });
                }
                Event::QrLoginStatus { status } => {
                    self.setting_view.update(cx, |view, cx| {
                        view.set_qr_status(status, cx);
                    });
                }
                Event::ConfigLoaded {
                    always_on_top,
                    guard_effect,
                    level_effect,
                    opacity,
                    lite_mode,
                    medal_display,
                    interact_display,
                    theme,
                    font_size,
                    tts_enabled,
                    tts_gift_enabled,
                    tts_sc_enabled,
                    tts_volume,
                    max_danmu_count,
                    log_level,
                    auto_update_check,
                } => {
                    crate::theme::set_theme(&theme);

                    self.setting_view.update(cx, |view, cx| {
                        view.load_config(
                            crate::views::ConfigValues {
                                always_on_top,
                                guard_effect,
                                level_effect,
                                opacity,
                                lite_mode,
                                medal_display,
                                interact_display,
                                theme,
                                font_size,
                                tts_enabled,
                                tts_gift_enabled,
                                tts_sc_enabled,
                                tts_volume,
                            },
                            cx,
                        );
                        // Set advanced settings
                        view.set_advanced_settings(max_danmu_count, log_level, cx);
                        // Set auto update check setting
                        view.set_auto_update_check(auto_update_check, cx);
                    });
                    self.opacity = opacity;
                    self.font_size = font_size;
                    self.lite_mode = lite_mode;
                    self.medal_display = medal_display;
                    self.interact_display = interact_display;
                    self.guard_effect = guard_effect;
                    self.level_effect = level_effect;
                    self.always_on_top = always_on_top;
                    self.gift_view
                        .update(cx, |v, cx| v.set_opacity(opacity, cx));
                    self.superchat_view
                        .update(cx, |v, cx| v.set_opacity(opacity, cx));
                    self.statistics_view
                        .update(cx, |v, cx| v.set_opacity(opacity, cx));
                    self.audience_view
                        .update(cx, |v, cx| v.set_opacity(opacity, cx));
                    if always_on_top {
                        self.pending_always_on_top = Some(always_on_top);
                    }
                }
                Event::RtmpInfo { addr, code } => {
                    self.setting_view.update(cx, |view, cx| {
                        view.set_rtmp_info(addr, code, cx);
                    });
                }
                Event::FaceAuthRequired { qr_url } => {
                    self.setting_view.update(cx, |view, cx| {
                        view.show_face_auth_dialog(qr_url, cx);
                    });
                }
                Event::ClearDanmuList => {
                    self.danmu_list.clear();
                    list_modified = true;
                }
                Event::UserInfoFetched { uid, user_info } => {
                    let mut selected = self.selected_user.borrow_mut();
                    if let Some(ref mut user) = *selected {
                        if user.sender.uid == uid {
                            user.fetched_info = Some(user_info);
                        }
                    }
                }
                Event::AudienceListFetched { list } => {
                    self.audience_view.update(cx, |view, cx| {
                        view.set_audience_list(list, cx);
                    });
                }
                Event::GuardListFetched { list, total, page } => {
                    self.audience_view.update(cx, |view, cx| {
                        view.set_guard_list(list, total, page, cx);
                    });
                }
                Event::PluginsRefreshed { plugins } => {
                    let ui_plugins: Vec<crate::views::setting_view::PluginInfo> = plugins
                        .into_iter()
                        .map(|p| crate::views::setting_view::PluginInfo {
                            id: p.id,
                            name: p.name,
                            author: p.author,
                            desc: p.desc,
                            version: p.version,
                            enabled: true,
                            path: p.path,
                        })
                        .collect();
                    self.set_plugins(ui_plugins, cx);
                }
                Event::PluginImportResult { success, message } => {
                    self.set_plugin_import_status(Some(message), cx);
                    // Clear status after 5 seconds
                    if success {
                        // Status will be cleared when user starts typing a new URL
                    }
                }
                Event::DataCleared => {
                    // Clear all UI lists when data is cleared
                    self.danmu_list.clear();
                    list_modified = true;
                    // Gift and SC views will be reloaded from database (which is now empty)
                    tracing::info!("Data cleared, UI lists reset");
                }
                Event::RoomChange(room_change) => {
                    // Update room title when it changes via WebSocket
                    if !room_change.title.is_empty() {
                        self.room_title = room_change.title.clone();
                        self.setting_view.update(cx, |view, cx| {
                            view.set_room_title(room_change.title, cx);
                        });
                    }
                }
                Event::UpdateCheckResult {
                    has_update,
                    latest_version,
                    release_url,
                    error,
                    ..
                } => {
                    use crate::views::setting_view::UpdateStatus;
                    let status = if let Some(err) = error {
                        UpdateStatus::Error(err)
                    } else if has_update {
                        // Show update dialog popup
                        self.show_update_dialog = true;
                        self.update_info = Some(super::UpdateDialogInfo {
                            latest_version: latest_version.clone(),
                            release_url: release_url.clone(),
                        });
                        UpdateStatus::UpdateAvailable {
                            version: latest_version,
                            url: release_url,
                        }
                    } else {
                        UpdateStatus::UpToDate
                    };
                    self.setting_view.update(cx, |view, cx| {
                        view.set_update_status(status, cx);
                    });
                }
                _ => {}
            }
        }
        if list_modified {
            self.update_snapshot();
        }
    }
}
