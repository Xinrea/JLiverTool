//! Content rendering utilities
//!
//! This module contains utilities for rendering content with special formatting:
//! - BV video link detection and rendering
//! - Guard icon URLs and level names
//! - DisplayMessage enum for unified message handling

use crate::theme::Colors;
use gpui::*;
use gpui_component::h_flex;
use jlivertool_core::messages::{
    DanmuMessage, EntryEffectMessage, GiftMessage, GuardMessage, InteractMessage, SuperChatMessage,
};
use regex::Regex;
use std::sync::LazyLock;

/// Static regex for matching BV video IDs (compiled once)
static BV_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(BV[0-9a-zA-Z]+)").unwrap());

/// Render content with BV links as clickable elements
pub fn render_content_with_links(
    content: &str,
    font_size: f32,
    text_color: Hsla,
    item_index: usize,
) -> Stateful<Div> {
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

    // Wrap in a div
    div()
        .id(SharedString::from(format!("content-{}", item_index)))
        .overflow_hidden()
        .child(container)
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

/// A single row in the rendered danmu list.
/// For messages that fit in one line, there is one RenderRow per DisplayMessage.
/// For long danmu messages, the content is split across two RenderRows.
/// SuperChat messages are always split into header + content rows.
#[derive(Clone)]
pub enum RenderRow {
    /// A complete message that fits in one row
    Full(DisplayMessage),
    /// First row of a wrapped danmu: shows medal + username + first portion of content
    DanmuFirstLine {
        danmu: DanmuMessage,
        content_slice: String,
    },
    /// Continuation row of a wrapped danmu: remaining content
    DanmuContinuation {
        danmu: DanmuMessage,
        content_slice: String,
        continuation_index: usize,
    },
    /// SuperChat header row: avatar + price + username
    SuperChatHeader {
        sc: SuperChatMessage,
    },
    /// SuperChat content row: message text
    SuperChatContent {
        sc: SuperChatMessage,
        content_slice: String,
        continuation_index: usize,
        is_last: bool,
    },
}

/// Estimate the rendered width of a string in pixels.
/// CJK characters are approximately `font_size` wide.
/// ASCII characters are approximately `font_size * 0.55` wide.
pub fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    let mut width = 0.0f32;
    for ch in text.chars() {
        if ch.is_ascii() {
            width += font_size * 0.55;
        } else {
            width += font_size;
        }
    }
    width
}

/// Estimate the pixel width of the prefix elements (medal badge + username + colon + gaps).
pub fn estimate_danmu_prefix_width(
    danmu: &DanmuMessage,
    font_size: f32,
    lite_mode: bool,
    medal_display: bool,
) -> f32 {
    let mut width = 0.0f32;

    // Padding: px_2 = 8px each side in normal, px_1 = 4px each side in lite
    let padding = if lite_mode { 4.0 * 2.0 } else { 8.0 * 2.0 };
    width += padding;

    let sender = &danmu.sender;
    let medal = &sender.medal_info;
    let show_medal = medal_display && !medal.medal_name.is_empty() && medal.is_lighted && !lite_mode;

    // Medal badge width (approximate)
    if show_medal {
        let medal_font_size = (font_size * 0.75).clamp(8.0, 12.0);
        let medal_height = (font_size * 1.2).clamp(14.0, 20.0);
        let guard_icon_width = if medal.guard_level >= 1 && medal.guard_level <= 3 {
            medal_height - 2.0 + 2.0
        } else {
            0.0
        };
        let medal_name_width = estimate_text_width(&medal.medal_name, medal_font_size);
        let medal_level_width = medal_height;
        let medal_total = guard_icon_width + medal_name_width + 6.0 + medal_level_width + 6.0 + 2.0;
        width += medal_total;
        width += 4.0; // gap_1
    }

    // Username text
    let uname_text = if lite_mode {
        sender.uname.clone()
    } else {
        format!("{}:", sender.uname)
    };
    width += estimate_text_width(&uname_text, font_size);

    // In lite mode, colon is a separate element
    if lite_mode {
        width += estimate_text_width(":", font_size);
        width += 4.0; // gap between uname and colon
    }

    // Reply indicator (non-lite only)
    if !lite_mode {
        if let Some(reply) = &danmu.reply_uname {
            width += estimate_text_width(&format!("@{}", reply), font_size);
            width += 4.0; // gap_1
        }
    }

    // gap_1 between username and content
    width += 4.0;

    width
}

/// Split content into lines that fit within the given pixel widths.
/// First line uses `first_line_width`, subsequent lines use `continuation_width`.
pub fn split_content_to_lines(
    content: &str,
    font_size: f32,
    first_line_width: f32,
    continuation_width: f32,
) -> Vec<String> {
    if content.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut remaining = content;
    let mut is_first = true;

    while !remaining.is_empty() {
        let available = if is_first { first_line_width } else { continuation_width };
        let mut current_width = 0.0f32;
        let mut split_pos = 0;

        for (byte_idx, ch) in remaining.char_indices() {
            let char_width = if ch.is_ascii() {
                font_size * 0.55
            } else {
                font_size
            };

            if current_width + char_width > available && split_pos > 0 {
                break;
            }
            current_width += char_width;
            split_pos = byte_idx + ch.len_utf8();
        }

        // Force at least one character per line
        if split_pos == 0 && !remaining.is_empty() {
            let ch = remaining.chars().next().unwrap();
            split_pos = ch.len_utf8();
        }

        lines.push(remaining[..split_pos].to_string());
        remaining = &remaining[split_pos..];
        is_first = false;
    }

    lines
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
