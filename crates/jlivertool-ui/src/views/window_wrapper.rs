//! Window wrapper that tracks and persists window bounds

use crate::app::UiCommand;
use crate::components::draggable_area;
use crate::components::window_controls::render_window_controls;
use crate::theme::Colors;
use gpui::*;
use gpui_component::v_flex;
use jlivertool_core::types::WindowType;
use std::sync::mpsc;

pub(crate) trait WindowFrameContent: Render {
    fn window_opacity(&self) -> f32;
}

/// Adds native-window chrome around a reusable content view.
///
/// Dashboard panels render the content entity directly, while standalone
/// windows wrap the same entity in this frame.
pub(crate) struct WindowFrame<V: WindowFrameContent> {
    inner: Entity<V>,
    title: SharedString,
}

impl<V: WindowFrameContent + 'static> WindowFrame<V> {
    pub(crate) fn new(inner: Entity<V>, title: impl Into<SharedString>) -> Self {
        Self {
            inner,
            title: title.into(),
        }
    }
}

impl<V: WindowFrameContent + 'static> Render for WindowFrame<V> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(target_os = "macos")]
        let left_padding = px(78.0);
        #[cfg(not(target_os = "macos"))]
        let left_padding = px(12.0);

        let is_maximized = window.is_maximized();
        let opacity = self.inner.read(cx).window_opacity();

        v_flex()
            .size_full()
            .child(
                div()
                    .h(px(32.0))
                    .w_full()
                    .flex()
                    .items_center()
                    .bg(Colors::bg_secondary_with_opacity(opacity))
                    .child(
                        draggable_area()
                            .flex_1()
                            .h_full()
                            .pl(left_padding)
                            .pr_2()
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(Colors::text_primary())
                                    .child(self.title.clone()),
                            ),
                    )
                    .child(render_window_controls(is_maximized)),
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(self.inner.clone()),
            )
    }
}

/// A wrapper view that tracks window bounds and sends save commands when they change
pub struct WindowBoundsTracker<V: Render> {
    inner: Entity<V>,
    window_type: WindowType,
    command_tx: mpsc::Sender<UiCommand>,
    last_bounds: Option<(i32, i32, u32, u32)>,
}

impl<V: Render + 'static> WindowBoundsTracker<V> {
    pub fn new(
        inner: Entity<V>,
        window_type: WindowType,
        command_tx: mpsc::Sender<UiCommand>,
    ) -> Self {
        Self {
            inner,
            window_type,
            command_tx,
            last_bounds: None,
        }
    }
}

impl<V: Render + 'static> Render for WindowBoundsTracker<V> {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Track and save window bounds if changed
        let bounds = window.bounds();
        let current_bounds = (
            f32::from(bounds.origin.x) as i32,
            f32::from(bounds.origin.y) as i32,
            f32::from(bounds.size.width) as u32,
            f32::from(bounds.size.height) as u32,
        );

        if self.last_bounds != Some(current_bounds) {
            self.last_bounds = Some(current_bounds);
            let _ = self.command_tx.send(UiCommand::SaveWindowBounds {
                window_type: self.window_type,
                x: current_bounds.0,
                y: current_bounds.1,
                width: current_bounds.2,
                height: current_bounds.3,
            });
        }

        self.inner.clone()
    }
}
