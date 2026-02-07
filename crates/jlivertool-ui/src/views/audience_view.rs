//! Audience window view

use crate::components::{draggable_area, render_window_controls};
use crate::theme::Colors;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::h_flex;
use gpui_component::scroll::Scrollbar;
use gpui_component::v_flex;
use jlivertool_core::bilibili::api::{GuardListItem, OnlineGoldRankItem};

/// Guard icon URLs from Bilibili CDN
const GUARD_ICON_1: &str = "https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard1.png@44w_44h.webp";
const GUARD_ICON_2: &str = "https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard2.png@44w_44h.webp";
const GUARD_ICON_3: &str = "https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard3.png@44w_44h.webp";

/// Get guard icon URL by level
fn guard_icon_url(level: u8) -> Option<&'static str> {
    match level {
        1 => Some(GUARD_ICON_1),
        2 => Some(GUARD_ICON_2),
        3 => Some(GUARD_ICON_3),
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

/// Tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudienceTab {
    Audience,
    Guards,
}

/// Type alias for fetch audience callback
type FetchAudienceCallback = Box<dyn Fn(&mut Window, &mut App) + 'static>;
/// Type alias for fetch guards callback (with page parameter)
type FetchGuardsCallback = Box<dyn Fn(u32, &mut Window, &mut App) + 'static>;

pub struct AudienceView {
    current_tab: AudienceTab,
    audience_list: Vec<OnlineGoldRankItem>,
    guard_list: Vec<GuardListItem>,
    guard_total: u32,
    guard_page: u32,
    scroll_handle_audience: UniformListScrollHandle,
    scroll_handle_guards: UniformListScrollHandle,
    opacity: f32,
    is_loading: bool,
    // Callbacks
    on_fetch_audience: Option<FetchAudienceCallback>,
    on_fetch_guards: Option<FetchGuardsCallback>,
}

impl AudienceView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            current_tab: AudienceTab::Audience,
            audience_list: Vec::new(),
            guard_list: Vec::new(),
            guard_total: 0,
            guard_page: 1,
            scroll_handle_audience: UniformListScrollHandle::new(),
            scroll_handle_guards: UniformListScrollHandle::new(),
            opacity: 1.0,
            is_loading: false,
            on_fetch_audience: None,
            on_fetch_guards: None,
        }
    }

    pub fn set_opacity(&mut self, opacity: f32, cx: &mut Context<Self>) {
        self.opacity = opacity;
        cx.notify();
    }

    pub fn set_audience_list(&mut self, list: Vec<OnlineGoldRankItem>, cx: &mut Context<Self>) {
        self.audience_list = list;
        self.is_loading = false;
        cx.notify();
    }

    pub fn set_guard_list(
        &mut self,
        list: Vec<GuardListItem>,
        total: u32,
        page: u32,
        cx: &mut Context<Self>,
    ) {
        if page == 1 {
            self.guard_list = list;
        } else {
            self.guard_list.extend(list);
        }
        self.guard_total = total;
        self.guard_page = page;
        self.is_loading = false;
        cx.notify();
    }

    pub fn on_fetch_audience<F>(&mut self, callback: F)
    where
        F: Fn(&mut Window, &mut App) + 'static,
    {
        self.on_fetch_audience = Some(Box::new(callback));
    }

    pub fn on_fetch_guards<F>(&mut self, callback: F)
    where
        F: Fn(u32, &mut Window, &mut App) + 'static,
    {
        self.on_fetch_guards = Some(Box::new(callback));
    }

    /// Trigger initial data fetch
    pub fn fetch_data(&mut self, window: &mut Window, cx: &mut App) {
        self.is_loading = true;
        if let Some(ref callback) = self.on_fetch_audience {
            callback(window, cx);
        }
        if let Some(ref callback) = self.on_fetch_guards {
            callback(1, window, cx);
        }
    }

    fn render_header(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let opacity = self.opacity;
        let current_tab = self.current_tab;

        #[cfg(target_os = "macos")]
        let left_padding = px(78.0);
        #[cfg(not(target_os = "macos"))]
        let left_padding = px(12.0);

        let is_maximized = window.is_maximized();

        v_flex()
            .w_full()
            .bg(Colors::bg_secondary_with_opacity(opacity))
            // Title bar
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
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(Colors::text_primary())
                                    .child("观众列表"),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(Colors::text_muted())
                                    .child(format!("舰长: {}", self.guard_total)),
                            ),
                    )
                    .child(render_window_controls(is_maximized)),
            )
            // Tab bar
            .child(
                h_flex()
                    .w_full()
                    .h(px(36.0))
                    .px_3()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .id("tab-audience")
                            .px_4()
                            .py(px(6.0))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_size(px(12.0))
                            .when(current_tab == AudienceTab::Audience, |el| {
                                el.bg(Colors::accent()).text_color(Colors::button_text())
                            })
                            .when(current_tab != AudienceTab::Audience, |el| {
                                el.bg(Colors::bg_hover())
                                    .text_color(Colors::text_secondary())
                                    .hover(|s| s.bg(Colors::bg_secondary()))
                            })
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.current_tab = AudienceTab::Audience;
                                cx.notify();
                            }))
                            .child("高能榜"),
                    )
                    .child(
                        div()
                            .id("tab-guards")
                            .px_4()
                            .py(px(6.0))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_size(px(12.0))
                            .when(current_tab == AudienceTab::Guards, |el| {
                                el.bg(Colors::accent()).text_color(Colors::button_text())
                            })
                            .when(current_tab != AudienceTab::Guards, |el| {
                                el.bg(Colors::bg_hover())
                                    .text_color(Colors::text_secondary())
                                    .hover(|s| s.bg(Colors::bg_secondary()))
                            })
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.current_tab = AudienceTab::Guards;
                                cx.notify();
                            }))
                            .child("舰长"),
                    ),
            )
    }

    fn render_audience_list(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let audience_list = self.audience_list.clone();
        let item_count = audience_list.len();
        let scroll_handle = self.scroll_handle_audience.clone();

        h_flex()
            .id("audience-container")
            .flex_1()
            .w_full()
            .overflow_hidden()
            .child(
                uniform_list("audience-list", item_count, {
                    let audience_list = audience_list.clone();
                    move |range, _window, cx| {
                        range
                            .map(|ix| {
                                let item = audience_list[ix].clone();
                                cx.new(|_| AudienceItemView::new(item, ix))
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

    fn render_guard_list(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let guard_list = self.guard_list.clone();
        let item_count = guard_list.len();
        let scroll_handle = self.scroll_handle_guards.clone();
        let has_more = self.guard_list.len() < self.guard_total as usize;
        let next_page = self.guard_page + 1;
        let is_loading = self.is_loading;

        v_flex()
            .id("guard-container")
            .flex_1()
            .w_full()
            .overflow_hidden()
            .child(
                h_flex()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(
                        uniform_list("guard-list", item_count, {
                            let guard_list = guard_list.clone();
                            move |range, _window, cx| {
                                range
                                    .map(|ix| {
                                        let item = guard_list[ix].clone();
                                        cx.new(|_| GuardItemView::new(item, ix))
                                    })
                                    .collect()
                            }
                        })
                        .flex_1()
                        .h_full()
                        .track_scroll(scroll_handle.clone()),
                    )
                    .child(Scrollbar::vertical(&scroll_handle)),
            )
            // Load more button
            .when(has_more && !is_loading, |el| {
                el.child(
                    div()
                        .id("load-more-btn")
                        .w_full()
                        .h(px(36.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .bg(Colors::bg_hover())
                        .hover(|s| s.bg(Colors::bg_secondary()))
                        .text_size(px(12.0))
                        .text_color(Colors::accent())
                        .on_click(cx.listener(move |this, _event, window, cx| {
                            this.is_loading = true;
                            if let Some(ref callback) = this.on_fetch_guards {
                                callback(next_page, window, cx);
                            }
                            cx.notify();
                        }))
                        .child("加载更多"),
                )
            })
            .when(is_loading, |el| {
                el.child(
                    div()
                        .w_full()
                        .h(px(36.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(12.0))
                        .text_color(Colors::text_muted())
                        .child("加载中..."),
                )
            })
    }
}

impl Render for AudienceView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let opacity = self.opacity;
        let current_tab = self.current_tab;

        v_flex()
            .size_full()
            .bg(Colors::bg_primary_with_opacity(opacity))
            .text_color(Colors::text_primary())
            .child(self.render_header(window, cx))
            .when(current_tab == AudienceTab::Audience, |el| {
                el.child(self.render_audience_list(window, cx))
            })
            .when(current_tab == AudienceTab::Guards, |el| {
                el.child(self.render_guard_list(window, cx))
            })
    }
}

/// Audience item view (for online gold rank)
struct AudienceItemView {
    item: OnlineGoldRankItem,
    index: usize,
}

impl AudienceItemView {
    fn new(item: OnlineGoldRankItem, index: usize) -> Self {
        Self { item, index }
    }
}

impl Render for AudienceItemView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let item = &self.item;
        // Check guard_level from medal_info first, then fall back to item.guard_level
        let guard_level = item
            .medal_info
            .as_ref()
            .map(|m| m.guard_level)
            .unwrap_or(item.guard_level);
        let has_guard = (1..=3).contains(&guard_level);

        h_flex()
            .w_full()
            .h(px(48.0))
            .px_3()
            .gap_3()
            .items_center()
            .border_b_1()
            .border_color(Colors::bg_hover())
            .hover(|s| s.bg(Colors::bg_hover()))
            // Rank number
            .child(
                div()
                    .w(px(24.0))
                    .text_size(px(12.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(if self.index < 3 {
                        Colors::warning()
                    } else {
                        Colors::text_muted()
                    })
                    .child(format!("{}", item.rank)),
            )
            // Avatar
            .child(
                div()
                    .size(px(32.0))
                    .rounded_full()
                    .bg(Colors::bg_hover())
                    .overflow_hidden()
                    .flex_shrink_0()
                    .when(!item.face.is_empty(), |el| {
                        el.child(
                            img(item.face.clone())
                                .size(px(32.0))
                                .rounded_full()
                                .object_fit(ObjectFit::Cover),
                        )
                    }),
            )
            // Name and guard badge
            .child(
                h_flex()
                    .flex_1()
                    .gap_2()
                    .items_center()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(Colors::text_primary())
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(item.name.clone()),
                    )
                    .when(has_guard, |el| {
                        if let Some(icon_url) = guard_icon_url(guard_level) {
                            el.child(img(icon_url).size(px(16.0)).object_fit(ObjectFit::Contain))
                        } else {
                            el
                        }
                    }),
            )
            // Score
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(Colors::accent())
                    .child(format!("{}", item.score)),
            )
    }
}

/// Guard item view
struct GuardItemView {
    item: GuardListItem,
    index: usize,
}

impl GuardItemView {
    fn new(item: GuardListItem, index: usize) -> Self {
        Self { item, index }
    }
}

impl Render for GuardItemView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let item = &self.item;
        let guard_name = guard_level_name(item.guard_level);
        let index = self.index;

        // Guard level colors
        let guard_color = match item.guard_level {
            1 => hsla(45.0 / 360.0, 0.9, 0.6, 1.0),  // 总督 - Gold
            2 => hsla(280.0 / 360.0, 0.7, 0.6, 1.0), // 提督 - Purple
            3 => hsla(200.0 / 360.0, 0.8, 0.6, 1.0), // 舰长 - Blue
            _ => Colors::text_muted(),
        };

        h_flex()
            .w_full()
            .h(px(48.0))
            .px_3()
            .gap_3()
            .items_center()
            .border_b_1()
            .border_color(Colors::bg_hover())
            .hover(|s| s.bg(Colors::bg_hover()))
            // Sequence number
            .child(
                div()
                    .w(px(24.0))
                    .text_size(px(12.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(Colors::text_muted())
                    .child(format!("{}", index + 1)),
            )
            // Guard icon
            .child({
                if let Some(icon_url) = guard_icon_url(item.guard_level) {
                    div()
                        .w(px(20.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(img(icon_url).size(px(18.0)).object_fit(ObjectFit::Contain))
                } else {
                    div().w(px(20.0)).flex_shrink_0()
                }
            })
            // Avatar
            .child(
                div()
                    .size(px(32.0))
                    .rounded_full()
                    .bg(Colors::bg_hover())
                    .overflow_hidden()
                    .flex_shrink_0()
                    .when(!item.face.is_empty(), |el| {
                        el.child(
                            img(item.face.clone())
                                .size(px(32.0))
                                .rounded_full()
                                .object_fit(ObjectFit::Cover),
                        )
                    }),
            )
            // Name and medal
            .child(
                v_flex()
                    .flex_1()
                    .gap(px(2.0))
                    .overflow_hidden()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(Colors::text_primary())
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(item.username.clone()),
                    )
                    .when_some(item.medal_info.as_ref(), |el, medal| {
                        if !medal.medal_name.is_empty() {
                            let (medal_bg, medal_border) = Colors::medal_colors(medal.medal_level);
                            el.child(
                                h_flex().child(
                                    h_flex()
                                        .h(px(14.0))
                                        .rounded_sm()
                                        .border_1()
                                        .border_color(medal_border)
                                        .overflow_hidden()
                                        .child(
                                            h_flex()
                                                .h_full()
                                                .px(px(3.0))
                                                .items_center()
                                                .bg(medal_bg)
                                                .child(
                                                    div()
                                                        .text_size(px(9.0))
                                                        .text_color(gpui::white())
                                                        .child(medal.medal_name.clone()),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .h_full()
                                                .px(px(3.0))
                                                .min_w(px(14.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .bg(gpui::white())
                                                .text_size(px(9.0))
                                                .text_color(medal_bg.opacity(1.0))
                                                .child(medal.medal_level.to_string()),
                                        ),
                                ),
                            )
                        } else {
                            el
                        }
                    }),
            )
            // Guard level badge
            .child(
                div()
                    .flex_shrink_0()
                    .px_2()
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(guard_color.opacity(0.2))
                    .text_size(px(10.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(guard_color)
                    .child(guard_name),
            )
    }
}
