//! Window wrapper that tracks and persists window bounds

use crate::app::UiCommand;
use gpui::*;
use jlivertool_core::types::WindowType;
use std::sync::mpsc;

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
