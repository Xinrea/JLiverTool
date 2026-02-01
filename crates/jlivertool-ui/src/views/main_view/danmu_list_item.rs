//! Danmu list item view
//!
//! This module contains the unified view for rendering different types of
//! messages in the danmu list (danmu, interact, entry effect, gift, guard, superchat).

use super::content_rendering::{guard_icon_url, guard_level_name, render_content_with_links, DisplayMessage};
use super::user_info_card::{SelectedUser, SelectedUserState};
use crate::theme::Colors;
use gpui::*;
use gpui_component::h_flex;
use gpui_component::tooltip::Tooltip;
use jlivertool_core::messages::{
    DanmuMessage, EntryEffectMessage, GiftMessage, GuardMessage, InteractMessage, SuperChatMessage,
};

/// Unified view for rendering any display message type
pub struct DanmuListItemView {
    message: DisplayMessage,
    index: usize,
    font_size: f32,
    lite_mode: bool,
    medal_display: bool,
    selected_user: SelectedUserState,
}

impl DanmuListItemView {
    pub fn new(
        message: DisplayMessage,
        index: usize,
        font_size: f32,
        lite_mode: bool,
        medal_display: bool,
        selected_user: SelectedUserState,
    ) -> Self {
        Self {
            message,
            index,
            font_size,
            lite_mode,
            medal_display,
            selected_user,
        }
    }

    fn row_height(&self) -> f32 {
        if self.lite_mode {
            self.font_size + 6.0
        } else {
            self.font_size + 10.0
        }
    }

    fn render_danmu(&self, danmu: &DanmuMessage) -> Div {
        let font_size = self.font_size;
        let lite_mode = self.lite_mode;
        let row_height = self.row_height();
        let sender = &danmu.sender;
        let medal = &sender.medal_info;
        let show_medal = self.medal_display && !medal.medal_name.is_empty() && medal.is_lighted;

        // Use fixed height for uniform_list
        let mut el = h_flex()
            .w_full()
            .h(px(row_height))
            .gap_1()
            .items_center()
            .rounded_sm()
            .hover(|s| s.bg(Colors::bg_hover()))
            .overflow_hidden();

        if lite_mode {
            el = el.px_1();
        } else {
            el = el.px_2();
        }

        // Medal badge
        if show_medal && !lite_mode {
            let (medal_bg, medal_border) = Colors::medal_colors(medal.medal_level);
            let guard_level = medal.guard_level;
            let medal_height = (font_size * 1.2).clamp(14.0, 20.0);
            let medal_font_size = (font_size * 0.75).clamp(8.0, 12.0);

            let mut medal_name_el = h_flex()
                .h_full()
                .px(px(3.0))
                .gap(px(2.0))
                .items_center()
                .bg(medal_bg);

            if let Some(icon_url) = guard_icon_url(guard_level) {
                medal_name_el = medal_name_el.child(
                    img(icon_url)
                        .size(px(medal_height - 2.0))
                        .object_fit(ObjectFit::Contain),
                );
            }

            medal_name_el = medal_name_el.child(
                div()
                    .text_size(px(medal_font_size))
                    .text_color(gpui::white())
                    .child(medal.medal_name.clone()),
            );

            el = el.child(
                h_flex()
                    .flex_shrink_0()
                    .h(px(medal_height))
                    .rounded_sm()
                    .border_1()
                    .border_color(medal_border)
                    .overflow_hidden()
                    .child(medal_name_el)
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
            );
        }

        // Username
        let username_color = if danmu.is_special {
            Colors::warning()
        } else {
            Colors::accent()
        };

        let item_index = self.index;
        let selected_user = self.selected_user.clone();
        let sender_clone = sender.clone();

        if lite_mode {
            let selected_user_lite = selected_user.clone();
            let sender_lite = sender_clone.clone();
            el = el
                .child(
                    div()
                        .id(SharedString::from(format!("user-{}", item_index)))
                        .text_size(px(font_size))
                        .text_color(username_color)
                        .cursor_pointer()
                        .child(sender.uname.clone())
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            *selected_user_lite.borrow_mut() = Some(SelectedUser {
                                sender: sender_lite.clone(),
                                fetched_info: None,
                                fetch_requested: false,
                            });
                            cx.refresh_windows();
                        }),
                )
                .child(
                    div()
                        .text_size(px(font_size))
                        .text_color(Colors::text_muted())
                        .child(":"),
                );
        } else {
            el = el.child(
                div()
                    .id(SharedString::from(format!("user-{}", item_index)))
                    .text_size(px(font_size))
                    .text_color(username_color)
                    .cursor_pointer()
                    .child(format!("{}:", sender.uname))
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        *selected_user.borrow_mut() = Some(SelectedUser {
                            sender: sender_clone.clone(),
                            fetched_info: None,
                            fetch_requested: false,
                        });
                        cx.refresh_windows();
                    }),
            );
        }

        // Reply indicator
        if let Some(reply) = &danmu.reply_uname {
            if !lite_mode {
                el = el.child(
                    div()
                        .text_size(px(font_size))
                        .text_color(Colors::text_muted())
                        .child(format!("@{}", reply)),
                );
            }
        }

        // Content or Emoji
        if let Some(emoji) = &danmu.emoji_content {
            // For emoji danmu, display emoji image constrained to row height
            let emoji_size = row_height - 4.0;

            el = el.child(
                img(emoji.url.clone())
                    .max_w(px(emoji_size))
                    .max_h(px(emoji_size))
                    .object_fit(ObjectFit::Contain),
            );
        } else {
            // Regular text content with BV link support and tooltip for long content
            el = el.child(
                render_content_with_links(&danmu.content, font_size, Colors::text_primary(), item_index)
                    .flex_1(),
            );
        }

        el
    }

    fn render_interact(&self, interact: &InteractMessage) -> Div {
        let font_size = self.font_size;
        let lite_mode = self.lite_mode;
        let row_height = self.row_height();

        let action_text = if interact.action == 2 {
            "关注了直播间"
        } else {
            "进入了直播间"
        };

        let mut el = h_flex()
            .w_full()
            .h(px(row_height))
            .gap_1()
            .items_center()
            .rounded_sm()
            .hover(|s| s.bg(Colors::bg_hover()))
            .overflow_hidden();

        if lite_mode {
            el = el.px_1();
        } else {
            el = el.px_2();
        }

        el = el
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(Colors::text_muted())
                    .child(interact.sender.uname.clone()),
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(Colors::text_muted())
                    .child(action_text),
            );

        el
    }

    fn render_entry_effect(&self, entry: &EntryEffectMessage) -> Div {
        let font_size = self.font_size;
        let lite_mode = self.lite_mode;
        let row_height = self.row_height();

        let privilege_type = entry.privilege_type;
        let level_name = guard_level_name(privilege_type);
        let icon_size = (font_size * 1.2).clamp(14.0, 20.0);

        // Golden color #EDB83F
        let entry_color = hsla(42.0 / 360.0, 0.85, 0.59, 1.0);

        let mut el = h_flex()
            .w_full()
            .h(px(row_height))
            .gap_1()
            .items_center()
            .rounded_sm()
            .border_l_2()
            .border_color(entry_color)
            .hover(|s| s.bg(Colors::bg_hover()))
            .overflow_hidden();

        if lite_mode {
            el = el.px_1();
        } else {
            el = el.px_2();
        }

        if let Some(icon_url) = guard_icon_url(privilege_type) {
            el = el.child(
                img(icon_url)
                    .size(px(icon_size))
                    .object_fit(ObjectFit::Contain),
            );
        }

        el = el.child(
            div()
                .text_size(px(font_size))
                .text_color(entry_color)
                .child(format!("{} {} 进入直播间", level_name, entry.sender.uname)),
        );

        el
    }

    fn render_gift(&self, gift: &GiftMessage) -> Div {
        let font_size = self.font_size;
        let lite_mode = self.lite_mode;
        let row_height = self.row_height();

        let is_paid = gift.gift_info.coin_type != "silver";
        let price_text = if is_paid {
            format!("¥{:.2}", gift.gift_info.price as f64 / 1000.0)
        } else {
            "免费".to_string()
        };

        // Golden color #EDB83F for paid gifts, secondary for free
        let gift_color = if is_paid {
            hsla(42.0 / 360.0, 0.85, 0.59, 1.0)
        } else {
            Colors::text_secondary()
        };

        let mut el = h_flex()
            .w_full()
            .h(px(row_height))
            .gap_1()
            .items_center()
            .rounded_sm()
            .border_l_2()
            .border_t_1()
            .border_b_1()
            .border_color(gift_color)
            .hover(|s| s.bg(Colors::bg_hover()))
            .overflow_hidden();

        if lite_mode {
            el = el.px_1();
        } else {
            el = el.px_2();
        }

        el = el
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(Colors::text_secondary())
                    .child(gift.sender.uname.clone()),
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(Colors::text_secondary())
                    .child(gift.action.clone()),
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .font_weight(FontWeight::BOLD)
                    .text_color(gift_color)
                    .child(gift.gift_info.name.clone()),
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(Colors::text_secondary())
                    .child(format!("x{}", gift.num)),
            );

        if !lite_mode && is_paid {
            el = el.child(
                div()
                    .text_size(px(font_size * 0.9))
                    .text_color(gift_color)
                    .child(price_text),
            );
        }

        el
    }

    fn render_guard(&self, guard: &GuardMessage) -> Div {
        let font_size = self.font_size;
        let lite_mode = self.lite_mode;
        let row_height = self.row_height();

        let guard_name = guard_level_name(guard.guard_level);
        let price_text = format!("¥{:.2}", guard.price as f64 / 1000.0);
        let icon_size = (font_size * 1.2).clamp(14.0, 20.0);

        // Golden color #EDB83F for guard messages
        let guard_color = hsla(42.0 / 360.0, 0.85, 0.59, 1.0);

        let mut el = h_flex()
            .w_full()
            .h(px(row_height))
            .gap_1()
            .items_center()
            .rounded_sm()
            .border_l_2()
            .border_color(guard_color)
            .bg(guard_color.opacity(0.1))
            .hover(|s| s.bg(guard_color.opacity(0.15)))
            .overflow_hidden();

        if lite_mode {
            el = el.px_1();
        } else {
            el = el.px_2();
        }

        if let Some(icon_url) = guard_icon_url(guard.guard_level) {
            el = el.child(
                img(icon_url)
                    .size(px(icon_size))
                    .object_fit(ObjectFit::Contain),
            );
        }

        el = el
            .child(
                div()
                    .text_size(px(font_size))
                    .font_weight(FontWeight::BOLD)
                    .text_color(guard_color)
                    .child(guard.sender.uname.clone()),
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(Colors::text_secondary())
                    .child("开通了"),
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .font_weight(FontWeight::BOLD)
                    .text_color(guard_color)
                    .child(guard_name),
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(Colors::text_secondary())
                    .child(format!("{}{}", guard.num, guard.unit)),
            );

        if !lite_mode {
            el = el.child(
                div()
                    .text_size(px(font_size * 0.9))
                    .font_weight(FontWeight::BOLD)
                    .text_color(guard_color)
                    .child(price_text),
            );
        }

        el
    }

    fn render_superchat(&self, sc: &SuperChatMessage) -> Div {
        let font_size = self.font_size;
        let lite_mode = self.lite_mode;
        let row_height = self.row_height();
        let item_index = self.index;

        // Golden color #EDB83F for superchat
        let sc_color = hsla(42.0 / 360.0, 0.85, 0.59, 1.0);

        // Get first character of username for avatar
        let avatar_char = sc.sender.uname.chars().next().unwrap_or('U').to_string();

        // Use fixed height for uniform_list
        let mut el = h_flex()
            .w_full()
            .h(px(row_height))
            .gap_2()
            .items_center()
            .rounded(px(4.0))
            .bg(sc_color.opacity(0.15))
            .border_l_4()
            .border_color(sc_color)
            .overflow_hidden();

        if lite_mode {
            el = el.px_1();
        } else {
            el = el.px_2();
        }

        // Clone message for tooltip
        let message_for_tooltip = sc.message.clone();
        let show_tooltip = sc.message.len() >= 15;

        let message_el = div()
            .id(SharedString::from(format!("sc-msg-{}", item_index)))
            .flex_1()
            .text_size(px(font_size * 0.9))
            .text_color(Colors::text_secondary())
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis()
            .child(sc.message.clone());

        let message_el = if show_tooltip {
            message_el.tooltip(move |window, cx| {
                let content = message_for_tooltip.clone();
                Tooltip::element(move |_, _| {
                    div()
                        .max_w(px(300.0))
                        .child(content.clone())
                })
                .build(window, cx)
            })
        } else {
            message_el
        };

        el = el
            // Avatar circle with user initial
            .child(
                div()
                    .flex_shrink_0()
                    .size(px(row_height * 0.7))
                    .rounded_full()
                    .bg(sc_color)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(font_size * 0.7))
                    .font_weight(FontWeight::BOLD)
                    .text_color(gpui::white())
                    .child(avatar_char),
            )
            // Price badge
            .child(
                div()
                    .flex_shrink_0()
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(10.0))
                    .bg(sc_color)
                    .text_size(px(font_size * 0.75))
                    .font_weight(FontWeight::BOLD)
                    .text_color(gpui::white())
                    .child(format!("¥{}", sc.price)),
            )
            // Username
            .child(
                div()
                    .flex_shrink_0()
                    .text_size(px(font_size * 0.9))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(Colors::text_primary())
                    .child(sc.sender.uname.clone()),
            )
            // Message with tooltip
            .child(message_el);

        el
    }
}

impl Render for DanmuListItemView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}

impl DanmuListItemView {
    /// Render the element directly without going through the Render trait.
    /// This is used by uniform_list to avoid creating entities which would
    /// reset tooltip state on each render.
    pub fn render_element(&self) -> Div {
        match &self.message {
            DisplayMessage::Danmu(danmu) => self.render_danmu(danmu),
            DisplayMessage::Interact(interact) => self.render_interact(interact),
            DisplayMessage::EntryEffect(entry) => self.render_entry_effect(entry),
            DisplayMessage::Gift(gift) => self.render_gift(gift),
            DisplayMessage::Guard(guard) => self.render_guard(guard),
            DisplayMessage::SuperChat(sc) => self.render_superchat(sc),
        }
    }
}
