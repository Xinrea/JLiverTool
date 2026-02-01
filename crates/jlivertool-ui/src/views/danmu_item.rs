//! Danmaku item view

use crate::theme::Colors;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::h_flex;
use jlivertool_core::messages::DanmuMessage;

/// Guard icon URLs from Bilibili CDN
const GUARD_ICON_1: &str = "https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard1.png@44w_44h.webp";
const GUARD_ICON_2: &str = "https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard2.png@44w_44h.webp";
const GUARD_ICON_3: &str = "https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard3.png@44w_44h.webp";

/// Get guard icon URL by level
fn guard_icon_url(level: u8) -> Option<&'static str> {
    match level {
        1 => Some(GUARD_ICON_1), // 总督
        2 => Some(GUARD_ICON_2), // 提督
        3 => Some(GUARD_ICON_3), // 舰长
        _ => None,
    }
}

/// Danmaku item view
pub struct DanmuItemView {
    message: DanmuMessage,
    font_size: f32,
    lite_mode: bool,
    medal_display: bool,
}

impl DanmuItemView {
    pub fn new(message: DanmuMessage) -> Self {
        Self {
            message,
            font_size: 14.0,
            lite_mode: false,
            medal_display: true,
        }
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn with_lite_mode(mut self, lite_mode: bool) -> Self {
        self.lite_mode = lite_mode;
        self
    }

    pub fn with_medal_display(mut self, medal_display: bool) -> Self {
        self.medal_display = medal_display;
        self
    }
}

impl Render for DanmuItemView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let sender = &self.message.sender;
        let medal = &sender.medal_info;
        // Only show medal if medal_display is enabled, has a name, and is lighted (not gray)
        let show_medal = self.medal_display && !medal.medal_name.is_empty() && medal.is_lighted;
        let font_size = self.font_size;
        let lite_mode = self.lite_mode;
        // Calculate row height based on font size (smaller in lite mode)
        let row_height = if lite_mode {
            font_size + 6.0
        } else {
            font_size + 10.0
        };

        h_flex()
            .h(px(row_height))
            .gap_1()
            .when(lite_mode, |this| this.px_1())
            .when(!lite_mode, |this| this.px_2())
            .items_center()
            .rounded_sm()
            .hover(|s| s.bg(Colors::bg_hover()))
            .overflow_hidden()
            // Medal badge (only show if medal_display is enabled and medal is lighted)
            .when(show_medal && !lite_mode, |this| {
                let (medal_bg, medal_border) = Colors::medal_colors(medal.medal_level);
                let guard_level = medal.guard_level;
                // Scale medal size with font size
                let medal_height = (font_size * 1.2).clamp(14.0, 20.0);
                let medal_font_size = (font_size * 0.75).clamp(8.0, 12.0);

                this.child(
                    h_flex()
                        .h(px(medal_height))
                        .rounded_sm()
                        .border_1()
                        .border_color(medal_border)
                        .overflow_hidden()
                        .child(
                            h_flex()
                                .h_full()
                                .px(px(3.0))
                                .gap(px(2.0))
                                .items_center()
                                .bg(medal_bg)
                                // Guard icon (舰长/提督/总督)
                                .when_some(guard_icon_url(guard_level), |this, icon_url| {
                                    this.child(
                                        img(icon_url)
                                            .size(px(medal_height - 2.0))
                                            .object_fit(ObjectFit::Contain),
                                    )
                                })
                                // Medal name - use fixed white color since bg is from API
                                .child(
                                    div()
                                        .text_size(px(medal_font_size))
                                        .text_color(gpui::white())
                                        .child(medal.medal_name.clone()),
                                ),
                        )
                        // Medal level (white background)
                        .child(
                            div()
                                .h_full()
                                .px(px(3.0))
                                .min_w(px(medal_height))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(gpui::white())
                                .text_size(px(medal_font_size))
                                .text_color(medal_bg.opacity(1.0))
                                .child(medal.medal_level.to_string()),
                        ),
                )
            })
            // Username (hide in lite mode)
            .when(!lite_mode, |this| {
                this.child(
                    div()
                        .text_size(px(font_size))
                        .text_color(if self.message.is_special {
                            Colors::warning()
                        } else {
                            Colors::accent()
                        })
                        .child(format!("{}:", sender.uname)),
                )
            })
            // In lite mode, show username inline with content
            .when(lite_mode, |this| {
                this.child(
                    div()
                        .text_size(px(font_size))
                        .text_color(if self.message.is_special {
                            Colors::warning()
                        } else {
                            Colors::accent()
                        })
                        .child(sender.uname.clone()),
                )
                .child(
                    div()
                        .text_size(px(font_size))
                        .text_color(Colors::text_muted())
                        .child(":"),
                )
            })
            // Reply indicator (hide in lite mode)
            .when(self.message.reply_uname.is_some() && !lite_mode, |this| {
                let reply = self.message.reply_uname.as_ref().unwrap();
                this.child(
                    div()
                        .text_size(px(font_size))
                        .text_color(Colors::text_muted())
                        .child(format!("@{}", reply)),
                )
            })
            // Content
            .child(
                div()
                    .flex_1()
                    .text_size(px(font_size))
                    .text_color(Colors::text_primary())
                    .child(self.message.content.clone()),
            )
    }
}
