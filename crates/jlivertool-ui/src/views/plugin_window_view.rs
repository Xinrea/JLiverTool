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
use std::time::Duration;

/// Plugin window view state
pub struct PluginWindowView {
    plugin_id: String,
    plugin_name: String,
    plugin_path: PathBuf,
    ws_port: u16,
    webview: Option<Entity<WebView>>,
    webview_initialized: bool,
    window_handle: Option<AnyWindowHandle>,
}

impl PluginWindowView {
    pub fn new(
        plugin_id: String,
        plugin_name: String,
        plugin_path: PathBuf,
        ws_port: u16,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Self {
        // Don't create webview in constructor - defer to first render
        Self {
            plugin_id,
            plugin_name,
            plugin_path,
            ws_port,
            webview: None,
            webview_initialized: false,
            window_handle: None,
        }
    }

    fn create_webview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Build the URL with WebSocket port parameter
        let index_path = self.plugin_path.join("index.html");
        let url = format!("file://{}?ws_port={}", index_path.display(), self.ws_port);

        // Get the window handle for wry
        let window_handle = match window.window_handle() {
            Ok(handle) => handle,
            Err(e) => {
                tracing::error!("Failed to get window handle: {}", e);
                return;
            }
        };

        // Create wry webview
        let wry_webview = wry::WebViewBuilder::new()
            .with_url(&url)
            .with_transparent(true)
            .with_initialization_script(include_str!("plugin_preload.js"))
            .build_as_child(&window_handle);

        match wry_webview {
            Ok(wv) => {
                let webview = cx.new(|cx| WebView::new(wv, window, cx));
                self.webview = Some(webview);
                cx.notify();
            }
            Err(e) => {
                tracing::error!("Failed to create webview: {}", e);
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
        // Store window handle for deferred webview creation
        if self.window_handle.is_none() {
            self.window_handle = Some(cx.window_handle());
        }

        // Defer webview creation to avoid reentrancy issues on Windows
        // The webview2 creation pumps the message loop which can cause RefCell borrow conflicts
        if !self.webview_initialized {
            self.webview_initialized = true;
            if let Some(window_handle) = self.window_handle {
                cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
                    // Small delay to ensure render is complete and borrow is released
                    Timer::after(Duration::from_millis(10)).await;
                    let _ = cx.update_window(window_handle, |_, window, cx| {
                        let _ = this.update(cx, |view, cx| {
                            view.create_webview(window, cx);
                        });
                    });
                })
                .detach();
            }
        }

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
