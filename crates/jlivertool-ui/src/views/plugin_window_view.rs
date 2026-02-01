//! Plugin window view
//!
//! Displays a plugin in a webview window.

use crate::components::{draggable_area, render_window_controls};
use crate::theme::Colors;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{h_flex, v_flex, webview::WebView, wry};
use raw_window_handle::HasWindowHandle;
use std::path::PathBuf;

/// Plugin window view state
pub struct PluginWindowView {
    plugin_id: String,
    plugin_name: String,
    plugin_path: PathBuf,
    ws_port: u16,
    webview: Option<Entity<WebView>>,
}

impl PluginWindowView {
    pub fn new(
        plugin_id: String,
        plugin_name: String,
        plugin_path: PathBuf,
        ws_port: u16,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Build the URL with WebSocket port parameter
        let index_path = plugin_path.join("index.html");
        let url = format!(
            "file://{}?ws_port={}",
            index_path.display(),
            ws_port
        );

        // Create the webview
        let webview = Self::create_webview(&url, window, cx);

        Self {
            plugin_id,
            plugin_name,
            plugin_path,
            ws_port,
            webview,
        }
    }

    fn create_webview(url: &str, window: &mut Window, cx: &mut Context<Self>) -> Option<Entity<WebView>> {
        // Get the window handle for wry
        let window_handle = match window.window_handle() {
            Ok(handle) => handle,
            Err(e) => {
                tracing::error!("Failed to get window handle: {}", e);
                return None;
            }
        };

        // Create wry webview
        let wry_webview = wry::WebViewBuilder::new()
            .with_url(url)
            .with_transparent(true)
            .with_initialization_script(include_str!("plugin_preload.js"))
            .build_as_child(&window_handle);

        match wry_webview {
            Ok(wv) => {
                let webview = cx.new(|cx| WebView::new(wv, window, cx));
                Some(webview)
            }
            Err(e) => {
                tracing::error!("Failed to create webview: {}", e);
                None
            }
        }
    }

    /// Get the plugin ID
    #[allow(dead_code)]
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    /// Reload the webview
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        if let Some(ref webview) = self.webview {
            webview.update(cx, |wv, _| {
                let index_path = self.plugin_path.join("index.html");
                let url = format!(
                    "file://{}?ws_port={}",
                    index_path.display(),
                    self.ws_port
                );
                wv.load_url(&url);
            });
        }
    }
}

impl Render for PluginWindowView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity().clone();

        // Leave space for traffic light buttons on macOS
        #[cfg(target_os = "macos")]
        let left_padding = px(78.0);
        #[cfg(not(target_os = "macos"))]
        let left_padding = px(12.0);

        let is_maximized = window.is_maximized();

        v_flex()
            .size_full()
            .bg(Colors::bg_primary())
            .child(
                // Title bar
                h_flex()
                    .w_full()
                    .h(px(32.0))
                    .bg(Colors::bg_secondary())
                    .items_center()
                    .child(
                        draggable_area()
                            .flex_1()
                            .h_full()
                            .pl(left_padding)
                            .pr_3()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(Colors::text_primary())
                                    .child(self.plugin_name.clone()),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .id("plugin-reload-btn")
                                            .px_2()
                                            .py_1()
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .text_size(px(11.0))
                                            .text_color(Colors::text_secondary())
                                            .hover(|s| s.bg(Colors::bg_hover()))
                                            .child("刷新")
                                            .on_click(move |_event, _window, cx| {
                                                entity.update(cx, |view, cx| {
                                                    view.reload(cx);
                                                });
                                            }),
                                    ),
                            ),
                    )
                    .child(render_window_controls(is_maximized)),
            )
            .child(
                // Webview content
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .when_some(self.webview.clone(), |this, webview| {
                        this.child(webview)
                    })
                    .when(self.webview.is_none(), |this| {
                        this.child(
                            div()
                                .size_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .text_size(px(14.0))
                                        .text_color(Colors::text_muted())
                                        .child("无法加载插件"),
                                ),
                        )
                    }),
            )
    }
}
