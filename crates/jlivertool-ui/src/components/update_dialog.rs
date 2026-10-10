use crate::app::UiCommand;
use crate::theme::Colors;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{h_flex, v_flex};
use jlivertool_core::update::install::InstallProgress;
use std::sync::mpsc;

/// Shared by the main window and Settings so a manual check's result stays visible.
pub struct UpdateDialog {
    pub visible: bool,
    pub busy: bool,
    version: String,
    release_url: String,
    message: Option<String>,
    error: bool,
    command_tx: mpsc::Sender<UiCommand>,
}

impl UpdateDialog {
    pub fn new(command_tx: mpsc::Sender<UiCommand>) -> Self {
        Self {
            visible: false,
            busy: false,
            version: String::new(),
            release_url: String::new(),
            message: None,
            error: false,
            command_tx,
        }
    }

    pub fn show(&mut self, version: String, release_url: String, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.version = version;
        self.release_url = release_url;
        self.visible = true;
        self.message = None;
        self.error = false;
        cx.notify();
    }

    pub fn progress(&mut self, progress: InstallProgress, cx: &mut Context<Self>) {
        self.busy = true;
        self.message = Some(match progress {
            InstallProgress::Downloading { downloaded, total } => {
                let percent = if total == 0 {
                    0
                } else {
                    downloaded.saturating_mul(100) / total
                };
                format!("正在下载更新… {percent}%")
            }
            InstallProgress::Preparing => "正在校验并准备安装，完成后将自动重启…".into(),
        });
        cx.notify();
    }

    pub fn fail(&mut self, error: String, cx: &mut Context<Self>) {
        self.visible = true;
        self.busy = false;
        self.message = Some(format!("更新失败：{error}"));
        self.error = true;
        cx.notify();
    }
}

impl Render for UpdateDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }
        let busy = self.busy;
        let url = self.release_url.clone();
        div()
            .id("update-dialog-overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(gpui::black().opacity(0.5))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .child(
                v_flex()
                    .w(px(400.0))
                    .max_w_full()
                    .p_4()
                    .rounded(px(8.0))
                    .bg(Colors::bg_secondary())
                    .border_1()
                    .border_color(Colors::border())
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .child("发现新版本"),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(Colors::text_secondary())
                            .child(format!(
                                "新版本 v{} 已发布。下载并更新后，应用将自动重启。",
                                self.version
                            )),
                    )
                    .when_some(self.message.clone(), |this, message| {
                        this.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(if self.error {
                                    Colors::error()
                                } else {
                                    Colors::text_secondary()
                                })
                                .child(message),
                        )
                    })
                    .child(
                        h_flex()
                            .gap_2()
                            .justify_end()
                            .flex_wrap()
                            .child(
                                div()
                                    .id("update-later-btn")
                                    .debug_selector(|| "update-later-btn".into())
                                    .px_3()
                                    .py(px(6.0))
                                    .rounded(px(4.0))
                                    .bg(Colors::bg_hover())
                                    .text_size(px(12.0))
                                    .when(busy, |this| this.opacity(0.5))
                                    .when(!busy, |this| {
                                        this.cursor_pointer().on_click(cx.listener(
                                            |this, _, _, cx| {
                                                this.visible = false;
                                                cx.notify();
                                            },
                                        ))
                                    })
                                    .child("稍后"),
                            )
                            .child(
                                div()
                                    .id("update-now-btn")
                                    .debug_selector(|| "update-now-btn".into())
                                    .px_3()
                                    .py(px(6.0))
                                    .rounded(px(4.0))
                                    .bg(Colors::accent())
                                    .text_size(px(12.0))
                                    .text_color(Colors::button_text())
                                    .when(busy, |this| this.opacity(0.5))
                                    .when(!busy, |this| {
                                        this.cursor_pointer().on_click(cx.listener(
                                            |this, _, _, cx| {
                                                if this
                                                    .command_tx
                                                    .send(UiCommand::DownloadAndInstallUpdate {
                                                        version: this.version.clone(),
                                                    })
                                                    .is_err()
                                                {
                                                    this.fail(
                                                        "无法连接更新服务，请重启应用后重试".into(),
                                                        cx,
                                                    );
                                                    return;
                                                }
                                                this.busy = true;
                                                this.error = false;
                                                this.message = Some("正在获取更新安装包…".into());
                                                cx.notify();
                                            },
                                        ))
                                    })
                                    .child("下载并更新"),
                            )
                            .child(
                                div()
                                    .id("update-release-notes-btn")
                                    .debug_selector(|| "update-release-notes-btn".into())
                                    .px_3()
                                    .py(px(6.0))
                                    .rounded(px(4.0))
                                    .bg(Colors::bg_hover())
                                    .text_size(px(12.0))
                                    .cursor_pointer()
                                    .on_click(move |_, _, cx| cx.open_url(&url))
                                    .child("查看更新内容"),
                            ),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateDialog;
    use crate::app::UiCommand;
    use gpui::{Modifiers, TestAppContext};
    use std::sync::mpsc;

    #[gpui::test]
    fn download_click_sends_one_command_and_failure_allows_retry(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let (tx, rx) = mpsc::channel();
        let (dialog, cx) = cx.add_window_view(|_, _| UpdateDialog::new(tx));
        dialog.update(cx, |dialog, cx| {
            dialog.show(
                "9.0.0".into(),
                "https://github.com/Xinrea/JLiverTool/releases/tag/v9.0.0".into(),
                cx,
            )
        });
        cx.run_until_parked();
        assert!(cx.debug_bounds("update-later-btn").is_some());
        assert!(cx.debug_bounds("update-release-notes-btn").is_some());
        let notes = cx.debug_bounds("update-release-notes-btn").unwrap();
        cx.simulate_click(notes.center(), Modifiers::default());
        assert_eq!(
            cx.opened_url().as_deref(),
            Some("https://github.com/Xinrea/JLiverTool/releases/tag/v9.0.0")
        );
        assert!(
            rx.try_recv().is_err(),
            "release notes must only open the URL"
        );
        let button = cx
            .debug_bounds("update-now-btn")
            .expect("download button is visible");
        cx.simulate_click(button.center(), Modifiers::default());
        assert!(
            matches!(rx.try_recv(), Ok(UiCommand::DownloadAndInstallUpdate { version }) if version == "9.0.0")
        );
        cx.simulate_click(button.center(), Modifiers::default());
        assert!(rx.try_recv().is_err(), "duplicate download is disabled");
        dialog.update(cx, |dialog, cx| dialog.fail("network error".into(), cx));
        cx.run_until_parked();
        let button = cx.debug_bounds("update-now-btn").unwrap();
        cx.simulate_click(button.center(), Modifiers::default());
        assert!(matches!(
            rx.try_recv(),
            Ok(UiCommand::DownloadAndInstallUpdate { .. })
        ));
    }
}
