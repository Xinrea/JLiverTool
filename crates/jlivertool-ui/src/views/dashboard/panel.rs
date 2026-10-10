use crate::theme::Colors;
use gpui::*;
use gpui_component::dock::{BasePanel, Panel, PanelControl, PanelEvent, TitleStyle};

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

impl<V: Render + 'static> BasePanel for DashboardPanel<V> {
    fn panel_name(&self) -> &'static str {
        self.panel_name
    }

    fn closable(&self, _cx: &App) -> bool {
        false
    }

    fn zoomable(&self, _cx: &App) -> bool {
        false
    }
}

impl<V: Render + 'static> Panel for DashboardPanel<V> {
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

    fn zoom_control(&self, _cx: &App) -> Option<PanelControl> {
        None
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

#[cfg(test)]
mod tests {
    use super::{DashboardPanel, PanelSpec};
    use gpui::{
        div, AppContext, Context, IntoElement, ParentElement, Render, TestAppContext, Window,
    };
    use gpui_component::dock::{
        register_panel, DockArea, DockAreaState, DockPlacement, DockSkin, PanelHandle,
    };
    use std::sync::Arc;

    struct Content;

    impl Render for Content {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child("panel content")
        }
    }

    #[gpui::test]
    fn restores_legacy_layout_with_styled_nonclosable_panels(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        for spec in [
            PanelSpec {
                name: "dashboard.danmu",
                title: "弹幕",
            },
            PanelSpec {
                name: "dashboard.gift",
                title: "礼物记录",
            },
        ] {
            cx.update(|cx| {
                register_panel(cx, spec.name, move |_, _, cx| {
                    let content = cx.new(|_| Content);
                    Arc::new(PanelHandle::new(
                        cx.new(|cx| DashboardPanel::new(spec, content, cx)),
                    ))
                });
            });
        }

        // The schema used by gpui-component 0.5.0 must still restore titles,
        // panel constraints, the active tab, and saved split proportions.
        let legacy: DockAreaState = serde_json::from_value(serde_json::json!({
            "version": 2,
            "center": {
                "panel_name": "StackPanel",
                "children": [{
                    "panel_name": "TabPanel",
                    "children": [{
                        "panel_name": "dashboard.danmu",
                        "children": [],
                        "info": { "panel": null }
                    }],
                    "info": { "tabs": { "active_index": 0 } }
                }, {
                    "panel_name": "TabPanel",
                    "children": [{
                        "panel_name": "dashboard.gift",
                        "children": [],
                        "info": { "panel": null }
                    }],
                    "info": { "tabs": { "active_index": 0 } }
                }],
                "info": { "stack": { "sizes": [400.0, 1520.0], "axis": 0 } }
            }
        }))
        .unwrap();

        let (area, cx) = cx.add_window_view(|window, cx| {
            DockArea::new("test-dashboard", Some(2), window, cx).with_renderer(DockSkin::new(cx))
        });
        cx.update(|window, cx| {
            area.update(cx, |area, cx| {
                area.load(legacy.clone(), window, cx).unwrap()
            });
        });
        cx.read(|cx| {
            let area = area.read(cx);
            let restored = area.dump(cx);
            assert_eq!(restored, legacy);
            let panels = area
                .layout(DockPlacement::Center)
                .unwrap()
                .panels()
                .collect::<Vec<_>>();
            assert_eq!(panels.len(), 2);
            for (id, title) in panels.into_iter().zip(["弹幕", "礼物记录"]) {
                let panel = area.panel(id).unwrap();
                assert!(!panel.closable(cx));
                assert!(!panel.zoomable(cx));
                let styled =
                    PanelHandle::of(panel).expect("restored panel retains its styled handle");
                assert_eq!(styled.tab_name(cx).as_deref(), Some(title));
            }
        });
    }
}
