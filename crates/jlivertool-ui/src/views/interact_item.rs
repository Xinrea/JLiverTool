//! Interact and entry effect item view

use crate::theme::Colors;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::h_flex;
use jlivertool_core::messages::{EntryEffectMessage, InteractMessage};

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

/// Get guard level name
fn guard_level_name(level: u8) -> &'static str {
    match level {
        1 => "总督",
        2 => "提督",
        3 => "舰长",
        _ => "",
    }
}

/// Interact item view (user enter/follow)
pub struct InteractItemView {
    message: InteractMessage,
    font_size: f32,
    lite_mode: bool,
}

impl InteractItemView {
    pub fn new(message: InteractMessage) -> Self {
        Self {
            message,
            font_size: 14.0,
            lite_mode: false,
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
}

impl Render for InteractItemView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let font_size = self.font_size;
        let lite_mode = self.lite_mode;
        let row_height = if lite_mode {
            font_size + 6.0
        } else {
            font_size + 10.0
        };

        let action_text = if self.message.action == 2 {
            "关注了直播间"
        } else {
            "进入了直播间"
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
            // Username
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(Colors::text_muted())
                    .child(self.message.sender.uname.clone()),
            )
            // Action text
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(Colors::text_muted())
                    .child(action_text),
            )
    }
}

/// Entry effect item view (guard/level entry)
pub struct EntryEffectItemView {
    message: EntryEffectMessage,
    font_size: f32,
    lite_mode: bool,
}

impl EntryEffectItemView {
    pub fn new(message: EntryEffectMessage) -> Self {
        Self {
            message,
            font_size: 14.0,
            lite_mode: false,
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
}

impl Render for EntryEffectItemView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let font_size = self.font_size;
        let lite_mode = self.lite_mode;
        let row_height = if lite_mode {
            font_size + 6.0
        } else {
            font_size + 10.0
        };

        let privilege_type = self.message.privilege_type;
        let level_name = guard_level_name(privilege_type);
        let icon_size = (font_size * 1.2).clamp(14.0, 20.0);

        h_flex()
            .h(px(row_height))
            .gap_1()
            .when(lite_mode, |this| this.px_1())
            .when(!lite_mode, |this| this.px_2())
            .items_center()
            .rounded_sm()
            .hover(|s| s.bg(Colors::bg_hover()))
            .overflow_hidden()
            // Guard icon (if applicable)
            .when_some(guard_icon_url(privilege_type), |this, icon_url| {
                this.child(
                    img(icon_url)
                        .size(px(icon_size))
                        .object_fit(ObjectFit::Contain),
                )
            })
            // Level name + username + action
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(Colors::warning())
                    .child(format!(
                        "{} {} 进入直播间",
                        level_name,
                        self.message.sender.uname
                    )),
            )
    }
}
