use super::{feedback_log::capture_app_log, feedback_store::FeedbackStore};
use crate::theme::Colors;
use futures::{channel::mpsc, StreamExt};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState, Textarea, TextareaState},
    v_flex, Disableable, FocusTrapElement,
};
use jlivertool_core::feedback::{
    FeedbackAttachment, FeedbackClient, FeedbackDraft, FeedbackError, FeedbackReceipt,
    PreparedFeedback,
};
use std::{io::Read, path::PathBuf, sync::Arc, time::Instant};

const MAX_ATTACHMENT_BYTES: u64 = 5 * 1024 * 1024;

enum SubmissionEvent {
    Frozen(FeedbackDraft),
    Uploading,
    Received(FeedbackReceipt, Option<String>),
    Failed(FeedbackError),
    LocalError(String),
}

/// The settings entity owns this dialog, so closing it preserves the current form.
pub struct FeedbackDialog {
    pub visible: bool,
    busy: bool,
    selecting: bool,
    inputs: Option<(Entity<TextareaState>, Entity<InputState>)>,
    input_window: Option<WindowId>,
    previous_focus: Option<FocusHandle>,
    focus_handle: FocusHandle,
    draft: Option<FeedbackDraft>,
    screenshot: Option<FeedbackAttachment>,
    screenshot_preview: Option<Arc<Image>>,
    log_directory: Option<PathBuf>,
    status: Option<String>,
    error: bool,
    receipt: Option<String>,
    retry_until: Option<Instant>,
    store: Option<FeedbackStore>,
    loaded: bool,
    restore_inputs: bool,
    focus_completion: bool,
}

impl FeedbackDialog {
    pub fn new(data_directory: Option<PathBuf>, cx: &mut Context<Self>) -> Self {
        let log_directory = data_directory
            .as_ref()
            .map(|directory| directory.join("logs"));
        let store = data_directory.map(|directory| FeedbackStore::new(directory.join("feedback")));
        let loaded = store.is_none();
        Self {
            visible: false,
            busy: false,
            selecting: false,
            inputs: None,
            input_window: None,
            previous_focus: None,
            focus_handle: cx.focus_handle(),
            draft: None,
            screenshot: None,
            screenshot_preview: None,
            log_directory,
            error: false,
            status: None,
            receipt: None,
            retry_until: None,
            store,
            loaded,
            restore_inputs: false,
            focus_completion: false,
        }
    }

    pub fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.previous_focus = window.focused(cx);
        if self.input_window != Some(window.window_handle().window_id()) {
            let (message, email) = self.inputs.as_ref().map_or_else(
                || {
                    self.draft.as_ref().map_or_else(
                        || (String::new(), String::new()),
                        |draft| (draft.message().to_string(), draft.email().to_string()),
                    )
                },
                |(message, email)| {
                    (
                        message.read(cx).text().to_string(),
                        email.read(cx).text().to_string(),
                    )
                },
            );
            let message = cx.new(|cx| {
                TextareaState::new(window, cx)
                    .placeholder("请描述遇到的问题、操作步骤或改进建议…")
                    .default_value(message)
            });
            let email = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("选填，便于我们联系你")
                    .default_value(email)
            });
            cx.subscribe(&message, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            })
            .detach();
            self.inputs = Some((message, email));
            self.input_window = Some(window.window_handle().window_id());
        }
        self.visible = true;
        if !self.loaded {
            self.loaded = true;
            self.selecting = true;
            self.status = Some("正在读取反馈草稿…".into());
            let store = self.store.clone();
            cx.spawn(async move |this, cx| {
                let result = cx
                    .background_executor()
                    .spawn(async move {
                        store
                            .as_ref()
                            .map(FeedbackStore::load_draft)
                            .transpose()
                            .map(Option::flatten)
                    })
                    .await;
                let _ = this.update(cx, |this, cx| {
                    this.selecting = false;
                    match result {
                        Ok(draft) => {
                            this.screenshot =
                                draft.as_ref().and_then(|draft| draft.screenshot().cloned());
                            this.screenshot_preview = this.screenshot.as_ref().map(preview_image);
                            this.restore_inputs = draft.is_some();
                            this.draft = draft;
                            this.status = None;
                        }
                        Err(error) => this.fail(error, cx),
                    }
                    cx.notify();
                });
            })
            .detach();
        }
        if self.receipt.is_some() {
            window.focus(&self.focus_handle, cx);
        } else if let Some((message, _)) = &self.inputs {
            message.update(cx, |state, cx| state.focus(window, cx));
        }
        cx.notify();
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.selecting {
            return;
        }
        self.visible = false;
        if let Some(focus) = self.previous_focus.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }

    fn fail(&mut self, message: String, cx: &mut Context<Self>) {
        self.busy = false;
        self.error = true;
        self.status = Some(message);
        cx.notify();
    }

    fn pick_screenshot(&mut self, cx: &mut Context<Self>) {
        if self.busy || self.selecting {
            return;
        }
        self.selecting = true;
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("选择 PNG / JPEG 截图".into()),
        });
        cx.spawn(async move |this, cx| {
            let selected = match paths.await {
                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                Ok(Ok(None)) => None,
                _ => {
                    let _ = this.update(cx, |this, cx| {
                        this.selecting = false;
                        this.fail("无法打开文件选择器，请重试".into(), cx);
                    });
                    return;
                }
            };
            let result = if let Some(path) = selected {
                Some(
                    cx.background_executor()
                        .spawn(async move { read_screenshot(path) })
                        .await,
                )
            } else {
                None
            };
            let _ = this.update(cx, |this, cx| {
                this.selecting = false;
                match result {
                    Some(Ok(attachment)) => {
                        this.screenshot_preview = Some(preview_image(&attachment));
                        this.screenshot = Some(attachment);
                        this.status = None;
                        this.error = false;
                    }
                    Some(Err(error)) => this.fail(error, cx),
                    None => {}
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        if self.busy
            || self.selecting
            || self.receipt.is_some()
            || self.retry_until.is_some_and(|until| Instant::now() < until)
        {
            return;
        }
        let Some((message, email)) = &self.inputs else {
            return;
        };
        let message = message.read(cx).text().to_string();
        let email = email.read(cx).text().to_string();
        if let Err(error) = FeedbackDraft::validate_input(&message, &email) {
            self.fail(error.user_message(), cx);
            return;
        }
        let existing_draft = self
            .draft
            .as_ref()
            .filter(|draft| {
                draft.message() == message.trim()
                    && draft.email() == email.trim()
                    && same_attachment(draft.screenshot(), self.screenshot.as_ref())
            })
            .cloned();
        let screenshot = self.screenshot.clone();
        let log_directory = self.log_directory.clone();
        self.busy = true;
        self.error = false;
        self.status = Some("正在准备提交…".into());
        let store = self.store.clone();
        let (tx, mut rx) = mpsc::unbounded();
        // Reqwest runs on its own Tokio thread, independently of GPUI's executor.
        let spawn = std::thread::Builder::new()
            .name("feedback-submit".into())
            .spawn(move || {
                let draft =
                    match freeze_draft(existing_draft, message, email, screenshot, log_directory) {
                        Ok(draft) => draft,
                        Err(error) => {
                            let _ = tx.unbounded_send(SubmissionEvent::LocalError(error));
                            return;
                        }
                    };
                let _ = tx.unbounded_send(SubmissionEvent::Frozen(draft.clone()));
                if let Some(store) = &store {
                    if let Err(error) = store.save_draft(&draft) {
                        let _ = tx.unbounded_send(SubmissionEvent::LocalError(error));
                        return;
                    }
                }
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(_) => {
                        let _ = tx.unbounded_send(SubmissionEvent::LocalError(
                            "无法启动反馈提交任务，请重试".into(),
                        ));
                        return;
                    }
                };
                let result = runtime.block_on(async {
                    let client = FeedbackClient::new()?;
                    match client.prepare(&draft).await? {
                        PreparedFeedback::AlreadySubmitted(receipt) => Ok(receipt),
                        PreparedFeedback::Upload(upload) => {
                            let _ = tx.unbounded_send(SubmissionEvent::Uploading);
                            client.upload(&draft, &upload).await
                        }
                    }
                });
                let event = match result {
                    Ok(receipt) => {
                        let storage_error = store
                            .as_ref()
                            .and_then(|store| store.complete(&receipt).err());
                        SubmissionEvent::Received(receipt, storage_error)
                    }
                    Err(error) => SubmissionEvent::Failed(error),
                };
                let _ = tx.unbounded_send(event);
            });
        if spawn.is_err() {
            self.fail("无法启动反馈提交任务，请重试".into(), cx);
            return;
        }
        cx.spawn(async move |this, cx| {
            while let Some(event) = rx.next().await {
                let mut retry_delay = None;
                if this
                    .update(cx, |this, cx| {
                        match event {
                            SubmissionEvent::Frozen(draft) => this.draft = Some(draft),
                            SubmissionEvent::Uploading => {
                                this.status = Some("正在发送反馈…".into())
                            }
                            SubmissionEvent::Received(receipt, storage_error) => {
                                this.busy = false;
                                this.error = false;
                                this.receipt = Some(receipt.id);
                                this.focus_completion = true;
                                this.draft = None;
                                this.status = Some(storage_error.map_or_else(
                                    || "反馈已收到，感谢你的反馈！".into(),
                                    |error| format!("反馈已收到。{error}"),
                                ));
                            }
                            SubmissionEvent::Failed(error) => {
                                retry_delay = error.retry_after_seconds;
                                this.retry_until = retry_delay.and_then(|seconds| {
                                    Instant::now()
                                        .checked_add(std::time::Duration::from_secs(seconds))
                                });
                                this.fail(error.user_message(), cx);
                            }
                            SubmissionEvent::LocalError(error) => this.fail(error, cx),
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }
                if let Some(seconds) = retry_delay {
                    cx.spawn(async move |cx| {
                        cx.background_executor()
                            .timer(std::time::Duration::from_secs(seconds))
                            .await;
                        let _ = this.update(cx, |_, cx| cx.notify());
                    })
                    .detach();
                    return;
                }
            }
            let _ = this.update(cx, |this, cx| {
                if this.busy {
                    this.fail("反馈提交中断，请保留原反馈后重试".into(), cx);
                }
            });
        })
        .detach();
        cx.notify();
    }
}

fn same_attachment(a: Option<&FeedbackAttachment>, b: Option<&FeedbackAttachment>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a.filename() == b.filename() && a.bytes() == b.bytes(),
        _ => false,
    }
}

fn freeze_draft(
    existing: Option<FeedbackDraft>,
    message: String,
    email: String,
    screenshot: Option<FeedbackAttachment>,
    log_directory: Option<PathBuf>,
) -> Result<FeedbackDraft, String> {
    // A retry must reuse the original log bytes even while the app keeps writing.
    if let Some(draft) = existing {
        return Ok(draft);
    }
    let log = log_directory
        .as_deref()
        .map(capture_app_log)
        .transpose()?
        .flatten();
    FeedbackDraft::new(
        message,
        email,
        env!("CARGO_PKG_VERSION"),
        format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        screenshot,
        log,
    )
    .map_err(|error| error.user_message())
}

fn read_screenshot(path: PathBuf) -> Result<FeedbackAttachment, String> {
    let file =
        std::fs::File::open(&path).map_err(|_| "无法读取附件，请检查文件权限".to_string())?;
    if !file
        .metadata()
        .map_err(|_| "无法读取附件信息".to_string())?
        .is_file()
    {
        return Err("请选择普通文件".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_ATTACHMENT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "读取附件失败".to_string())?;
    if bytes.len() as u64 > MAX_ATTACHMENT_BYTES {
        return Err("每个附件不能超过 5 MiB".into());
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("附件文件名无效")?;
    FeedbackAttachment::screenshot(name, bytes).map_err(|error| error.user_message())
}

fn preview_image(attachment: &FeedbackAttachment) -> Arc<Image> {
    let format = if attachment.filename().to_ascii_lowercase().ends_with(".png") {
        ImageFormat::Png
    } else {
        ImageFormat::Jpeg
    };
    Arc::new(Image::from_bytes(format, attachment.bytes().to_vec()))
}

impl Render for FeedbackDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }
        if self.input_window != Some(window.window_handle().window_id()) {
            self.show(window, cx);
        }
        if self.focus_completion {
            self.focus_completion = false;
            window.focus(&self.focus_handle, cx);
        }
        if self.restore_inputs {
            self.restore_inputs = false;
            if let (Some(draft), Some((message, email))) = (&self.draft, &self.inputs) {
                message.update(cx, |state, cx| {
                    state.set_value(draft.message().to_string(), window, cx)
                });
                email.update(cx, |state, cx| {
                    state.set_value(draft.email().to_string(), window, cx)
                });
            }
        }
        let Some((message, email)) = &self.inputs else {
            return div().into_any_element();
        };
        let locked = self.busy || self.selecting;
        let received = self.receipt.is_some();
        let waiting = self.retry_until.is_some_and(|until| Instant::now() < until);
        let chars = message.read(cx).text().to_string().trim().chars().count();
        div()
            .id("feedback-modal-overlay")
            .debug_selector(|| "feedback-modal-overlay".into())
            .absolute().inset_0().flex().items_center().justify_center()
            .bg(gpui::black().opacity(0.5))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .child(
                v_flex().id("feedback-modal").track_focus(&self.focus_handle).w(px(520.0)).max_w_full().max_h_full()
                    .overflow_y_scroll().p_4().rounded(px(8.0)).gap_3()
                    .bg(Colors::bg_secondary()).border_1().border_color(Colors::border())
                    .text_color(Colors::text_primary()).text_size(px(13.0))
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                        if event.keystroke.key == "escape" { this.close(window, cx); cx.stop_propagation(); }
                    }))
                    .child(div().text_size(px(16.0)).font_weight(FontWeight::BOLD).child("问题反馈"))
                    .when(!received, |this| this
                        .child(div().text_size(px(12.0)).text_color(Colors::text_secondary()).child("版本、系统信息和应用日志会随反馈一同发送，不含敏感信息。"))
                        .child(h_flex().justify_between().child("问题描述").child(div().text_size(px(12.0)).text_color(if chars > 5000 { Colors::error() } else { Colors::text_muted() }).child(format!("{chars} / 5000"))))
                        .child(Textarea::new(message).h(px(140.0)).disabled(locked).aria_label("问题描述"))
                        .child("联系邮箱（选填）")
                        .child(Input::new(email).disabled(locked))
                        .child(h_flex().gap_2()
                            .child(Button::new("feedback-screenshot").label("选择截图").disabled(locked)
                                .on_click(cx.listener(|this, _, _, cx| this.pick_screenshot(cx)))))
                        .when_some(self.screenshot.clone(), |this, attachment| this
                            .child(h_flex().gap_2().justify_between()
                                .child(div().flex_1().child(format!("{}（{} KiB）", attachment.filename(), attachment.size().div_ceil(1024))))
                                .child(Button::new("feedback-remove-screenshot").ghost().label("移除").disabled(locked)
                                    .on_click(cx.listener(|this, _, _, cx| { this.screenshot = None; this.screenshot_preview = None; cx.notify(); }))))
                            .when_some(self.screenshot_preview.clone(), |this, image| this.child(img(image).w_full().h(px(100.0)).object_fit(ObjectFit::Contain))))
                        .child(div().text_size(px(11.0)).text_color(Colors::text_muted()).child("截图选填，支持 PNG/JPEG，最大 5 MiB。")))
                    .when_some(self.status.clone(), |this, status| this.child(div().text_size(px(12.0)).text_color(if self.error { Colors::error() } else { Colors::text_secondary() }).child(status)))
                    .when_some(self.receipt.clone(), |this, id| this.child(div().text_size(px(12.0)).child(format!("反馈编号：{id}"))))
                    .child(h_flex().justify_end().gap_2()
                        .child(Button::new("feedback-close").debug_selector(|| "feedback-close".into()).disabled(locked)
                            .on_click(cx.listener(|this, _, window, cx| this.close(window, cx)))
                            .label(if received { "完成" } else { "关闭" }))
                        .when(!received, |this| this.child(Button::new("feedback-submit").debug_selector(|| "feedback-submit".into()).primary().disabled(locked || waiting)
                            .on_click(cx.listener(|this, _, _, cx| this.submit(cx)))
                            .label(if self.selecting { "正在读取附件…" } else if self.busy { "提交中…" } else if waiting { "请稍后重试" } else { "提交反馈" })))
                        .when(received, |this| this.child(Button::new("feedback-new").primary()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.receipt = None; this.status = None; this.screenshot = None; this.screenshot_preview = None;
                                if let Some((message, email)) = &this.inputs {
                                    message.update(cx, |state, cx| { state.set_value("", window, cx); state.focus(window, cx); });
                                    email.update(cx, |state, cx| state.set_value("", window, cx));
                                }
                                cx.notify();
                            })).label("继续反馈"))))
                    .focus_trap("feedback-focus-trap", &self.focus_handle)
            ).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{freeze_draft, FeedbackDialog, FeedbackDraft};
    use gpui::{AppContext, Entity, Modifiers, TestAppContext, VisualTestContext};
    use gpui_component::Root;
    use std::time::{Duration, Instant};

    #[test]
    fn new_feedback_captures_application_log_and_retries_retain_its_snapshot() {
        struct TestDirectory(std::path::PathBuf);
        impl Drop for TestDirectory {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = TestDirectory(std::env::temp_dir().join(format!(
            "jlivertool-feedback-auto-log-{}-{unique}",
            std::process::id(),
        )));
        std::fs::create_dir_all(&directory.0).unwrap();
        let file = directory.0.join("jlivertool.log.2026-10-23");
        std::fs::write(&file, "当前运行日志\n").unwrap();
        let freeze = |existing| {
            freeze_draft(
                existing,
                "测试自动日志".into(),
                "".into(),
                None,
                Some(directory.0.clone()),
            )
            .unwrap()
        };

        let original = freeze(None);
        assert_eq!(original.log().unwrap().bytes(), "当前运行日志\n".as_bytes());
        std::fs::write(&file, "日志已继续写入\n").unwrap();

        let retry = freeze(Some(original.clone()));
        assert_eq!(retry.submission_id(), original.submission_id());
        assert_eq!(
            retry.log().unwrap().bytes(),
            original.log().unwrap().bytes()
        );

        let new_feedback = freeze(None);
        assert_ne!(new_feedback.submission_id(), original.submission_id());
        assert_eq!(
            new_feedback.log().unwrap().bytes(),
            "日志已继续写入\n".as_bytes()
        );
    }

    fn dialog_window(cx: &mut TestAppContext) -> (Entity<FeedbackDialog>, &mut VisualTestContext) {
        cx.update(gpui_component::init);
        let mut dialog = None;
        let (_, visual) = cx.add_window_view(|window, cx| {
            let view = cx.new(|cx| FeedbackDialog::new(None, cx));
            dialog = Some(view.clone());
            Root::new(view, window, cx)
        });
        (dialog.unwrap(), visual)
    }

    fn show(dialog: &Entity<FeedbackDialog>, cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            dialog.update(cx, |dialog, cx| dialog.show(window, cx));
        });
        cx.run_until_parked();
    }

    #[gpui::test]
    fn closing_and_reopening_preserves_text_and_restores_focus(cx: &mut TestAppContext) {
        let (dialog, cx) = dialog_window(cx);
        let previous_focus = cx.update(|window, cx| {
            let focus = cx.focus_handle();
            window.focus(&focus, cx);
            focus
        });
        show(&dialog, cx);
        cx.update(|window, cx| {
            dialog.update(cx, |dialog, cx| {
                let (message, email) = dialog.inputs.as_ref().unwrap();
                message.update(cx, |state, cx| {
                    state.set_value("问题描述\n第二行 @测试;", window, cx);
                });
                email.update(cx, |state, cx| {
                    state.set_value("reply@example.com", window, cx);
                });
            });
        });
        cx.run_until_parked();
        let close = cx.debug_bounds("feedback-close").unwrap();
        cx.simulate_click(close.center(), Modifiers::default());
        cx.update(|window, cx| {
            assert!(!dialog.read(cx).visible);
            assert!(previous_focus.is_focused(window));
        });

        show(&dialog, cx);
        cx.read(|cx| {
            let dialog = dialog.read(cx);
            let (message, email) = dialog.inputs.as_ref().unwrap();
            assert_eq!(
                message.read(cx).text().to_string(),
                "问题描述\n第二行 @测试;"
            );
            assert_eq!(email.read(cx).text().to_string(), "reply@example.com");
            assert!(dialog.visible);
        });
        assert!(cx.debug_bounds("feedback-modal-overlay").is_some());
    }

    #[gpui::test]
    fn control_characters_are_rejected_without_a_submission_worker(cx: &mut TestAppContext) {
        let (dialog, cx) = dialog_window(cx);
        show(&dialog, cx);
        cx.update(|window, cx| {
            dialog.update(cx, |dialog, cx| {
                let (message, _) = dialog.inputs.as_ref().unwrap();
                message.update(cx, |state, cx| {
                    state.set_value("请保留这段描述\0", window, cx)
                });
                dialog.submit(cx);
            });
        });
        cx.read(|cx| {
            let dialog = dialog.read(cx);
            assert!(
                !dialog.busy,
                "control characters must not launch a submission worker"
            );
            assert!(dialog.error);
            assert!(dialog.draft.is_none());
            assert_eq!(
                dialog
                    .inputs
                    .as_ref()
                    .unwrap()
                    .0
                    .read(cx)
                    .text()
                    .to_string(),
                "请保留这段描述\0"
            );
        });
    }

    #[gpui::test]
    fn busy_submission_rejects_duplicate_clicks_and_close(cx: &mut TestAppContext) {
        let (dialog, cx) = dialog_window(cx);
        show(&dialog, cx);
        let pending = FeedbackDraft::new("已冻结的反馈", "", "1.0", "test OS", None, None).unwrap();
        cx.update(|_, cx| {
            dialog.update(cx, |dialog, cx| {
                dialog.draft = Some(pending.clone());
                dialog.busy = true;
                dialog.status = Some("正在发送反馈…".into());
                cx.notify();
            });
        });
        cx.run_until_parked();
        let submit = cx.debug_bounds("feedback-submit").unwrap();
        cx.simulate_click(submit.center(), Modifiers::default());
        cx.simulate_click(submit.center(), Modifiers::default());
        cx.update(|window, cx| {
            dialog.update(cx, |dialog, cx| {
                dialog.submit(cx);
                dialog.close(window, cx);
                assert!(dialog.visible);
                assert!(dialog.busy);
                assert!(dialog.draft.as_ref().is_some_and(|draft| draft == &pending));
                assert_eq!(dialog.status.as_deref(), Some("正在发送反馈…"));
            });
        });
    }

    #[gpui::test]
    fn retry_after_blocks_new_attempts_until_the_deadline(cx: &mut TestAppContext) {
        let (dialog, cx) = dialog_window(cx);
        show(&dialog, cx);
        cx.update(|window, cx| {
            dialog.update(cx, |dialog, cx| {
                dialog.inputs.as_ref().unwrap().0.update(cx, |state, cx| {
                    state.set_value("含控制字符\0", window, cx);
                });
                dialog.retry_until = Some(Instant::now() + Duration::from_secs(60));
                dialog.status = Some("请稍后重试".into());
                dialog.submit(cx);
                assert!(!dialog.busy);
                assert!(!dialog.error);
                assert!(dialog.draft.is_none());
                assert_eq!(dialog.status.as_deref(), Some("请稍后重试"));

                dialog.retry_until = Some(Instant::now() - Duration::from_secs(1));
                dialog.submit(cx);
                assert!(
                    dialog.error,
                    "the elapsed deadline permits local form validation"
                );
                assert!(
                    !dialog.busy,
                    "the invalid form still must not contact the server"
                );
            });
        });
    }

    #[gpui::test]
    fn received_feedback_shows_completion_and_cannot_be_resubmitted(cx: &mut TestAppContext) {
        let (dialog, cx) = dialog_window(cx);
        show(&dialog, cx);
        let receipt = "a91c047b-78aa-4029-9e0e-bf8bca87e153";
        cx.update(|_, cx| {
            dialog.update(cx, |dialog, cx| {
                dialog.receipt = Some(receipt.into());
                dialog.status = Some("反馈已收到，感谢你的反馈！".into());
                dialog.submit(cx);
                assert!(!dialog.busy);
                assert!(dialog.draft.is_none());
                cx.notify();
            });
        });
        cx.run_until_parked();
        assert!(cx.debug_bounds("feedback-submit").is_none());
        let close = cx.debug_bounds("feedback-close").unwrap();
        cx.simulate_click(close.center(), Modifiers::default());
        show(&dialog, cx);
        cx.read(|cx| assert_eq!(dialog.read(cx).receipt.as_deref(), Some(receipt)));
        assert!(cx.debug_bounds("feedback-close").is_some());
        assert!(cx.debug_bounds("feedback-submit").is_none());
    }
}
