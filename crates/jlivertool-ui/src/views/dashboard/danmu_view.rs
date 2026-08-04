use crate::theme::Colors;
use crate::views::main_view::{
    DanmuListItemView, DisplayMessage, RenderRow, SelectedUserState, append_message_rows,
};
use gpui::*;
use gpui_component::h_flex;
use gpui_component::scroll::Scrollbar;
use std::collections::VecDeque;
use std::rc::Rc;

pub(crate) struct DashboardDanmuView {
    messages: VecDeque<DisplayMessage>,
    /// Number of rendered rows produced by each entry in `messages`.
    /// This keeps front eviction incremental even when one message wraps.
    row_counts: VecDeque<usize>,
    rows: Rc<Vec<RenderRow>>,
    scroll_handle: UniformListScrollHandle,
    pending_scroll_to_bottom: bool,
    font_size: f32,
    lite_mode: bool,
    medal_display: bool,
    opacity: f32,
    selected_user: SelectedUserState,
    content_width: f32,
}

impl DashboardDanmuView {
    pub(crate) fn new(selected_user: SelectedUserState) -> Self {
        Self {
            messages: VecDeque::new(),
            row_counts: VecDeque::new(),
            rows: Rc::new(Vec::new()),
            scroll_handle: UniformListScrollHandle::new(),
            pending_scroll_to_bottom: true,
            font_size: 14.0,
            lite_mode: false,
            medal_display: true,
            opacity: 1.0,
            selected_user,
            content_width: 0.0,
        }
    }

    pub(crate) fn replace_messages(
        &mut self,
        messages: &VecDeque<DisplayMessage>,
        cx: &mut Context<Self>,
    ) {
        let should_auto_scroll = self.is_at_bottom();
        self.messages = messages.clone();
        self.rebuild_rows(should_auto_scroll);
        cx.notify();
    }

    pub(crate) fn push_message(
        &mut self,
        message: DisplayMessage,
        removed_from_front: usize,
        cx: &mut Context<Self>,
    ) {
        let should_auto_scroll = self.is_at_bottom();
        let mut rows = Rc::try_unwrap(std::mem::replace(&mut self.rows, Rc::new(Vec::new())))
            .unwrap_or_else(|rows| (*rows).clone());

        let mut rows_to_remove = 0;
        for _ in 0..removed_from_front.min(self.messages.len()) {
            self.messages.pop_front();
            rows_to_remove += self.row_counts.pop_front().unwrap_or(0);
        }
        rows.drain(..rows_to_remove.min(rows.len()));

        let previous_row_count = rows.len();
        if self.content_width <= 0.0 {
            rows.push(RenderRow::Full(message.clone()));
        } else {
            append_message_rows(
                &mut rows,
                &message,
                self.content_width - 14.0,
                self.font_size,
                self.lite_mode,
                self.medal_display,
            );
        }
        self.row_counts.push_back(rows.len() - previous_row_count);
        self.messages.push_back(message);
        self.rows = Rc::new(rows);
        self.pending_scroll_to_bottom = should_auto_scroll;
        cx.notify();
    }

    pub(crate) fn set_style(
        &mut self,
        font_size: f32,
        lite_mode: bool,
        medal_display: bool,
        opacity: f32,
        cx: &mut Context<Self>,
    ) {
        let layout_changed = (self.font_size - font_size).abs() > f32::EPSILON
            || self.lite_mode != lite_mode
            || self.medal_display != medal_display;
        let opacity_changed = (self.opacity - opacity).abs() > f32::EPSILON;
        if !layout_changed && !opacity_changed {
            return;
        }

        let should_auto_scroll = layout_changed && self.is_at_bottom();
        self.font_size = font_size;
        self.lite_mode = lite_mode;
        self.medal_display = medal_display;
        self.opacity = opacity;
        if layout_changed {
            self.rebuild_rows(should_auto_scroll);
        }
        cx.notify();
    }

    fn set_content_width(&mut self, width: f32, cx: &mut Context<Self>) {
        if width <= 0.0 || (self.content_width - width).abs() <= 1.0 {
            return;
        }
        let should_auto_scroll = self.is_at_bottom();
        self.content_width = width;
        self.rebuild_rows(should_auto_scroll);
        cx.notify();
    }

    pub(crate) fn is_at_bottom(&self) -> bool {
        if self.pending_scroll_to_bottom || self.rows.len() <= 1 {
            return true;
        }

        let scroll_state = self.scroll_handle.0.borrow();
        let base_handle = &scroll_state.base_handle;
        let offset = base_handle.offset();
        let max_offset = base_handle.max_offset();
        offset.y <= -max_offset.height + px(50.0)
    }

    fn rebuild_rows(&mut self, should_auto_scroll: bool) {
        let mut rows = Vec::new();
        self.row_counts.clear();
        for message in &self.messages {
            let previous_row_count = rows.len();
            if self.content_width <= 0.0 {
                rows.push(RenderRow::Full(message.clone()));
            } else {
                append_message_rows(
                    &mut rows,
                    message,
                    self.content_width - 14.0,
                    self.font_size,
                    self.lite_mode,
                    self.medal_display,
                );
            }
            self.row_counts.push_back(rows.len() - previous_row_count);
        }
        self.rows = Rc::new(rows);
        self.pending_scroll_to_bottom = should_auto_scroll;
    }
}

impl Render for DashboardDanmuView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.pending_scroll_to_bottom && !self.rows.is_empty() {
            self.pending_scroll_to_bottom = false;
            self.scroll_handle
                .scroll_to_item(self.rows.len().saturating_sub(1), ScrollStrategy::Bottom);
        }

        let rows = Rc::clone(&self.rows);
        let item_count = rows.len();
        let font_size = self.font_size;
        let lite_mode = self.lite_mode;
        let medal_display = self.medal_display;
        let opacity = self.opacity;
        let selected_user = self.selected_user.clone();
        let scroll_handle = self.scroll_handle.clone();
        let view = cx.entity().clone();

        h_flex()
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(Colors::bg_primary_with_opacity(opacity))
            .child(
                canvas(
                    move |bounds, _, cx| {
                        let width = f32::from(bounds.size.width);
                        view.update(cx, |view, cx| view.set_content_width(width, cx));
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .child(
                uniform_list("dashboard-danmu-list", item_count, move |range, _, _| {
                    range
                        .map(|ix| {
                            DanmuListItemView::new(
                                rows[ix].clone(),
                                ix,
                                font_size,
                                lite_mode,
                                medal_display,
                                opacity,
                                selected_user.clone(),
                            )
                            .render_element()
                        })
                        .collect()
                })
                .flex_1()
                .h_full()
                .track_scroll(scroll_handle.clone()),
            )
            .child(Scrollbar::vertical(&scroll_handle))
    }
}
