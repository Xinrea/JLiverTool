//! Window control buttons for Windows platform
//!
//! Provides minimize, maximize/restore, and close buttons that match the macOS style.

#[cfg(target_os = "windows")]
use crate::theme::Colors;
#[cfg(target_os = "windows")]
use gpui::prelude::FluentBuilder;
use gpui::*;
#[cfg(target_os = "windows")]
#[cfg(target_os = "windows")]
use gpui_component::h_flex;

/// Window control button width
#[cfg(target_os = "windows")]
const BUTTON_WIDTH: f32 = 46.0;
/// Window control button height
#[cfg(target_os = "windows")]
const BUTTON_HEIGHT: f32 = 32.0;
/// Icon size
#[cfg(target_os = "windows")]
const ICON_SIZE: f32 = 10.0;

/// Renders window control buttons (minimize, maximize/restore, close)
/// Only rendered on Windows platform.
#[cfg(target_os = "windows")]
pub fn render_window_controls(is_maximized: bool) -> impl IntoElement {
    h_flex()
        .flex_shrink_0()
        .h(px(BUTTON_HEIGHT))
        .child(render_minimize_button())
        .child(render_maximize_button(is_maximized))
        .child(render_close_button())
}

/// Renders window control buttons - no-op on non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn render_window_controls(_is_maximized: bool) -> impl IntoElement {
    div()
}

/// Minimize button with horizontal line icon
#[cfg(target_os = "windows")]
fn render_minimize_button() -> impl IntoElement {
    div()
        .id("window-minimize-btn")
        .w(px(BUTTON_WIDTH))
        .h(px(BUTTON_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(Colors::bg_hover()))
        .window_control_area(WindowControlArea::Min)
        .child(
            // Horizontal line icon
            div()
                .w(px(ICON_SIZE))
                .h(px(1.0))
                .bg(Colors::text_secondary()),
        )
}

/// Maximize/Restore button with square or overlapping squares icon
#[cfg(target_os = "windows")]
fn render_maximize_button(is_maximized: bool) -> impl IntoElement {
    div()
        .id("window-maximize-btn")
        .w(px(BUTTON_WIDTH))
        .h(px(BUTTON_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(Colors::bg_hover()))
        .window_control_area(WindowControlArea::Max)
        .when(!is_maximized, |this| {
            // Single square icon for maximize
            this.child(
                div()
                    .size(px(ICON_SIZE))
                    .border_1()
                    .border_color(Colors::text_secondary()),
            )
        })
        .when(is_maximized, |this| {
            // Overlapping squares icon for restore
            this.child(
                div()
                    .size(px(ICON_SIZE + 2.0))
                    .relative()
                    // Back square (offset up-right)
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .right_0()
                            .size(px(ICON_SIZE - 2.0))
                            .border_1()
                            .border_color(Colors::text_secondary()),
                    )
                    // Front square (offset down-left)
                    .child(
                        div()
                            .absolute()
                            .bottom_0()
                            .left_0()
                            .size(px(ICON_SIZE - 2.0))
                            .border_1()
                            .border_color(Colors::text_secondary())
                            .bg(Colors::bg_secondary()),
                    ),
            )
        })
}

/// Close button with X icon, red hover background
#[cfg(target_os = "windows")]
fn render_close_button() -> impl IntoElement {
    let close_hover_bg = hsla(0.0, 0.7, 0.5, 1.0); // Red

    div()
        .id("window-close-btn")
        .w(px(BUTTON_WIDTH))
        .h(px(BUTTON_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(close_hover_bg))
        .window_control_area(WindowControlArea::Close)
        .child(
            // X icon using two rotated lines
            div()
                .size(px(ICON_SIZE))
                .relative()
                .child(
                    // First diagonal line (top-left to bottom-right)
                    div()
                        .absolute()
                        .top(px(ICON_SIZE / 2.0 - 0.5))
                        .left_0()
                        .w(px(ICON_SIZE))
                        .h(px(1.0))
                        .bg(Colors::text_secondary())
                        .rotate(Rotation::Degrees(45.0)),
                )
                .child(
                    // Second diagonal line (top-right to bottom-left)
                    div()
                        .absolute()
                        .top(px(ICON_SIZE / 2.0 - 0.5))
                        .left_0()
                        .w(px(ICON_SIZE))
                        .h(px(1.0))
                        .bg(Colors::text_secondary())
                        .rotate(Rotation::Degrees(-45.0)),
                ),
        )
}

/// Helper to create a draggable title bar area on Windows
#[cfg(target_os = "windows")]
pub fn draggable_area() -> Div {
    div().window_control_area(WindowControlArea::Drag)
}

/// Helper to create a draggable title bar area - no-op on non-Windows
#[cfg(not(target_os = "windows"))]
pub fn draggable_area() -> Div {
    div()
}
