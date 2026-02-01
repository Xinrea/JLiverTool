//! Superchat window view

use crate::components::{draggable_area, render_window_controls};
use crate::theme::Colors;
use crate::views::render_content_with_links;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::h_flex;
use gpui_component::v_flex;
use jlivertool_core::database::Database;
use jlivertool_core::messages::SuperChatMessage;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

const MAX_SC_COUNT: usize = 500;

/// Shared state for pending archive toggle
type PendingArchiveToggle = Rc<RefCell<Option<String>>>;

/// Shared state for pending delete
type PendingDelete = Rc<RefCell<Option<String>>>;

pub struct SuperChatView {
    sc_list: VecDeque<SuperChatMessage>,
    scroll_handle: ScrollHandle,
    opacity: f32,
    database: Option<Arc<Database>>,
    room_id: Option<u64>,
    pending_archive_toggle: PendingArchiveToggle,
    pending_delete: PendingDelete,
    show_clear_confirm: bool,
    show_archived: bool,
    search_query: String,
    search_input: Option<Entity<gpui_component::input::InputState>>,
}

impl SuperChatView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            sc_list: VecDeque::with_capacity(MAX_SC_COUNT),
            scroll_handle: ScrollHandle::new(),
            opacity: 1.0,
            database: None,
            room_id: None,
            pending_archive_toggle: Rc::new(RefCell::new(None)),
            pending_delete: Rc::new(RefCell::new(None)),
            show_clear_confirm: false,
            show_archived: false,
            search_query: String::new(),
            search_input: None,
        }
    }

    pub fn add_superchat(&mut self, sc: SuperChatMessage, cx: &mut Context<Self>) {
        let should_auto_scroll = self.is_at_bottom();

        self.sc_list.push_back(sc);
        while self.sc_list.len() > MAX_SC_COUNT {
            self.sc_list.pop_front();
        }

        if should_auto_scroll {
            self.scroll_handle.scroll_to_bottom();
        }

        cx.notify();
    }

    pub fn clear_all(&mut self, cx: &mut Context<Self>) {
        // Delete from database
        if let (Some(db), Some(room_id)) = (&self.database, self.room_id) {
            let _ = db.clear_superchats(room_id);
        }
        self.sc_list.clear();
        cx.notify();
    }

    /// Load historical superchats from database
    pub fn load_from_database(
        &mut self,
        db: &Arc<Database>,
        room_id: u64,
        cx: &mut Context<Self>,
    ) {
        self.database = Some(db.clone());
        self.room_id = Some(room_id);
        self.sc_list.clear();

        // Load recent superchats
        if let Ok(superchats) = db.get_recent_superchats(room_id, MAX_SC_COUNT) {
            for sc in superchats {
                self.sc_list.push_back(sc);
            }
        }

        // Sort by archived status first (archived at front), then by timestamp
        let mut sorted: Vec<SuperChatMessage> = self.sc_list.drain(..).collect();
        sorted.sort_by(|a, b| {
            match (a.archived, b.archived) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.timestamp.cmp(&b.timestamp),
            }
        });
        self.sc_list = sorted.into_iter().collect();

        // Scroll to bottom
        if !self.sc_list.is_empty() {
            self.scroll_handle.scroll_to_bottom();
        }

        cx.notify();
    }

    pub fn set_opacity(&mut self, opacity: f32, cx: &mut Context<Self>) {
        self.opacity = opacity;
        cx.notify();
    }

    /// Toggle archived status for a superchat by ID
    pub fn toggle_archived(&mut self, id: &str, cx: &mut Context<Self>) {
        // Find the entry and toggle its archived status
        for sc in self.sc_list.iter_mut() {
            if sc.id == id {
                sc.archived = !sc.archived;

                // Update in database
                if let Some(db) = &self.database {
                    let _ = db.set_superchat_archived(id, sc.archived);
                }
                break;
            }
        }

        // Re-sort the list (archived items at front)
        let mut sorted: Vec<SuperChatMessage> = self.sc_list.drain(..).collect();
        sorted.sort_by(|a, b| {
            match (a.archived, b.archived) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.timestamp.cmp(&b.timestamp),
            }
        });
        self.sc_list = sorted.into_iter().collect();

        cx.notify();
    }

    /// Delete a superchat by ID
    pub fn delete_superchat(&mut self, id: &str, cx: &mut Context<Self>) {
        // Remove from list
        self.sc_list.retain(|sc| sc.id != id);

        // Delete from database
        if let Some(db) = &self.database {
            let _ = db.delete_superchat(id);
        }

        cx.notify();
    }

    fn is_at_bottom(&self) -> bool {
        if self.sc_list.len() <= 1 {
            return true;
        }

        let offset = self.scroll_handle.offset();
        let max_offset = self.scroll_handle.max_offset();
        let threshold = px(50.0);
        offset.y <= -max_offset.height + threshold
    }

    /// Filter superchats based on search query and archived status
    fn filtered_sc_list(&self) -> Vec<SuperChatMessage> {
        self.sc_list
            .iter()
            .filter(|sc| {
                // Filter out archived entries if not showing them
                if !self.show_archived && sc.archived {
                    return false;
                }
                // Filter by search query
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    return sc.sender.uname.to_lowercase().contains(&query)
                        || sc.message.to_lowercase().contains(&query);
                }
                true
            })
            .cloned()
            .collect()
    }

    fn render_header(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let opacity = self.opacity;

        #[cfg(target_os = "macos")]
        let left_padding = px(78.0);
        #[cfg(not(target_os = "macos"))]
        let left_padding = px(12.0);

        // Initialize search input if not already done
        if self.search_input.is_none() {
            let input = cx.new(|cx| {
                gpui_component::input::InputState::new(window, cx)
                    .placeholder("搜索用户名或内容...")
            });
            self.search_input = Some(input);
        }

        let input_state = self.search_input.as_ref().unwrap().clone();

        // Read current search text and update if changed
        let current_text = input_state.read(cx).text().to_string();
        if current_text != self.search_query {
            self.search_query = current_text;
        }

        let is_maximized = window.is_maximized();

        v_flex()
            .w_full()
            .bg(Colors::bg_secondary_with_opacity(opacity))
            // Title bar - only title, count and window controls
            .child(
                h_flex()
                    .w_full()
                    .h(px(32.0))
                    .items_center()
                    .child(
                        draggable_area()
                            .flex_1()
                            .h_full()
                            .pl(left_padding)
                            .pr_2()
                            .flex()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(Colors::text_primary())
                                            .child("醒目留言"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(Colors::text_muted())
                                            .child(format!("({})", self.sc_list.len())),
                                    ),
                            ),
                    )
                    .child(render_window_controls(is_maximized)),
            )
            // Filter controls - in content area
            .child(
                h_flex()
                    .w_full()
                    .h(px(36.0))
                    .px_3()
                    .py_1()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex_1()
                            .child(
                                gpui_component::input::Input::new(&input_state)
                                    .cleanable(true),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .pl_2()
                            .items_center()
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
                                                cx.notify();
                                            })),
                                    ),
                            )
                            // Clear button
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
                    ),
            )
    }

    fn render_sc_list(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sc_list: Vec<SuperChatMessage> = self.filtered_sc_list();
        let scroll_handle = self.scroll_handle.clone();
        let pending_archive_toggle = self.pending_archive_toggle.clone();
        let pending_delete = self.pending_delete.clone();
        let is_filtered = !self.search_query.is_empty();
        let filtered_count = sc_list.len();
        let opacity = self.opacity;

        div()
            .id("sc-container")
            .flex_1()
            .w_full()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&scroll_handle)
            .child(
                v_flex()
                    .id("sc-list")
                    .w_full()
                    .p_1()
                    .gap(px(2.0))
                    // Show filter result count
                    .when(is_filtered, |this| {
                        this.child(
                            div()
                                .w_full()
                                .px_2()
                                .py_1()
                                .text_size(px(11.0))
                                .text_color(Colors::text_muted())
                                .child(format!("找到 {} 条结果", filtered_count)),
                        )
                    })
                    .children(
                        sc_list.iter().enumerate().map(|(ix, sc)| {
                            let pending = pending_archive_toggle.clone();
                            let pending_del = pending_delete.clone();
                            cx.new(|_| SuperChatItemView::new(sc.clone(), ix, pending, pending_del, opacity))
                        })
                    ),
            )
    }
}

impl Render for SuperChatView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Process pending archive toggle
        let pending_id = self.pending_archive_toggle.borrow_mut().take();
        if let Some(id) = pending_id {
            self.toggle_archived(&id, cx);
        }

        // Process pending delete
        let pending_del_id = self.pending_delete.borrow_mut().take();
        if let Some(id) = pending_del_id {
            self.delete_superchat(&id, cx);
        }

        let opacity = self.opacity;
        let show_confirm = self.show_clear_confirm;

        v_flex()
            .size_full()
            .bg(Colors::bg_primary_with_opacity(opacity))
            .text_color(Colors::text_primary())
            .child(self.render_header(window, cx))
            .child(self.render_sc_list(window, cx))
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
                                        .child("确定要清空所有SC记录吗？此操作不可撤销。"),
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

struct SuperChatItemView {
    sc: SuperChatMessage,
    index: usize,
    pending_archive_toggle: PendingArchiveToggle,
    pending_delete: PendingDelete,
    opacity: f32,
}

impl SuperChatItemView {
    fn new(
        sc: SuperChatMessage,
        index: usize,
        pending_archive_toggle: PendingArchiveToggle,
        pending_delete: PendingDelete,
        opacity: f32,
    ) -> Self {
        Self {
            sc,
            index,
            pending_archive_toggle,
            pending_delete,
            opacity,
        }
    }

    fn format_timestamp(timestamp: i64) -> String {
        use chrono::{Local, TimeZone};
        let dt = Local.timestamp_opt(timestamp, 0).unwrap();
        dt.format("%m/%d %H:%M").to_string()
    }
}

impl Render for SuperChatItemView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let archived = self.sc.archived;
        let id = self.sc.id.clone();
        let index = self.index;
        let pending_archive_toggle = self.pending_archive_toggle.clone();
        let pending_delete = self.pending_delete.clone();
        let opacity = self.opacity;

        // Use SC theme color
        let sc_color = Colors::superchat();
        let accent = if archived { Colors::text_muted() } else { sc_color };
        let text_primary = if archived { Colors::text_muted() } else { Colors::text_primary() };
        let text_secondary = if archived { Colors::text_muted() } else { Colors::text_secondary() };

        // Simplified single-row layout
        h_flex()
            .w_full()
            .px_2()
            .py(px(6.0))
            .gap_2()
            .items_start()
            .rounded(px(4.0))
            .bg(Colors::bg_secondary_with_opacity(opacity))
            .border_l_2()
            .border_color(accent)
            .hover(|s| s.bg(Colors::bg_hover_with_opacity(opacity)))
            .when(archived, |el| el.opacity(0.6))
            // Price badge (compact)
            .child(
                div()
                    .flex_shrink_0()
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(8.0))
                    .bg(accent)
                    .text_size(px(11.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(gpui::white())
                    .child(format!("¥{}", self.sc.price)),
            )
            // Content area
            .child(
                v_flex()
                    .flex_1()
                    .gap(px(2.0))
                    .overflow_hidden()
                    // Username and timestamp row
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(text_primary)
                                    .child(self.sc.sender.uname.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(10.0))
                                    .text_color(text_secondary)
                                    .child(Self::format_timestamp(self.sc.timestamp)),
                            ),
                    )
                    // Message with BV link support
                    .child(
                        render_content_with_links(&self.sc.message, 12.0, text_primary, index),
                    ),
            )
            // Action buttons (compact)
            .child(
                h_flex()
                    .flex_shrink_0()
                    .gap_1()
                    .child({
                        let id = id.clone();
                        let pending = pending_archive_toggle.clone();
                        div()
                            .id(SharedString::from(format!("archive-sc-btn-{}", index)))
                            .px(px(6.0))
                            .py(px(2.0))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .bg(Colors::bg_hover_with_opacity(opacity))
                            .text_size(px(10.0))
                            .text_color(Colors::text_muted())
                            .hover(|s| s.text_color(Colors::text_primary()))
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
                            .id(SharedString::from(format!("delete-sc-btn-{}", index)))
                            .px(px(6.0))
                            .py(px(2.0))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_size(px(10.0))
                            .text_color(hsla(0.0, 0.6, 0.5, 0.8))
                            .hover(|s| s.text_color(hsla(0.0, 0.7, 0.5, 1.0)))
                            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                *pending.borrow_mut() = Some(id.clone());
                                cx.refresh_windows();
                            })
                            .child("删除")
                    }),
            )
    }
}
