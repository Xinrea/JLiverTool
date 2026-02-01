//! Content rendering utilities
//!
//! This module contains utilities for rendering content with special formatting:
//! - BV video link detection and rendering
//! - Guard icon URLs and level names
//! - DisplayMessage enum for unified message handling

use crate::theme::Colors;
use gpui::*;
use gpui_component::h_flex;
use gpui_component::tooltip::Tooltip;
use jlivertool_core::messages::{
    DanmuMessage, EntryEffectMessage, GiftMessage, GuardMessage, InteractMessage, SuperChatMessage,
};
use regex::Regex;
use std::sync::LazyLock;

/// Static regex for matching BV video IDs (compiled once)
static BV_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(BV[0-9a-zA-Z]+)").unwrap());

/// Minimum content length to show tooltip (shorter content unlikely to be truncated)
const TOOLTIP_MIN_LENGTH: usize = 15;

/// Render content with BV links as clickable elements and tooltip for long content
pub fn render_content_with_links(
    content: &str,
    font_size: f32,
    text_color: Hsla,
    item_index: usize,
) -> Stateful<Div> {
    let content_for_tooltip = content.to_string();
    let show_tooltip = content.len() >= TOOLTIP_MIN_LENGTH;

    tracing::debug!(
        "render_content_with_links: index={}, len={}, show_tooltip={}",
        item_index,
        content.len(),
        show_tooltip
    );

    let mut container = h_flex().gap(px(0.0)).items_center().overflow_hidden();
    let mut last_end = 0;

    for (link_index, cap) in BV_REGEX.captures_iter(content).enumerate() {
        let m = cap.get(0).unwrap();

        // Add text before the match
        if m.start() > last_end {
            let text_before = &content[last_end..m.start()];
            container = container.child(
                div()
                    .text_size(px(font_size))
                    .text_color(text_color)
                    .child(text_before.to_string()),
            );
        }

        // Add the BV link
        let bv_id = m.as_str().to_string();
        let url = format!("https://www.bilibili.com/video/{}", bv_id);
        let link_color = Colors::accent();
        container = container.child(
            div()
                .id(SharedString::from(format!("bv-link-{}-{}", item_index, link_index)))
                .text_size(px(font_size))
                .text_color(link_color)
                .cursor_pointer()
                .child(bv_id)
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.open_url(&url);
                }),
        );

        last_end = m.end();
    }

    // Add remaining text after the last match (only if there were matches)
    if last_end > 0 && last_end < content.len() {
        let text_after = &content[last_end..];
        container = container.child(
            div()
                .text_size(px(font_size))
                .text_color(text_color)
                .child(text_after.to_string()),
        );
    }

    // If no matches found, just return the plain text
    if last_end == 0 {
        container = container.child(
            div()
                .text_size(px(font_size))
                .text_color(text_color)
                .child(content.to_string()),
        );
    }

    // Wrap in a div with tooltip for long content
    let wrapper = div()
        .id(SharedString::from(format!("content-{}", item_index)))
        .overflow_hidden()
        .text_ellipsis()
        .child(container);

    if show_tooltip {
        let tooltip_content = content_for_tooltip.clone();
        tracing::debug!("Adding tooltip for content-{}: '{}'", item_index, &tooltip_content[..tooltip_content.len().min(50)]);
        wrapper.tooltip(move |window, cx| {
            tracing::debug!("Tooltip builder called for content: '{}'", &tooltip_content[..tooltip_content.len().min(50)]);
            let content = tooltip_content.clone();
            Tooltip::element(move |_, _| {
                div()
                    .max_w(px(300.0))
                    .child(content.clone())
            })
            .build(window, cx)
        })
    } else {
        wrapper
    }
}

/// Unified display message type for the danmu list
#[derive(Clone)]
pub enum DisplayMessage {
    Danmu(DanmuMessage),
    Interact(InteractMessage),
    EntryEffect(EntryEffectMessage),
    Gift(GiftMessage),
    Guard(GuardMessage),
    SuperChat(SuperChatMessage),
}

/// Guard icon URLs from Bilibili CDN
const GUARD_ICON_1: &str = "https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard1.png@44w_44h.webp";
const GUARD_ICON_2: &str = "https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard2.png@44w_44h.webp";
const GUARD_ICON_3: &str = "https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard3.png@44w_44h.webp";

/// Get guard icon URL by level
pub fn guard_icon_url(level: u8) -> Option<&'static str> {
    match level {
        1 => Some(GUARD_ICON_1),
        2 => Some(GUARD_ICON_2),
        3 => Some(GUARD_ICON_3),
        _ => None,
    }
}

/// Get guard level name
pub fn guard_level_name(level: u8) -> &'static str {
    match level {
        1 => "总督",
        2 => "提督",
        3 => "舰长",
        _ => "",
    }
}
