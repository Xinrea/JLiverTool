use crate::theme::Colors;
use gpui::*;
use gpui_component::dock::{Panel, PanelControl, PanelEvent, PanelState, TitleStyle};

use super::dashboard_view::PanelSpec;

pub(super) struct DashboardPanel<V: Render> {
    panel_name: &'static str,
    title: SharedString,
    content: Entity<V>,
    focus_handle: FocusHandle,
}

impl<V: Render + 'static> DashboardPanel<V> {
    pub(super) fn new(spec: PanelSpec, content: Entity<V>, cx: &mut Context<Self>) -> Self {
        Self {
            panel_name: spec.name,
            title: spec.title.into(),
            content,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl<V: Render + 'static> Focusable for DashboardPanel<V> {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<V: Render + 'static> EventEmitter<PanelEvent> for DashboardPanel<V> {}

impl<V: Render + 'static> Panel for DashboardPanel<V> {
    fn panel_name(&self) -> &'static str {
        self.panel_name
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .text_size(px(12.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(Colors::text_secondary())
            .child(self.title.clone())
    }

    fn title_style(&self, _cx: &App) -> Option<TitleStyle> {
        Some(TitleStyle {
            background: Colors::bg_secondary(),
            foreground: Colors::text_secondary(),
        })
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some(self.title.clone())
    }

    fn closable(&self, _cx: &App) -> bool {
        false
    }

    fn zoomable(&self, _cx: &App) -> Option<PanelControl> {
        None
    }

    fn dump(&self, _cx: &App) -> PanelState {
        PanelState::new(self)
    }

    fn inner_padding(&self, _cx: &App) -> bool {
        false
    }
}

impl<V: Render + 'static> Render for DashboardPanel<V> {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .overflow_hidden()
            .child(self.content.clone())
    }
}
