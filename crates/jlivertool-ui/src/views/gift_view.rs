//! Gift window view

use crate::theme::Colors;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::h_flex;
use gpui_component::scroll::Scrollbar;
use gpui_component::v_flex;
use gpui_component::Sizable;
use jlivertool_core::database::Database;
use jlivertool_core::messages::{GiftMessage, GuardMessage};
use jlivertool_core::types::guard_level_name;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

const MAX_GIFT_COUNT: usize = 500;

#[derive(Debug, Clone)]
pub enum GiftEntry {
    Gift(GiftMessage),
    Guard(GuardMessage),
}

impl GiftEntry {
    fn timestamp(&self) -> i64 {
        match self {
            GiftEntry::Gift(g) => g.timestamp,
            GiftEntry::Guard(g) => g.timestamp,
        }
    }

    fn id(&self) -> &str {
        match self {
            GiftEntry::Gift(g) => &g.id,
            GiftEntry::Guard(g) => &g.id,
        }
    }

    fn archived(&self) -> bool {
        match self {
            GiftEntry::Gift(g) => g.archived,
            GiftEntry::Guard(g) => g.archived,
        }
    }

    fn set_archived(&mut self, archived: bool) {
        match self {
            GiftEntry::Gift(g) => g.archived = archived,
            GiftEntry::Guard(g) => g.archived = archived,
        }
    }

    /// Get the value in CNY (yuan)
    fn value_cny(&self) -> f64 {
        match self {
            GiftEntry::Gift(g) => {
                if g.gift_info.coin_type == "silver" {
                    0.0
                } else {
                    (g.gift_info.price * g.num as u64) as f64 / 1000.0
                }
            }
            GiftEntry::Guard(g) => g.price as f64 / 1000.0,
        }
    }
}

/// Shared state for pending archive toggle
type PendingArchiveToggle = Rc<RefCell<Option<String>>>;

/// Shared state for pending delete
type PendingDelete = Rc<RefCell<Option<String>>>;

pub struct GiftView {
    gift_list: VecDeque<GiftEntry>,
    only_guards: bool,  // Whether to show only guard entries
    show_archived: bool,  // Whether to show archived entries
    min_value: f64,  // Minimum value filter in CNY
    max_value: f64,  // Maximum value filter in CNY (0 = no limit)
    min_value_input: Option<Entity<gpui_component::input::InputState>>,
    max_value_input: Option<Entity<gpui_component::input::InputState>>,
    scroll_handle: UniformListScrollHandle,
    opacity: f32,
    database: Option<Arc<Database>>,
    room_id: Option<u64>,
    pending_archive_toggle: PendingArchiveToggle,
    pending_delete: PendingDelete,
    show_clear_confirm: bool,
    // Cached filtered list to avoid O(n) filter on every render
    filtered_cache: Option<Vec<GiftEntry>>,
    // Version counter to track when cache needs invalidation
    list_version: u64,
    cached_list_version: u64,
}

impl GiftView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            gift_list: VecDeque::with_capacity(MAX_GIFT_COUNT),
            only_guards: false,
            show_archived: false,
            min_value: 0.0,
            max_value: 0.0,
            min_value_input: None,
            max_value_input: None,
            scroll_handle: UniformListScrollHandle::new(),
            opacity: 1.0,
            database: None,
            room_id: None,
            pending_archive_toggle: Rc::new(RefCell::new(None)),
            pending_delete: Rc::new(RefCell::new(None)),
            show_clear_confirm: false,
            filtered_cache: None,
            list_version: 0,
            cached_list_version: 0,
        }
    }

    /// Invalidate the filtered cache (call when list or filters change)
    fn invalidate_cache(&mut self) {
        self.list_version = self.list_version.wrapping_add(1);
    }

    pub fn add_gift(&mut self, gift: GiftMessage, cx: &mut Context<Self>) {
        // Check for duplicate by ID
        if self.gift_list.iter().any(|e| e.id() == gift.id) {
            return;
        }

        let should_auto_scroll = self.is_at_bottom();

        self.gift_list.push_back(GiftEntry::Gift(gift));
        while self.gift_list.len() > MAX_GIFT_COUNT {
            self.gift_list.pop_front();
        }

        self.invalidate_cache();

        if should_auto_scroll {
            let last_index = self.gift_list.len().saturating_sub(1);
            self.scroll_handle
                .scroll_to_item(last_index, ScrollStrategy::Bottom);
        }

        cx.notify();
    }

    pub fn add_guard(&mut self, guard: GuardMessage, cx: &mut Context<Self>) {
        // Check for duplicate by ID
        if self.gift_list.iter().any(|e| e.id() == guard.id) {
            return;
        }

        let should_auto_scroll = self.is_at_bottom();

        self.gift_list.push_back(GiftEntry::Guard(guard));
        while self.gift_list.len() > MAX_GIFT_COUNT {
            self.gift_list.pop_front();
        }

        self.invalidate_cache();

        if should_auto_scroll {
            let last_index = self.gift_list.len().saturating_sub(1);
            self.scroll_handle
                .scroll_to_item(last_index, ScrollStrategy::Bottom);
        }

        cx.notify();
    }

    /// Filter gift list based on current filters (with caching)
    fn filtered_gift_list(&mut self) -> Vec<GiftEntry> {
        // Return cached result if still valid
        if self.cached_list_version == self.list_version {
            if let Some(ref cached) = self.filtered_cache {
                return cached.clone();
            }
        }

        // Rebuild the filtered list
        let filtered: Vec<GiftEntry> = self.gift_list
            .iter()
            .filter(|entry| {
                // Filter out archived entries if not showing them
                if !self.show_archived && entry.archived() {
                    return false;
                }
                // Filter to only guards if enabled
                if self.only_guards && !matches!(entry, GiftEntry::Guard(_)) {
                    return false;
                }
                let value = entry.value_cny();
                let min_ok = value >= self.min_value;
                let max_ok = self.max_value <= 0.0 || value <= self.max_value;
                min_ok && max_ok
            })
            .cloned()
            .collect();

        // Update cache
        self.filtered_cache = Some(filtered.clone());
        self.cached_list_version = self.list_version;

        filtered
    }

    pub fn clear_all(&mut self, cx: &mut Context<Self>) {
        // Delete from database
        if let (Some(db), Some(room_id)) = (&self.database, self.room_id) {
            let _ = db.clear_gifts(room_id);
            let _ = db.clear_guards(room_id);
        }
        self.gift_list.clear();
        self.invalidate_cache();
        cx.notify();
    }

    /// Load historical gifts and guards from database
    pub fn load_from_database(
        &mut self,
        db: &Arc<Database>,
        room_id: u64,
        cx: &mut Context<Self>,
    ) {
        self.database = Some(db.clone());
        self.room_id = Some(room_id);
        self.gift_list.clear();

        // Load recent gifts
        if let Ok(gifts) = db.get_recent_gifts(room_id, MAX_GIFT_COUNT / 2) {
            for gift in gifts {
                self.gift_list.push_back(GiftEntry::Gift(gift));
            }
        }

        // Load recent guards
        if let Ok(guards) = db.get_recent_guards(room_id, MAX_GIFT_COUNT / 2) {
            for guard in guards {
                self.gift_list.push_back(GiftEntry::Guard(guard));
            }
        }

        // Sort by archived status first (archived at front), then by timestamp
        let mut sorted: Vec<GiftEntry> = self.gift_list.drain(..).collect();
        sorted.sort_by(|a, b| {
            // Archived items come first (at the top/front of the list)
            match (a.archived(), b.archived()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.timestamp().cmp(&b.timestamp()),
            }
        });
        self.gift_list = sorted.into_iter().collect();

        // Trim to max count
        while self.gift_list.len() > MAX_GIFT_COUNT {
            self.gift_list.pop_front();
        }

        self.invalidate_cache();

        // Scroll to bottom
        if !self.gift_list.is_empty() {
            let last_index = self.gift_list.len().saturating_sub(1);
            self.scroll_handle
                .scroll_to_item(last_index, ScrollStrategy::Bottom);
        }

        cx.notify();
    }

    pub fn set_opacity(&mut self, opacity: f32, cx: &mut Context<Self>) {
        self.opacity = opacity;
        cx.notify();
    }

    /// Toggle archived status for an entry by ID
    pub fn toggle_archived(&mut self, id: &str, cx: &mut Context<Self>) {
        // Find the entry and toggle its archived status
        for entry in self.gift_list.iter_mut() {
            if entry.id() == id {
                let new_archived = !entry.archived();
                entry.set_archived(new_archived);

                // Update in database
                if let Some(db) = &self.database {
                    match entry {
                        GiftEntry::Gift(_) => {
                            let _ = db.set_gift_archived(id, new_archived);
                        }
                        GiftEntry::Guard(_) => {
                            let _ = db.set_guard_archived(id, new_archived);
                        }
                    }
                }
                break;
            }
        }

        // Re-sort the list (archived items at front)
        let mut sorted: Vec<GiftEntry> = self.gift_list.drain(..).collect();
        sorted.sort_by(|a, b| {
            match (a.archived(), b.archived()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.timestamp().cmp(&b.timestamp()),
            }
        });
        self.gift_list = sorted.into_iter().collect();

        self.invalidate_cache();
        cx.notify();
    }

    /// Delete an entry by ID
    pub fn delete_entry(&mut self, id: &str, cx: &mut Context<Self>) {
        // Find and remove the entry
        let mut entry_type = None;
        self.gift_list.retain(|entry| {
            if entry.id() == id {
                entry_type = Some(match entry {
                    GiftEntry::Gift(_) => "gift",
                    GiftEntry::Guard(_) => "guard",
                });
                false
            } else {
                true
            }
        });

        // Delete from database
        if let (Some(db), Some(etype)) = (&self.database, entry_type) {
            match etype {
                "gift" => {
                    let _ = db.delete_gift(id);
                }
                "guard" => {
                    let _ = db.delete_guard(id);
                }
                _ => {}
            }
        }

        self.invalidate_cache();
        cx.notify();
    }

    fn is_at_bottom(&self) -> bool {
        if self.gift_list.len() <= 1 {
            return true;
        }

        let scroll_state = self.scroll_handle.0.borrow();
        let base_handle = &scroll_state.base_handle;
        let offset = base_handle.offset();
        let max_offset = base_handle.max_offset();
        let threshold = px(50.0);
        offset.y <= -max_offset.height + threshold
    }

    fn render_header(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let opacity = self.opacity;

        #[cfg(target_os = "macos")]
        let left_padding = px(78.0);
        #[cfg(not(target_os = "macos"))]
        let left_padding = px(12.0);

        // Initialize min value input if not already done
        if self.min_value_input.is_none() {
            let input = cx.new(|cx| {
                gpui_component::input::InputState::new(window, cx)
                    .placeholder("最小")
                    .mask_pattern(gpui_component::input::MaskPattern::number(None))
            });
            self.min_value_input = Some(input);
        }

        // Initialize max value input if not already done
        if self.max_value_input.is_none() {
            let input = cx.new(|cx| {
                gpui_component::input::InputState::new(window, cx)
                    .placeholder("最大")
                    .mask_pattern(gpui_component::input::MaskPattern::number(None))
            });
            self.max_value_input = Some(input);
        }

        let min_input_state = self.min_value_input.as_ref().unwrap().clone();
        let max_input_state = self.max_value_input.as_ref().unwrap().clone();

        // Read current input values and update filters
        let min_text = min_input_state.read(cx).text().to_string();
        let max_text = max_input_state.read(cx).text().to_string();

        // Parse min value
        let new_min = if min_text.is_empty() {
            0.0
        } else {
            min_text.parse::<f64>().unwrap_or(0.0)
        };

        // Parse max value (0 means no limit)
        let new_max = if max_text.is_empty() {
            0.0
        } else {
            max_text.parse::<f64>().unwrap_or(0.0)
        };

        // Update filter values if changed
        if (new_min - self.min_value).abs() > 0.001 {
            self.min_value = new_min;
            self.invalidate_cache();
        }
        if (new_max - self.max_value).abs() > 0.001 {
            self.max_value = new_max;
            self.invalidate_cache();
        }

        h_flex()
            .w_full()
            .h(px(32.0))
            .pl(left_padding)
            .pr_2()
            .items_center()
            .justify_between()
            .bg(Colors::bg_secondary_with_opacity(opacity))
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(Colors::text_primary())
                            .child("礼物记录"),
                    )
                    // Value filter inputs (hidden when only_guards is enabled)
                    .when(!self.only_guards, |this| {
                        this.child(
                            h_flex()
                                .gap_1()
                                .items_center()
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(Colors::text_muted())
                                        .child("¥"),
                                )
                                .child(
                                    div()
                                        .w(px(70.0))
                                        .child(
                                            gpui_component::input::Input::new(&min_input_state)
                                                .small()
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(Colors::text_muted())
                                        .child("-"),
                                )
                                .child(
                                    div()
                                        .w(px(70.0))
                                        .child(
                                            gpui_component::input::Input::new(&max_input_state)
                                                .small()
                                        ),
                                ),
                        )
                    }),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    // Only guards toggle
                    .child(
                        h_flex()
                            .gap_1()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(10.0))
                                    .text_color(Colors::text_muted())
                                    .child("仅舰长"),
                            )
                            .child(
                                gpui_component::switch::Switch::new("only-guards-toggle")
                                    .checked(self.only_guards)
                                    .on_click(cx.listener(|this, checked: &bool, _window, cx| {
                                        this.only_guards = *checked;
                                        this.invalidate_cache();
                                        cx.notify();
                                    })),
                            ),
                    )
                    // Show archived toggle
                    .child(
                        h_flex()
                            .gap_1()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(10.0))
                                    .text_color(Colors::text_muted())
                                    .child("已归档"),
                            )
                            .child(
                                gpui_component::switch::Switch::new("show-archived-toggle")
                                    .checked(self.show_archived)
                                    .on_click(cx.listener(|this, checked: &bool, _window, cx| {
                                        this.show_archived = *checked;
                                        this.invalidate_cache();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .id("clear-btn")
                            .px_2()
                            .py_1()
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_size(px(11.0))
                            .text_color(Colors::text_muted())
                            .hover(|s| s.bg(Colors::bg_hover()))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_clear_confirm = true;
                                cx.notify();
                            }))
                            .child("清空"),
                    ),
            )
    }

    fn render_gift_list(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let gift_list: Vec<GiftEntry> = self.filtered_gift_list();
        let item_count = gift_list.len();
        let scroll_handle = self.scroll_handle.clone();
        let pending_archive_toggle = self.pending_archive_toggle.clone();
        let pending_delete = self.pending_delete.clone();

        h_flex()
            .id("gift-container")
            .flex_1()
            .w_full()
            .overflow_hidden()
            .child(
                uniform_list("gift-list", item_count, {
                    let gift_list = gift_list.clone();
                    move |range, _window, cx| {
                        range
                            .map(|ix| {
                                let entry = gift_list[ix].clone();
                                let pending = pending_archive_toggle.clone();
                                let pending_del = pending_delete.clone();
                                cx.new(|_| GiftItemView::new(entry, ix, pending, pending_del))
                            })
                            .collect()
                    }
                })
                .flex_1()
                .h_full()
                .track_scroll(scroll_handle.clone()),
            )
            .child(Scrollbar::vertical(&scroll_handle))
    }
}

impl Render for GiftView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Process pending archive toggle
        let pending_id = self.pending_archive_toggle.borrow_mut().take();
        if let Some(id) = pending_id {
            self.toggle_archived(&id, cx);
        }

        // Process pending delete
        let pending_del_id = self.pending_delete.borrow_mut().take();
        if let Some(id) = pending_del_id {
            self.delete_entry(&id, cx);
        }

        let opacity = self.opacity;
        let show_confirm = self.show_clear_confirm;

        v_flex()
            .size_full()
            .bg(Colors::bg_primary_with_opacity(opacity))
            .text_color(Colors::text_primary())
            .child(self.render_header(window, cx))
            .child(self.render_gift_list(window, cx))
            // Confirmation dialog overlay
            .when(show_confirm, |this| {
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(hsla(0.0, 0.0, 0.0, 0.5))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            v_flex()
                                .w(px(280.0))
                                .p_4()
                                .rounded(px(8.0))
                                .bg(Colors::bg_secondary())
                                .border_1()
                                .border_color(Colors::border())
                                .gap_3()
                                .child(
                                    div()
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::BOLD)
                                        .child("确认清空"),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(Colors::text_secondary())
                                        .child("确定要清空所有礼物记录吗？此操作不可撤销。"),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .justify_end()
                                        .child(
                                            div()
                                                .id("cancel-clear-btn")
                                                .px_3()
                                                .py(px(6.0))
                                                .rounded(px(4.0))
                                                .cursor_pointer()
                                                .bg(Colors::bg_hover())
                                                .text_size(px(12.0))
                                                .hover(|s| s.opacity(0.8))
                                                .on_click(cx.listener(|this, _event, _window, cx| {
                                                    this.show_clear_confirm = false;
                                                    cx.notify();
                                                }))
                                                .child("取消"),
                                        )
                                        .child(
                                            div()
                                                .id("confirm-clear-btn")
                                                .px_3()
                                                .py(px(6.0))
                                                .rounded(px(4.0))
                                                .cursor_pointer()
                                                .bg(hsla(0.0, 0.7, 0.5, 1.0))
                                                .text_size(px(12.0))
                                                .text_color(gpui::white())
                                                .hover(|s| s.opacity(0.8))
                                                .on_click(cx.listener(|this, _event, _window, cx| {
                                                    this.show_clear_confirm = false;
                                                    this.clear_all(cx);
                                                }))
                                                .child("确认"),
                                        ),
                                ),
                        ),
                )
            })
    }
}

struct GiftItemView {
    entry: GiftEntry,
    index: usize,
    pending_archive_toggle: PendingArchiveToggle,
    pending_delete: PendingDelete,
}

impl GiftItemView {
    fn new(entry: GiftEntry, index: usize, pending_archive_toggle: PendingArchiveToggle, pending_delete: PendingDelete) -> Self {
        Self { entry, index, pending_archive_toggle, pending_delete }
    }

    fn format_timestamp(timestamp: i64) -> String {
        use chrono::{Local, TimeZone};
        let dt = Local.timestamp_opt(timestamp, 0).unwrap();
        dt.format("%Y/%m/%d %H:%M:%S").to_string()
    }
}

impl Render for GiftItemView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let archived = self.entry.archived();
        let id = self.entry.id().to_string();
        let index = self.index;
        let pending_archive_toggle = self.pending_archive_toggle.clone();
        let pending_delete = self.pending_delete.clone();

        match &self.entry {
            GiftEntry::Gift(gift) => {
                let is_paid = gift.gift_info.coin_type != "silver";
                let price_text = if is_paid {
                    format!("¥{:.2}", gift.gift_info.price as f64 / 1000.0)
                } else {
                    "免费".to_string()
                };

                // Use muted colors for archived items
                let text_color = if archived {
                    Colors::text_muted()
                } else {
                    Colors::text_primary()
                };
                let accent_color = if archived {
                    Colors::text_muted()
                } else if is_paid {
                    Colors::accent()
                } else {
                    Colors::text_secondary()
                };

                v_flex()
                    .w_full()
                    .px_3()
                    .py_2()
                    .gap_1()
                    .border_b_1()
                    .border_color(Colors::bg_hover())
                    .when(archived, |el| el.opacity(0.5))
                    .hover(|s| s.bg(Colors::bg_hover()))
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(text_color)
                                            .child(gift.sender.uname.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .text_color(Colors::text_secondary())
                                            .child(gift.action.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(accent_color)
                                            .child(gift.gift_info.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .text_color(Colors::text_secondary())
                                            .child(format!("x{}", gift.num)),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .text_color(accent_color)
                                            .child(price_text),
                                    )
                                    .child({
                                        let id = id.clone();
                                        let pending = pending_archive_toggle.clone();
                                        div()
                                            .id(SharedString::from(format!("archive-btn-{}", index)))
                                            .px_2()
                                            .py(px(2.0))
                                            .rounded(px(4.0))
                                            .bg(if archived { Colors::accent().opacity(0.2) } else { Colors::bg_hover() })
                                            .cursor_pointer()
                                            .text_size(px(10.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(if archived {
                                                Colors::accent()
                                            } else {
                                                Colors::text_secondary()
                                            })
                                            .hover(|s| s.bg(if archived { Colors::accent().opacity(0.3) } else { Colors::bg_secondary() }))
                                            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                                *pending.borrow_mut() = Some(id.clone());
                                                cx.refresh_windows();
                                            })
                                            .child(if archived { "恢复" } else { "归档" })
                                    })
                                    .child({
                                        let id = id.clone();
                                        let pending = pending_delete.clone();
                                        div()
                                            .id(SharedString::from(format!("delete-btn-{}", index)))
                                            .px_2()
                                            .py(px(2.0))
                                            .rounded(px(4.0))
                                            .bg(Colors::bg_hover())
                                            .cursor_pointer()
                                            .text_size(px(10.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(hsla(0.0, 0.7, 0.5, 1.0))
                                            .hover(|s| s.bg(hsla(0.0, 0.7, 0.5, 0.2)))
                                            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                                *pending.borrow_mut() = Some(id.clone());
                                                cx.refresh_windows();
                                            })
                                            .child("删除")
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(Colors::text_muted())
                            .child(Self::format_timestamp(gift.timestamp)),
                    )
            }
            GiftEntry::Guard(guard) => {
                let guard_name = guard_level_name(guard.guard_level);
                let price_text = format!("¥{:.2}", guard.price as f64 / 1000.0);

                // Use muted colors for archived items
                let text_color = if archived {
                    Colors::text_muted()
                } else {
                    Colors::text_primary()
                };
                let guard_bg = if archived {
                    hsla(0.0, 0.0, 0.5, 0.1)
                } else {
                    hsla(0.15, 0.8, 0.5, 0.1)
                };
                let guard_hover_bg = if archived {
                    hsla(0.0, 0.0, 0.5, 0.15)
                } else {
                    hsla(0.15, 0.8, 0.5, 0.15)
                };
                let guard_badge_bg = if archived {
                    hsla(0.0, 0.0, 0.5, 0.3)
                } else {
                    hsla(0.15, 0.8, 0.5, 0.3)
                };
                let price_color = if archived {
                    Colors::text_muted()
                } else {
                    hsla(0.15, 0.8, 0.6, 1.0)
                };

                v_flex()
                    .w_full()
                    .px_3()
                    .py_2()
                    .gap_1()
                    .border_b_1()
                    .border_color(Colors::bg_hover())
                    .bg(guard_bg)
                    .when(archived, |el| el.opacity(0.5))
                    .hover(|s| s.bg(guard_hover_bg))
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(text_color)
                                            .child(guard.sender.uname.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .text_color(Colors::text_secondary())
                                            .child("开通了"),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py(px(2.0))
                                            .rounded(px(4.0))
                                            .bg(guard_badge_bg)
                                            .text_size(px(12.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(text_color)
                                            .child(guard_name),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .text_color(Colors::text_secondary())
                                            .child(format!("{}{}", guard.num, guard.unit)),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(price_color)
                                            .child(price_text),
                                    )
                                    .child({
                                        let id = id.clone();
                                        let pending = pending_archive_toggle.clone();
                                        div()
                                            .id(SharedString::from(format!("archive-btn-{}", index)))
                                            .px_2()
                                            .py(px(2.0))
                                            .rounded(px(4.0))
                                            .bg(if archived { Colors::accent().opacity(0.2) } else { Colors::bg_hover() })
                                            .cursor_pointer()
                                            .text_size(px(10.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(if archived {
                                                Colors::accent()
                                            } else {
                                                Colors::text_secondary()
                                            })
                                            .hover(|s| s.bg(if archived { Colors::accent().opacity(0.3) } else { Colors::bg_secondary() }))
                                            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                                *pending.borrow_mut() = Some(id.clone());
                                                cx.refresh_windows();
                                            })
                                            .child(if archived { "恢复" } else { "归档" })
                                    })
                                    .child({
                                        let id = id.clone();
                                        let pending = pending_delete.clone();
                                        div()
                                            .id(SharedString::from(format!("delete-btn-{}", index)))
                                            .px_2()
                                            .py(px(2.0))
                                            .rounded(px(4.0))
                                            .bg(Colors::bg_hover())
                                            .cursor_pointer()
                                            .text_size(px(10.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(hsla(0.0, 0.7, 0.5, 1.0))
                                            .hover(|s| s.bg(hsla(0.0, 0.7, 0.5, 0.2)))
                                            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                                *pending.borrow_mut() = Some(id.clone());
                                                cx.refresh_windows();
                                            })
                                            .child("删除")
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(Colors::text_muted())
                            .child(Self::format_timestamp(guard.timestamp)),
                    )
            }
        }
    }
}
