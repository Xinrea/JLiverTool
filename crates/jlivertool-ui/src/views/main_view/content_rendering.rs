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
static BV_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)(BV[0-9a-zA-Z]+)").unwrap());

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
                .id(SharedString::from(format!(
                    "bv-link-{}-{}",
                    item_index, link_index
                )))
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
    /// One fixed-height line of a wrapped gift message.
    GiftLine {
        gift: GiftMessage,
        runs: Vec<GiftTextRun>,
        line_index: usize,
        is_last: bool,
    },
    /// SuperChat header row: avatar + price + username
    SuperChatHeader { sc: SuperChatMessage },
    /// SuperChat content row: message text
    SuperChatContent {
        sc: SuperChatMessage,
        content_slice: String,
        continuation_index: usize,
        is_last: bool,
    },
}

/// Semantic text fragments used when a gift is converted from raw message
/// data into fixed-height render rows. Keeping the role lets the view retain
/// the original per-field colors and font weights after wrapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GiftTextRole {
    Sender,
    Action,
    GiftName,
    Count,
    Price,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GiftTextRun {
    pub role: GiftTextRole,
    pub text: String,
}

/// Build the fixed-height rows used by danmu lists for the supplied width.
///
/// Both the main window and dashboard use this so wrapping behavior remains
/// consistent when either surface is resized.
pub(crate) fn build_render_rows<'a>(
    messages: impl IntoIterator<Item = &'a DisplayMessage>,
    available_width: f32,
    font_size: f32,
    lite_mode: bool,
    medal_display: bool,
) -> Vec<RenderRow> {
    let mut rows = Vec::new();
    for message in messages {
        append_message_rows(
            &mut rows,
            message,
            available_width,
            font_size,
            lite_mode,
            medal_display,
        );
    }
    rows
}

/// Convert one display message into one or more fixed-height render rows.
pub(crate) fn append_message_rows(
    rows: &mut Vec<RenderRow>,
    message: &DisplayMessage,
    available_width: f32,
    font_size: f32,
    lite_mode: bool,
    medal_display: bool,
) {
    match message {
        DisplayMessage::Danmu(danmu) => {
            if danmu.emoji_content.is_some() {
                rows.push(RenderRow::Full(message.clone()));
                return;
            }

            let prefix_width =
                estimate_danmu_prefix_width(danmu, font_size, lite_mode, medal_display);
            let first_line_content_width = available_width - prefix_width;
            let padding = if lite_mode { 4.0 * 2.0 } else { 8.0 * 2.0 };
            let continuation_content_width = available_width - padding;

            let content_width = estimate_text_width(&danmu.content, font_size);
            if content_width <= first_line_content_width || first_line_content_width <= 0.0 {
                rows.push(RenderRow::Full(message.clone()));
            } else {
                let lines = split_content_to_lines(
                    &danmu.content,
                    font_size,
                    first_line_content_width,
                    continuation_content_width,
                );

                if lines.len() <= 1 {
                    rows.push(RenderRow::Full(message.clone()));
                } else {
                    rows.push(RenderRow::DanmuFirstLine {
                        danmu: danmu.clone(),
                        content_slice: lines[0].clone(),
                    });
                    for (index, line) in lines[1..].iter().enumerate() {
                        rows.push(RenderRow::DanmuContinuation {
                            danmu: danmu.clone(),
                            content_slice: line.clone(),
                            continuation_index: index,
                        });
                    }
                }
            }
        }
        DisplayMessage::SuperChat(sc) => {
            rows.push(RenderRow::SuperChatHeader { sc: sc.clone() });

            if !sc.message.is_empty() {
                let padding = if lite_mode { 4.0 * 2.0 } else { 8.0 * 2.0 };
                let content_line_width = available_width - padding - 4.0;
                let content_width = estimate_text_width(&sc.message, font_size * 0.9);

                if content_width <= content_line_width || content_line_width <= 0.0 {
                    rows.push(RenderRow::SuperChatContent {
                        sc: sc.clone(),
                        content_slice: sc.message.clone(),
                        continuation_index: 0,
                        is_last: true,
                    });
                } else {
                    let lines = split_content_to_lines_exact(
                        &sc.message,
                        font_size * 0.9,
                        content_line_width,
                        content_line_width,
                    );
                    let last_index = lines.len().saturating_sub(1);
                    for (index, line) in lines.iter().enumerate() {
                        rows.push(RenderRow::SuperChatContent {
                            sc: sc.clone(),
                            content_slice: line.clone(),
                            continuation_index: index,
                            is_last: index == last_index,
                        });
                    }
                }
            }
        }
        DisplayMessage::Gift(gift) => {
            let runs = gift_text_runs(gift, lite_mode);
            let padding = if lite_mode { 4.0 * 2.0 } else { 8.0 * 2.0 };
            let content_width = available_width - padding - 2.0; // left border
            let lines = split_gift_runs_to_lines(&runs, font_size, content_width);

            if lines.len() <= 1 {
                rows.push(RenderRow::Full(message.clone()));
            } else {
                let last_index = lines.len() - 1;
                for (line_index, runs) in lines.into_iter().enumerate() {
                    rows.push(RenderRow::GiftLine {
                        gift: gift.clone(),
                        runs,
                        line_index,
                        is_last: line_index == last_index,
                    });
                }
            }
        }
        _ => rows.push(RenderRow::Full(message.clone())),
    }
}

fn gift_text_runs(gift: &GiftMessage, lite_mode: bool) -> Vec<GiftTextRun> {
    let mut runs = vec![
        GiftTextRun {
            role: GiftTextRole::Sender,
            text: gift.sender.uname.clone(),
        },
        GiftTextRun {
            role: GiftTextRole::Action,
            text: gift.action.clone(),
        },
        GiftTextRun {
            role: GiftTextRole::GiftName,
            text: gift.gift_info.name.clone(),
        },
        GiftTextRun {
            role: GiftTextRole::Count,
            text: format!("x{}", gift.num),
        },
    ];

    if !lite_mode && gift.gift_info.coin_type != "silver" {
        runs.push(GiftTextRun {
            role: GiftTextRole::Price,
            text: format!("¥{:.2}", gift.gift_info.price as f64 / 1000.0),
        });
    }

    runs
}

fn gift_run_font_size(role: GiftTextRole, font_size: f32) -> f32 {
    if role == GiftTextRole::Price {
        font_size * 0.9
    } else {
        font_size
    }
}

/// Wrap semantic text runs without losing field boundaries. This is the gift
/// equivalent of `split_content_to_lines`, but operates on structured data so
/// the renderer can preserve sender/gift/price styling on continuation rows.
fn split_gift_runs_to_lines(
    runs: &[GiftTextRun],
    font_size: f32,
    available_width: f32,
) -> Vec<Vec<GiftTextRun>> {
    if runs.is_empty() {
        return vec![Vec::new()];
    }

    // `available_width` has already had the list scrollbar and row padding
    // removed by the caller. Applying the danmu splitter's extra 10% safety
    // margin here made gift messages wrap visibly too early.
    let safe_width = available_width.max(font_size);
    let gap_width = 4.0; // h_flex().gap_1()
    let mut lines = Vec::new();
    let mut line = Vec::new();
    let mut line_width = 0.0;

    for run in runs {
        let mut remaining = run.text.as_str();
        let mut starts_new_field = true;

        while !remaining.is_empty() {
            let gap = if starts_new_field && !line.is_empty() {
                gap_width
            } else {
                0.0
            };
            let capacity = safe_width - line_width - gap;

            if capacity <= 0.0 && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
                line_width = 0.0;
                continue;
            }

            let run_font_size = gift_run_font_size(run.role, font_size);
            let remaining_width = estimate_text_width(remaining, run_font_size);

            // Keep short semantic fields intact. In particular, price and
            // count should move to the next row instead of rendering as
            // `¥5.0` / `0` or `x` / `99`.
            let keep_atomic = matches!(run.role, GiftTextRole::Count | GiftTextRole::Price);
            if keep_atomic && remaining_width > capacity && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
                line_width = 0.0;
                continue;
            }

            if remaining_width <= capacity {
                line_width += gap + remaining_width;
                line.push(GiftTextRun {
                    role: run.role,
                    text: remaining.to_string(),
                });
                break;
            }

            let mut consumed = 0;
            let mut consumed_width = 0.0;
            for (byte_index, ch) in remaining.char_indices() {
                let char_width = estimate_text_width(&ch.to_string(), run_font_size);
                if consumed > 0 && consumed_width + char_width > capacity {
                    break;
                }
                consumed_width += char_width;
                consumed = byte_index + ch.len_utf8();
            }

            if consumed == 0 {
                if !line.is_empty() {
                    lines.push(std::mem::take(&mut line));
                    line_width = 0.0;
                    continue;
                }
                let ch = remaining.chars().next().unwrap();
                consumed = ch.len_utf8();
                consumed_width = estimate_text_width(&remaining[..consumed], run_font_size);
            }

            line_width += gap + consumed_width;
            line.push(GiftTextRun {
                role: run.role,
                text: remaining[..consumed].to_string(),
            });
            remaining = &remaining[consumed..];
            starts_new_field = false;

            if !remaining.is_empty() {
                lines.push(std::mem::take(&mut line));
                line_width = 0.0;
            }
        }
    }

    if !line.is_empty() {
        lines.push(line);
    }

    if lines.is_empty() {
        lines.push(Vec::new());
    }
    lines
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
    let show_medal =
        medal_display && !medal.medal_name.is_empty() && medal.is_lighted && !lite_mode;

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
/// BV video IDs are kept intact and not split across lines.
pub fn split_content_to_lines(
    content: &str,
    font_size: f32,
    first_line_width: f32,
    continuation_width: f32,
) -> Vec<String> {
    split_content_to_lines_with_metrics(
        content,
        font_size,
        first_line_width,
        continuation_width,
        true,
    )
}

/// Split SC content using the same width estimate as gift rows. Unlike the
/// danmu splitter this uses the full available width and the shared
/// `estimate_text_width` metrics, so adjacent gift and SC cards wrap
/// consistently.
fn split_content_to_lines_exact(
    content: &str,
    font_size: f32,
    first_line_width: f32,
    continuation_width: f32,
) -> Vec<String> {
    split_content_to_lines_with_metrics(
        content,
        font_size,
        first_line_width,
        continuation_width,
        false,
    )
}

fn split_content_to_lines_with_metrics(
    content: &str,
    font_size: f32,
    first_line_width: f32,
    continuation_width: f32,
    conservative: bool,
) -> Vec<String> {
    if content.is_empty() {
        return vec![String::new()];
    }

    // Find all BV IDs in the content
    let bv_matches: Vec<_> = BV_REGEX.find_iter(content).collect();

    let mut lines = Vec::new();
    let mut remaining = content;
    let mut is_first = true;

    while !remaining.is_empty() {
        let available = if is_first {
            first_line_width
        } else {
            continuation_width
        };
        let safe_width = if conservative {
            // Preserve the existing danmu behavior for now: its first line has
            // several independently estimated prefix elements.
            let safety_margin = (available * 0.1).max(8.0);
            (available - safety_margin).max(font_size)
        } else {
            available.max(font_size)
        };

        let mut current_width = 0.0f32;
        let mut split_pos = 0;
        let mut skip_until = 0;

        let remaining_start = content.len() - remaining.len();

        for (byte_idx, ch) in remaining.char_indices() {
            let absolute_pos = remaining_start + byte_idx;

            // A BV ID is accounted for as one atomic token at its start.
            if absolute_pos < skip_until {
                continue;
            }

            // Check if we're at the start of a BV ID
            let at_bv_start = bv_matches.iter().any(|m| m.start() == absolute_pos);

            if at_bv_start {
                // Find the matching BV ID
                if let Some(bv_match) = bv_matches.iter().find(|m| m.start() == absolute_pos) {
                    let bv_text = bv_match.as_str();
                    let bv_width = estimate_text_width(bv_text, font_size);

                    // Check if the entire BV ID fits in the remaining space
                    if current_width + bv_width > safe_width {
                        // BV ID doesn't fit, break before it
                        if split_pos > 0 {
                            break;
                        } else {
                            // Current line is empty, force include the BV ID
                            current_width += bv_width;
                            split_pos = byte_idx + bv_text.len();
                            skip_until = bv_match.end();
                            continue;
                        }
                    } else {
                        // BV ID fits, add its width and skip to the end of it
                        current_width += bv_width;
                        split_pos = byte_idx + bv_text.len();
                        skip_until = bv_match.end();
                        continue;
                    }
                }
            }

            let char_width = if conservative {
                if ch.is_ascii() {
                    font_size * 0.6
                } else {
                    font_size * 1.1
                }
            } else {
                estimate_text_width(&ch.to_string(), font_size)
            };

            if current_width + char_width > safe_width && split_pos > 0 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use jlivertool_core::messages::GiftInfo;
    use jlivertool_core::types::Sender;

    fn gift(name: &str) -> GiftMessage {
        GiftMessage {
            id: "gift-test".to_string(),
            room: 1,
            gift_info: GiftInfo {
                id: 1,
                name: name.to_string(),
                price: 1000,
                coin_type: "gold".to_string(),
                img_basic: String::new(),
                img_dynamic: String::new(),
                gif: String::new(),
                webp: String::new(),
            },
            sender: Sender {
                uname: "测试用户".to_string(),
                ..Default::default()
            },
            action: "投喂".to_string(),
            num: 2,
            timestamp: 0,
            archived: false,
        }
    }

    #[test]
    fn short_gift_stays_in_one_render_row() {
        let message = DisplayMessage::Gift(gift("花"));
        let mut rows = Vec::new();
        append_message_rows(&mut rows, &message, 500.0, 14.0, false, true);
        assert!(matches!(rows.as_slice(), [RenderRow::Full(_)]));
    }

    #[test]
    fn long_gift_wraps_without_losing_semantic_text() {
        let gift = gift("一个名字特别特别长的测试礼物");
        let expected: String = gift_text_runs(&gift, false)
            .into_iter()
            .map(|run| run.text)
            .collect();
        let mut rows = Vec::new();
        append_message_rows(
            &mut rows,
            &DisplayMessage::Gift(gift),
            130.0,
            14.0,
            false,
            true,
        );

        assert!(rows.len() > 1);
        let actual: String = rows
            .iter()
            .flat_map(|row| match row {
                RenderRow::GiftLine { runs, .. } => runs.iter().map(|run| run.text.as_str()).collect(),
                _ => Vec::new(),
            })
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn gift_wrap_uses_full_available_width_and_keeps_price_intact() {
        let mut gift = gift("特别长礼物名称");
        gift.sender.uname = "JLiverTool 测试员".to_string();
        gift.action = "送出".to_string();
        gift.gift_info.price = 5000;

        let lines = split_gift_runs_to_lines(&gift_text_runs(&gift, false), 14.0, 340.0);

        // This fits in the actual width. The previous extra 10% margin caused
        // it to wrap despite leaving a large blank area on the right.
        assert_eq!(lines.len(), 1);
        assert!(lines.iter().any(|line| {
            line.iter()
                .any(|run| run.role == GiftTextRole::Price && run.text == "¥5.00")
        }));
        assert!(!lines.iter().any(|line| {
            line.iter().any(|run| {
                run.role == GiftTextRole::Price && run.text != "¥5.00"
            })
        }));
    }

    #[test]
    fn superchat_wrap_uses_full_available_width() {
        let content = "这是一条比较长的醒目留言，用于验证多行内容";
        let font_size = 14.0 * 0.9;
        let exact_width = estimate_text_width(content, font_size);

        // The SC splitter should agree with the shared width estimator. The
        // previous danmu-style 10% margin split this into two lines.
        let lines = split_content_to_lines_exact(content, font_size, exact_width + 1.0, exact_width + 1.0);
        assert_eq!(lines, vec![content.to_string()]);
    }

    #[test]
    fn exact_content_wrap_keeps_bv_ids_intact() {
        let content = "前缀BV1xx411c7mD后缀";
        let lines = split_content_to_lines_exact(content, 14.0, 70.0, 70.0);

        assert!(lines.iter().any(|line| line.contains("BV1xx411c7mD")));
        assert_eq!(lines.concat(), content);
    }
}
