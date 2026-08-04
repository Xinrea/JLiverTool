use crate::theme::Colors;
use crate::views::main_view::{
    DanmuListItemView, DisplayMessage, RenderRow, SelectedUserState, build_render_rows,
};
use gpui::*;
use gpui_component::h_flex;
use gpui_component::scroll::Scrollbar;
use std::collections::VecDeque;
use std::rc::Rc;

pub(crate) struct DashboardDanmuView {
    messages: Rc<Vec<DisplayMessage>>,
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
            messages: Rc::new(Vec::new()),
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

    pub(crate) fn set_messages(
        &mut self,
        messages: &VecDeque<DisplayMessage>,
        cx: &mut Context<Self>,
    ) {
        self.messages = Rc::new(messages.iter().cloned().collect());
        self.rebuild_rows();
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
        if (self.font_size - font_size).abs() <= f32::EPSILON
            && self.lite_mode == lite_mode
            && self.medal_display == medal_display
            && (self.opacity - opacity).abs() <= f32::EPSILON
        {
            return;
        }
        self.font_size = font_size;
        self.lite_mode = lite_mode;
        self.medal_display = medal_display;
        self.opacity = opacity;
        self.rebuild_rows();
        cx.notify();
    }

    fn set_content_width(&mut self, width: f32, cx: &mut Context<Self>) {
        if width <= 0.0 || (self.content_width - width).abs() <= 1.0 {
            return;
        }
        self.content_width = width;
        self.rebuild_rows();
        cx.notify();
    }

    fn rebuild_rows(&mut self) {
        if self.content_width <= 0.0 {
            self.rows = Rc::new(self.messages.iter().cloned().map(RenderRow::Full).collect());
        } else {
            self.rows = Rc::new(build_render_rows(
                self.messages.iter(),
                self.content_width - 14.0,
                self.font_size,
                self.lite_mode,
                self.medal_display,
            ));
        }
        self.pending_scroll_to_bottom = true;
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
