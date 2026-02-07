//! User info card component
//!
//! This module contains the user info card popup that displays detailed
//! information about a user when clicked in the danmu list.

use super::content_rendering::guard_icon_url;
use crate::theme::Colors;
use chrono::{Local, TimeZone};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::h_flex;
use gpui_component::v_flex;
use jlivertool_core::bilibili::api::UserInfoData;
use jlivertool_core::types::Sender;
use std::cell::RefCell;
use std::rc::Rc;

/// Selected user data with optional fetched info
#[derive(Clone)]
pub struct SelectedUser {
    pub sender: Sender,
    pub fetched_info: Option<UserInfoData>,
    pub fetch_requested: bool,
}

/// Shared state for user info popup
pub type SelectedUserState = Rc<RefCell<Option<SelectedUser>>>;

/// User info card for tooltip
pub struct UserInfoCard;

impl UserInfoCard {
    /// Create the user info card element directly (for use in Tooltip::element)
    pub fn render_element(
        selected: &SelectedUser,
        danmu_history: Vec<(String, i64)>,
    ) -> impl IntoElement {
        let sender = &selected.sender;
        let uid = sender.uid;
        let uname = sender.uname.clone();
        let face = sender.face.clone();
        let medal = sender.medal_info.clone();
        let fetched = selected.fetched_info.clone();

        // Use fetched face if available (higher quality)
        let display_face = fetched
            .as_ref()
            .map(|f| f.face.clone())
            .filter(|f| !f.is_empty())
            .unwrap_or(face);

        // URLs for opening in browser
        let space_url = format!("https://space.bilibili.com/{}", uid);
        let space_url_clone = space_url.clone();

        v_flex()
            .w(px(300.0))
            .p_3()
            .gap_2()
            .bg(Colors::bg_secondary())
            .rounded_lg()
            .border_2()
            .border_color(Colors::accent().opacity(0.5))
            .shadow_lg()
            // User avatar and name row
            .child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .items_center()
                    // Avatar (clickable to open space)
                    .child(
                        div()
                            .id("user-avatar")
                            .size(px(56.0))
                            .rounded(px(8.0))
                            .bg(Colors::bg_hover())
                            .cursor_pointer()
                            .hover(|s| s.opacity(0.8))
                            .when(!display_face.is_empty(), |this| {
                                let face_url = display_face.clone();
                                this.child(
                                    img(face_url)
                                        .size(px(56.0))
                                        .rounded(px(8.0))
                                        .object_fit(ObjectFit::Cover),
                                )
                            })
                            .on_click({
                                let url = space_url.clone();
                                move |_, _, cx| {
                                    cx.open_url(&url);
                                }
                            }),
                    )
                    // Name, UID, Level, and badges
                    .child(
                        v_flex()
                            .flex_1()
                            .gap(px(3.0))
                            .overflow_hidden()
                            // Name row with VIP badge
                            .child(
                                h_flex()
                                    .gap_1()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(15.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(Colors::text_primary())
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .child(uname),
                                    )
                                    // VIP badge
                                    .when_some(
                                        fetched.as_ref().and_then(|f| {
                                            if f.vip.status == 1 && !f.vip.label.text.is_empty() {
                                                Some(f.vip.label.text.clone())
                                            } else {
                                                None
                                            }
                                        }),
                                        |this, vip_text| {
                                            this.child(
                                                div()
                                                    .px(px(4.0))
                                                    .py(px(1.0))
                                                    .rounded(px(2.0))
                                                    .bg(hsla(350.0 / 360.0, 0.8, 0.55, 1.0))
                                                    .text_size(px(9.0))
                                                    .text_color(gpui::white())
                                                    .child(vip_text),
                                            )
                                        },
                                    ),
                            )
                            // UID and Level row
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(Colors::text_muted())
                                            .child(format!("UID: {}", uid)),
                                    )
                                    .when_some(fetched.as_ref().map(|f| f.level), |this, level| {
                                        let level_color = match level {
                                            0..=1 => hsla(0.0, 0.0, 0.6, 1.0),
                                            2..=3 => hsla(120.0 / 360.0, 0.5, 0.45, 1.0),
                                            4..=5 => hsla(200.0 / 360.0, 0.7, 0.5, 1.0),
                                            _ => hsla(30.0 / 360.0, 0.9, 0.5, 1.0),
                                        };
                                        this.child(
                                            div()
                                                .px(px(5.0))
                                                .py(px(1.0))
                                                .rounded(px(3.0))
                                                .bg(level_color)
                                                .text_size(px(10.0))
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(gpui::white())
                                                .child(format!("Lv.{}", level)),
                                        )
                                    }),
                            )
                            // Sex and birthday row
                            .when_some(
                                fetched.as_ref().and_then(|f| {
                                    if f.sex == "男" || f.sex == "女" {
                                        Some(f.sex.clone())
                                    } else {
                                        None
                                    }
                                }),
                                |this, sex| {
                                    let sex_color = if sex == "男" {
                                        hsla(210.0 / 360.0, 0.7, 0.55, 1.0)
                                    } else {
                                        hsla(340.0 / 360.0, 0.7, 0.6, 1.0)
                                    };
                                    let sex_icon = if sex == "男" { "♂" } else { "♀" };
                                    this.child(
                                        h_flex().gap_1().items_center().child(
                                            div()
                                                .text_size(px(12.0))
                                                .text_color(sex_color)
                                                .child(sex_icon),
                                        ),
                                    )
                                },
                            ),
                    ),
            )
            // Official verification (if available)
            .when_some(
                fetched.as_ref().and_then(|f| {
                    if f.official.official_type >= 0 && !f.official.title.is_empty() {
                        Some(f.official.clone())
                    } else {
                        None
                    }
                }),
                |this, official| {
                    let (icon, bg_color) = if official.official_type == 0 {
                        ("⚡", hsla(30.0 / 360.0, 0.9, 0.5, 1.0)) // Personal - orange
                    } else {
                        ("✓", hsla(200.0 / 360.0, 0.8, 0.5, 1.0)) // Organization - blue
                    };
                    this.child(
                        h_flex()
                            .w_full()
                            .gap_2()
                            .items_center()
                            .px_2()
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .bg(bg_color.opacity(0.15))
                            .child(
                                div()
                                    .size(px(16.0))
                                    .rounded_full()
                                    .bg(bg_color)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_size(px(10.0))
                                    .text_color(gpui::white())
                                    .child(icon),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(px(11.0))
                                    .text_color(Colors::text_secondary())
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(official.title),
                            ),
                    )
                },
            )
            // Sign (if available from fetched info)
            .when_some(
                fetched.as_ref().and_then(|f| {
                    if f.sign.is_empty() {
                        None
                    } else {
                        Some(f.sign.clone())
                    }
                }),
                |this, sign| {
                    this.child(
                        div()
                            .w_full()
                            .px_2()
                            .py(px(6.0))
                            .rounded(px(4.0))
                            .bg(Colors::bg_hover())
                            .text_size(px(11.0))
                            .text_color(Colors::text_secondary())
                            .overflow_hidden()
                            .line_clamp(3)
                            .child(sign),
                    )
                },
            )
            // Live room info (if available and streaming)
            .when_some(
                fetched.as_ref().and_then(|f| f.live_room.clone()),
                |this, live_room| {
                    let is_live = live_room.live_status == 1;
                    let room_url = live_room.url.clone();
                    this.child(
                        div()
                            .id("live-room-link")
                            .w_full()
                            .px_2()
                            .py(px(6.0))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .bg(if is_live {
                                hsla(0.0, 0.7, 0.55, 0.15)
                            } else {
                                Colors::bg_hover()
                            })
                            .hover(|s| s.opacity(0.8))
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(div().size(px(8.0)).rounded_full().bg(if is_live {
                                        hsla(0.0, 0.8, 0.55, 1.0)
                                    } else {
                                        Colors::text_muted()
                                    }))
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(if is_live {
                                                hsla(0.0, 0.8, 0.55, 1.0)
                                            } else {
                                                Colors::text_muted()
                                            })
                                            .child(if is_live { "直播中" } else { "直播间" }),
                                    )
                                    .when(!live_room.title.is_empty(), |el| {
                                        el.child(
                                            div()
                                                .flex_1()
                                                .text_size(px(10.0))
                                                .text_color(Colors::text_muted())
                                                .overflow_hidden()
                                                .text_ellipsis()
                                                .child(live_room.title.clone()),
                                        )
                                    }),
                            )
                            .on_click(move |_, _, cx| {
                                if !room_url.is_empty() {
                                    cx.open_url(&room_url);
                                }
                            }),
                    )
                },
            )
            // Medal info (if available)
            .when(!medal.medal_name.is_empty(), |this| {
                let (medal_bg, medal_border) = Colors::medal_colors(medal.medal_level);
                let guard_level = medal.guard_level;

                this.child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(Colors::text_muted())
                                .child("粉丝牌:"),
                        )
                        .child(
                            h_flex()
                                .h(px(18.0))
                                .rounded_sm()
                                .border_1()
                                .border_color(medal_border)
                                .overflow_hidden()
                                .child(
                                    h_flex()
                                        .h_full()
                                        .px(px(4.0))
                                        .gap(px(2.0))
                                        .items_center()
                                        .bg(medal_bg)
                                        .when((1..=3).contains(&guard_level), |el| {
                                            if let Some(icon_url) = guard_icon_url(guard_level) {
                                                el.child(
                                                    img(icon_url)
                                                        .size(px(14.0))
                                                        .object_fit(ObjectFit::Contain),
                                                )
                                            } else {
                                                el
                                            }
                                        })
                                        .child(
                                            div()
                                                .text_size(px(11.0))
                                                .text_color(gpui::white())
                                                .child(medal.medal_name.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .h_full()
                                        .px(px(4.0))
                                        .min_w(px(18.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(gpui::white())
                                        .text_size(px(11.0))
                                        .text_color(medal_bg.opacity(1.0))
                                        .child(medal.medal_level.to_string()),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(Colors::text_muted())
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(format!("({})", medal.anchor_uname)),
                        ),
                )
            })
            // Action buttons row
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .mt_1()
                    // Open space button
                    .child(
                        div()
                            .id("open-space-btn")
                            .flex_1()
                            .px_2()
                            .py(px(6.0))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .bg(Colors::accent())
                            .hover(|s| s.opacity(0.8))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(11.0))
                            .text_color(gpui::white())
                            .child("访问主页")
                            .on_click(move |_, _, cx| {
                                cx.open_url(&space_url_clone);
                            }),
                    ),
            )
            // Danmu history section
            .when(!danmu_history.is_empty(), |this| {
                this.child(
                    v_flex()
                        .w_full()
                        .gap_1()
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(Colors::text_muted())
                                .child(format!("发言记录 ({})", danmu_history.len())),
                        )
                        .child(
                            div()
                                .id("danmu-history-container")
                                .w_full()
                                .h(px(150.0))
                                .rounded(px(4.0))
                                .bg(Colors::bg_hover())
                                .overflow_y_scroll()
                                .p_2()
                                .child(
                                    v_flex().gap(px(6.0)).children(
                                        danmu_history
                                            .iter()
                                            .take(50)
                                            .map(|(content, timestamp)| {
                                                let time_str = Local
                                                    .timestamp_opt(*timestamp, 0)
                                                    .single()
                                                    .map(|dt| {
                                                        dt.format("%Y-%m-%d %H:%M:%S").to_string()
                                                    })
                                                    .unwrap_or_default();
                                                v_flex()
                                                    .gap(px(1.0))
                                                    .child(
                                                        div()
                                                            .text_size(px(9.0))
                                                            .text_color(Colors::text_muted())
                                                            .child(time_str),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_size(px(11.0))
                                                            .text_color(Colors::text_secondary())
                                                            .line_clamp(2)
                                                            .child(content.clone()),
                                                    )
                                            })
                                            .collect::<Vec<_>>(),
                                    ),
                                ),
                        ),
                )
            })
            // Loading indicator when fetching
            .when(fetched.is_none(), |this| {
                this.child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .items_center()
                        .justify_center()
                        .py_1()
                        .child(div().size(px(10.0)).rounded_full().bg(Colors::accent()))
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(Colors::text_muted())
                                .child("加载中..."),
                        ),
                )
            })
    }
}
