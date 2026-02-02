//! Render methods for MainView

use super::{DanmuListItemView, MainView, UserInfoCard};
use crate::app::UiCommand;
use crate::components::draggable_area;
use crate::theme::Colors;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::h_flex;
use gpui_component::scroll::Scrollbar;
use gpui_component::v_flex;
use std::rc::Rc;

impl MainView {
    pub(super) fn render_header(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_live = self.live_status == 1;
        let opacity = self.opacity;

        #[cfg(target_os = "macos")]
        let left_padding = px(78.0);
        #[cfg(not(target_os = "macos"))]
        let left_padding = px(12.0);

        let header_bg = if is_live {
            Colors::live()
        } else {
            Colors::bg_secondary_with_opacity(opacity)
        };

        let header_text_color = if is_live {
            hsla(0.0, 0.0, 1.0, 1.0)
        } else {
            Colors::text_primary()
        };

        h_flex()
            .w_full()
            .h(px(32.0))
            .items_center()
            .bg(header_bg)
            .child(
                draggable_area()
                    .flex_1()
                    .h_full()
                    .pl(left_padding)
                    .flex()
                    .items_center()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .when(is_live, |this| {
                                this.child(
                                    div()
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(header_text_color)
                                        .child("LIVE"),
                                )
                            })
                            .when(is_live, |this| {
                                this.child(
                                    h_flex().gap_1().items_center().child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(header_text_color)
                                            .child(format!("{}", self.online_count)),
                                    ),
                                )
                            }),
                    ),
            )
            .child(
                h_flex()
                    .pr_2()
                    .gap_2()
                    .items_center()
                    .child(self.render_pin_button(is_live, cx))
                    .child(self.render_gift_button(is_live, cx))
                    .child(self.render_superchat_button(is_live, cx))
                    .child(self.render_stats_button(is_live, cx))
                    .child(self.render_audience_button(is_live, cx))
                    .child(self.render_settings_button(is_live, cx)),
            )
    }

    fn render_pin_button(&self, is_live: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let is_pinned = self.always_on_top;
        let icon_color = if is_pinned {
            Colors::accent()
        } else if is_live {
            hsla(0.0, 0.0, 1.0, 0.7)
        } else {
            Colors::text_muted()
        };

        div()
            .id("pin-btn")
            .size(px(24.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .when(is_pinned, |this| this.bg(hsla(0.0, 0.0, 1.0, 0.1)))
            .hover(|s| s.bg(hsla(0.0, 0.0, 1.0, 0.2)))
            .on_click(cx.listener(|this, _event, window, cx| {
                this.always_on_top = !this.always_on_top;
                let always_on_top = this.always_on_top;

                crate::platform::set_window_always_on_top(window, always_on_top);

                if let Some(handle) = &this.settings_window {
                    let _ = cx.update_window(*handle, |_, win, _| {
                        crate::platform::set_window_always_on_top(win, always_on_top);
                    });
                }
                if let Some(handle) = &this.gift_window {
                    let _ = cx.update_window(*handle, |_, win, _| {
                        crate::platform::set_window_always_on_top(win, always_on_top);
                    });
                }
                if let Some(handle) = &this.superchat_window {
                    let _ = cx.update_window(*handle, |_, win, _| {
                        crate::platform::set_window_always_on_top(win, always_on_top);
                    });
                }
                if let Some(handle) = &this.statistics_window {
                    let _ = cx.update_window(*handle, |_, win, _| {
                        crate::platform::set_window_always_on_top(win, always_on_top);
                    });
                }
                if let Some(handle) = &this.audience_window {
                    let _ = cx.update_window(*handle, |_, win, _| {
                        crate::platform::set_window_always_on_top(win, always_on_top);
                    });
                }

                let _ = this
                    .command_tx
                    .send(UiCommand::UpdateAlwaysOnTop(always_on_top));
                cx.notify();
            }))
            .child(
                div()
                    .size(px(14.0))
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left(px(3.0))
                            .size(px(8.0))
                            .rounded_full()
                            .border_1()
                            .border_color(icon_color)
                            .when(is_pinned, |this| this.bg(icon_color)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(8.0))
                            .left(px(6.0))
                            .w(px(2.0))
                            .h(px(6.0))
                            .bg(icon_color),
                    ),
            )
    }

    fn render_gift_button(&self, is_live: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let icon_color = if is_live {
            hsla(0.0, 0.0, 1.0, 0.7)
        } else {
            Colors::text_muted()
        };

        div()
            .id("gift-window-btn")
            .size(px(24.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .hover(|s| s.bg(hsla(0.0, 0.0, 1.0, 0.2)))
            .on_click(cx.listener(|this, _event, window, cx| {
                this.open_gift_window(window, cx);
            }))
            .child(
                div()
                    .size(px(14.0))
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .bottom_0()
                            .left_0()
                            .right_0()
                            .h(px(9.0))
                            .border_1()
                            .border_color(icon_color)
                            .rounded_b(px(2.0)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(2.0))
                            .left(px(-1.0))
                            .right(px(-1.0))
                            .h(px(3.0))
                            .border_1()
                            .border_color(icon_color)
                            .rounded_t(px(1.0)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(5.0))
                            .bottom_0()
                            .left(px(6.0))
                            .w(px(2.0))
                            .bg(icon_color),
                    ),
            )
    }

    fn render_superchat_button(&self, is_live: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let icon_color = if is_live {
            hsla(0.0, 0.0, 1.0, 0.7)
        } else {
            Colors::text_muted()
        };

        div()
            .id("sc-window-btn")
            .size(px(24.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .hover(|s| s.bg(hsla(0.0, 0.0, 1.0, 0.2)))
            .on_click(cx.listener(|this, _event, window, cx| {
                this.open_superchat_window(window, cx);
            }))
            .child(
                div()
                    .size(px(14.0))
                    .border_1()
                    .border_color(icon_color)
                    .rounded(px(3.0))
                    .rounded_bl(px(0.0)),
            )
    }

    fn render_stats_button(&self, is_live: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let icon_color = if is_live {
            hsla(0.0, 0.0, 1.0, 0.7)
        } else {
            Colors::text_muted()
        };

        div()
            .id("stats-window-btn")
            .size(px(24.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .hover(|s| s.bg(hsla(0.0, 0.0, 1.0, 0.2)))
            .on_click(cx.listener(|this, _event, window, cx| {
                this.open_statistics_window(window, cx);
            }))
            .child(
                h_flex()
                    .gap(px(1.0))
                    .items_end()
                    .h(px(12.0))
                    .child(div().w(px(3.0)).h(px(6.0)).bg(icon_color))
                    .child(div().w(px(3.0)).h(px(10.0)).bg(icon_color))
                    .child(div().w(px(3.0)).h(px(8.0)).bg(icon_color)),
            )
    }

    fn render_audience_button(&self, is_live: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let icon_color = if is_live {
            hsla(0.0, 0.0, 1.0, 0.7)
        } else {
            Colors::text_muted()
        };

        div()
            .id("audience-window-btn")
            .size(px(24.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .hover(|s| s.bg(hsla(0.0, 0.0, 1.0, 0.2)))
            .on_click(cx.listener(|this, _event, window, cx| {
                this.open_audience_window(window, cx);
            }))
            .child(
                h_flex()
                    .gap(px(1.0))
                    .items_end()
                    .h(px(12.0))
                    .child(
                        v_flex()
                            .items_center()
                            .child(div().size(px(4.0)).rounded_full().bg(icon_color))
                            .child(
                                div()
                                    .w(px(6.0))
                                    .h(px(4.0))
                                    .rounded_t(px(3.0))
                                    .bg(icon_color),
                            ),
                    )
                    .child(
                        v_flex()
                            .items_center()
                            .child(div().size(px(4.0)).rounded_full().bg(icon_color))
                            .child(
                                div()
                                    .w(px(6.0))
                                    .h(px(4.0))
                                    .rounded_t(px(3.0))
                                    .bg(icon_color),
                            ),
                    ),
            )
    }

    fn render_settings_button(&self, is_live: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let icon_color = if is_live {
            hsla(0.0, 0.0, 1.0, 0.7)
        } else {
            Colors::text_muted()
        };

        div()
            .id("settings-btn")
            .size(px(24.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .hover(|s| s.bg(hsla(0.0, 0.0, 1.0, 0.2)))
            .on_click(cx.listener(|this, _event, window, cx| {
                this.open_settings_window(window, cx);
            }))
            .child(
                v_flex()
                    .gap(px(2.0))
                    .child(div().w(px(12.0)).h(px(2.0)).rounded_sm().bg(icon_color))
                    .child(div().w(px(12.0)).h(px(2.0)).rounded_sm().bg(icon_color))
                    .child(div().w(px(12.0)).h(px(2.0)).rounded_sm().bg(icon_color)),
            )
    }

    pub(super) fn render_footer(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        use super::AVAILABLE_COMMANDS;

        let opacity = self.opacity;
        let command_tx = self.command_tx.clone();
        let room_id = self.room.as_ref().map(|r| r.real_id());

        struct CommandInputWrapper {
            input: Entity<gpui_component::input::InputState>,
        }

        let state = window.use_keyed_state(
            SharedString::from("command-input-state"),
            cx,
            |window, cx| {
                let input = cx.new(|cx| {
                    gpui_component::input::InputState::new(window, cx)
                        .placeholder("发送弹幕或输入命令 (/title, /bye)...")
                });
                CommandInputWrapper { input }
            },
        );

        let input_state = state.read(cx).input.clone();

        if self.input_state.as_ref() != Some(&input_state) {
            let command_tx_enter = self.command_tx.clone();
            let pending_clear = self.pending_input_clear.clone();
            let show_popup = self.show_command_popup.clone();
            let selected_idx = self.selected_command_index.clone();
            let pending_cmd = self.pending_command_insert.clone();
            let subscription = cx.subscribe(
                &input_state,
                move |this, input, event: &gpui_component::input::InputEvent, cx| {
                    match event {
                        gpui_component::input::InputEvent::PressEnter { .. } => {
                            // If popup is showing and user presses enter, select the command
                            if show_popup.get() {
                                let idx = selected_idx.get();
                                if idx < AVAILABLE_COMMANDS.len() {
                                    let (cmd, _) = AVAILABLE_COMMANDS[idx];
                                    // Set the command text with a trailing space for /title
                                    let new_text = if cmd == "/title" {
                                        format!("{} ", cmd)
                                    } else {
                                        cmd.to_string()
                                    };
                                    *pending_cmd.borrow_mut() = Some(new_text);
                                    show_popup.set(false);
                                    selected_idx.set(0);
                                    cx.notify();
                                    return;
                                }
                            }

                            let text = input.read(cx).text().to_string().trim().to_string();
                            if text.is_empty() {
                                return;
                            }

                            if let Some(room_id) = this.room.as_ref().map(|r| r.real_id()) {
                                if let Some(title) = text.strip_prefix("/title ") {
                                    let title = title.trim().to_string();
                                    if !title.is_empty() {
                                        let _ = command_tx_enter
                                            .send(UiCommand::UpdateRoomTitle { room_id, title });
                                    }
                                } else if text == "/bye" {
                                    let _ = command_tx_enter.send(UiCommand::StopLive { room_id });
                                } else {
                                    let _ = command_tx_enter.send(UiCommand::SendDanmu {
                                        room_id,
                                        message: text,
                                    });
                                }
                            }

                            pending_clear.set(true);
                            show_popup.set(false);
                            selected_idx.set(0);
                            cx.notify();
                        }
                        gpui_component::input::InputEvent::Change => {
                            let text = input.read(cx).text().to_string();
                            // Show popup when text starts with "/" but is not a complete command with args
                            let should_show = text.starts_with('/')
                                && !text.contains(' ')
                                && text.len() < 10;
                            show_popup.set(should_show);
                            if !should_show {
                                selected_idx.set(0);
                            }
                            cx.notify();
                        }
                        gpui_component::input::InputEvent::Blur => {
                            // Hide popup when input loses focus
                            show_popup.set(false);
                            selected_idx.set(0);
                            cx.notify();
                        }
                        _ => {}
                    }
                },
            );

            self.input_state = Some(input_state.clone());
            self._input_subscription = Some(subscription);
        }

        // Handle pending input clear
        if self.pending_input_clear.get() {
            input_state.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
            self.pending_input_clear.set(false);
            self.show_command_popup.set(false);
            self.selected_command_index.set(0);
        }

        // Handle pending command insert
        if let Some(cmd) = self.pending_command_insert.borrow_mut().take() {
            input_state.update(cx, |state, cx| {
                state.set_value(&cmd, window, cx);
            });
        }

        let input_state_for_click = input_state.clone();
        let show_popup = self.show_command_popup.get();
        let selected_idx = self.selected_command_index.get();

        // Get current input text to filter commands
        let current_text = input_state.read(cx).text().to_string();
        let filtered_commands: Vec<(usize, &str, &str)> = AVAILABLE_COMMANDS
            .iter()
            .enumerate()
            .filter(|(_, (cmd, _))| {
                current_text.is_empty() || cmd.starts_with(&current_text)
            })
            .map(|(i, (cmd, desc))| (i, *cmd, *desc))
            .collect();

        // Clone state for popup click handlers
        let input_state_for_popup = input_state.clone();
        let show_popup_state = self.show_command_popup.clone();
        let selected_idx_state = self.selected_command_index.clone();

        // Clone state for key handler
        let show_popup_for_key = self.show_command_popup.clone();
        let selected_idx_for_key = self.selected_command_index.clone();
        let pending_cmd_for_key = self.pending_command_insert.clone();
        let filtered_count = filtered_commands.len();

        v_flex()
            .w_full()
            .bg(Colors::bg_secondary_with_opacity(opacity))
            .border_t_1()
            .border_color(Colors::bg_hover())
            .child(
                h_flex()
                    .w_full()
                    .h(px(32.0))
                    .px_3()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(Colors::text_secondary())
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(if self.room_title.is_empty() {
                                "未连接房间".to_string()
                            } else {
                                self.room_title.clone()
                            }),
                    )
                    .child(h_flex().gap_1().items_center().child(
                        div().size(px(6.0)).rounded_full().bg(if self.connected {
                            Colors::success()
                        } else {
                            Colors::error()
                        }),
                    )),
            )
            .child(
                div()
                    .relative()
                    .w_full()
                    .on_key_down(move |event, _window, cx| {
                        if !show_popup_for_key.get() || filtered_count == 0 {
                            return;
                        }

                        match event.keystroke.key.as_str() {
                            "up" => {
                                let current = selected_idx_for_key.get();
                                if current == 0 {
                                    selected_idx_for_key.set(filtered_count - 1);
                                } else {
                                    selected_idx_for_key.set(current - 1);
                                }
                                cx.stop_propagation();
                                cx.refresh_windows();
                            }
                            "down" => {
                                let current = selected_idx_for_key.get();
                                selected_idx_for_key.set((current + 1) % filtered_count);
                                cx.stop_propagation();
                                cx.refresh_windows();
                            }
                            "escape" => {
                                show_popup_for_key.set(false);
                                selected_idx_for_key.set(0);
                                cx.stop_propagation();
                                cx.refresh_windows();
                            }
                            "tab" => {
                                // Tab also selects the current command
                                let idx = selected_idx_for_key.get();
                                if idx < AVAILABLE_COMMANDS.len() {
                                    let (cmd, _) = AVAILABLE_COMMANDS[idx];
                                    let new_text = if cmd == "/title" {
                                        format!("{} ", cmd)
                                    } else {
                                        cmd.to_string()
                                    };
                                    *pending_cmd_for_key.borrow_mut() = Some(new_text);
                                    show_popup_for_key.set(false);
                                    selected_idx_for_key.set(0);
                                    cx.stop_propagation();
                                    cx.refresh_windows();
                                }
                            }
                            _ => {}
                        }
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .h(px(40.0))
                            .px_3()
                            .py_2()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .flex_1()
                                    .child(gpui_component::input::Input::new(&input_state).cleanable(true)),
                            )
                            .child(
                                div()
                                    .id("send-btn")
                                    .px_3()
                                    .py(px(6.0))
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .bg(Colors::accent())
                                    .hover(|s| s.opacity(0.8))
                                    .text_size(px(12.0))
                                    .text_color(Colors::button_text())
                                    .child("发送")
                                    .on_click({
                                        move |_event, window, cx| {
                                            let text = input_state_for_click
                                                .read(cx)
                                                .text()
                                                .to_string()
                                                .trim()
                                                .to_string();
                                            if text.is_empty() {
                                                return;
                                            }

                                            if let Some(room_id) = room_id {
                                                if let Some(title) = text.strip_prefix("/title ") {
                                                    let title = title.trim().to_string();
                                                    if !title.is_empty() {
                                                        let _ =
                                                            command_tx.send(UiCommand::UpdateRoomTitle {
                                                                room_id,
                                                                title,
                                                            });
                                                    }
                                                } else if text == "/bye" {
                                                    let _ =
                                                        command_tx.send(UiCommand::StopLive { room_id });
                                                } else {
                                                    let _ = command_tx.send(UiCommand::SendDanmu {
                                                        room_id,
                                                        message: text,
                                                    });
                                                }
                                            }

                                            input_state_for_click.update(cx, |state, cx| {
                                                state.set_value("", window, cx);
                                            });
                                        }
                                    }),
                            ),
                    )
                    // Command autocomplete popup
                    .when(show_popup && !filtered_commands.is_empty(), |this| {
                        this.child(
                            div()
                                .absolute()
                                .bottom(px(44.0))
                                .left(px(12.0))
                                .right(px(12.0))
                                .bg(Colors::bg_secondary())
                                .border_1()
                                .border_color(Colors::border())
                                .rounded(px(6.0))
                                .shadow_lg()
                                .overflow_hidden()
                                .child(
                                    v_flex()
                                        .w_full()
                                        .py_1()
                                        .children(filtered_commands.iter().enumerate().map(|(display_idx, (_, cmd, desc))| {
                                            let is_selected = display_idx == selected_idx;
                                            let cmd_str = cmd.to_string();
                                            let show_popup_for_item = show_popup_state.clone();
                                            let selected_idx_for_item = selected_idx_state.clone();
                                            let input_for_focus = input_state_for_popup.clone();

                                            div()
                                                .id(SharedString::from(format!("cmd-{}", display_idx)))
                                                .w_full()
                                                .px_3()
                                                .py_2()
                                                .cursor_pointer()
                                                .when(is_selected, |s| s.bg(Colors::bg_hover()))
                                                .hover(|s| s.bg(Colors::bg_hover()))
                                                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                                    let new_text = if cmd_str == "/title" {
                                                        format!("{} ", cmd_str)
                                                    } else {
                                                        cmd_str.clone()
                                                    };
                                                    // Set the value directly since we have window access
                                                    input_for_focus.update(cx, |state, cx| {
                                                        state.set_value(&new_text, window, cx);
                                                    });
                                                    show_popup_for_item.set(false);
                                                    selected_idx_for_item.set(0);
                                                    cx.refresh_windows();
                                                })
                                                .child(
                                                    h_flex()
                                                        .gap_3()
                                                        .items_center()
                                                        .child(
                                                            div()
                                                                .text_size(px(13.0))
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(Colors::accent())
                                                                .child(cmd.to_string()),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_size(px(12.0))
                                                                .text_color(Colors::text_muted())
                                                                .child(desc.to_string()),
                                                        ),
                                                )
                                        })),
                                ),
                        )
                    }),
            )
    }

    pub(super) fn render_danmu_list(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let font_size = self.font_size;
        let lite_mode = self.lite_mode;
        let medal_display = self.medal_display;
        let scroll_handle = self.scroll_handle.clone();
        let selected_user = self.selected_user.clone();

        let danmu_list = Rc::clone(&self.danmu_list_snapshot);
        let item_count = danmu_list.len();

        h_flex()
            .id("danmu-container")
            .flex_1()
            .w_full()
            .overflow_hidden()
            .child(
                uniform_list("danmu-list", item_count, {
                    move |range, _window, _cx| {
                        range
                            .map(|ix| {
                                let msg = danmu_list[ix].clone();
                                let selected_user = selected_user.clone();
                                // Return element directly instead of creating an entity
                                // This allows tooltip state to be preserved
                                DanmuListItemView::new(
                                    msg,
                                    ix,
                                    font_size,
                                    lite_mode,
                                    medal_display,
                                    selected_user,
                                ).render_element()
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
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.process_events(cx);

        if let Some(always_on_top) = self.pending_always_on_top.take() {
            crate::platform::set_window_always_on_top(window, always_on_top);
        }

        let bounds = window.bounds();
        let current_bounds = (
            f32::from(bounds.origin.x) as i32,
            f32::from(bounds.origin.y) as i32,
            f32::from(bounds.size.width) as u32,
            f32::from(bounds.size.height) as u32,
        );
        if self.last_saved_bounds != Some(current_bounds) {
            self.last_saved_bounds = Some(current_bounds);
            let _ = self.command_tx.send(UiCommand::SaveWindowBounds {
                window_type: jlivertool_core::types::WindowType::Main,
                x: current_bounds.0,
                y: current_bounds.1,
                width: current_bounds.2,
                height: current_bounds.3,
            });
        }

        {
            let mut selected = self.selected_user.borrow_mut();
            if let Some(ref mut user) = *selected {
                if user.fetched_info.is_none() && !user.fetch_requested {
                    user.fetch_requested = true;
                    let uid = user.sender.uid;
                    let _ = self.command_tx.send(UiCommand::FetchUserInfo { uid });
                }
            }
        }

        let opacity = self.opacity;
        let selected_user = self.selected_user.borrow().clone();
        let selected_user_state = self.selected_user.clone();

        let danmu_history: Vec<(String, i64)> = if let Some(ref selected) = selected_user {
            if let (Some(db), Some(room)) = (&self.database, &self.room) {
                let uid = selected.sender.uid;
                let room_id = room.real_id();
                db.get_danmus_by_user(room_id, uid, 50).unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        v_flex()
            .size_full()
            .bg(Colors::bg_primary_with_opacity(opacity))
            .text_color(Colors::text_primary())
            .child(self.render_header(window, cx))
            .child(self.render_danmu_list(window, cx))
            .child(self.render_footer(window, cx))
            .when_some(selected_user, |this, selected| {
                let state_for_close = selected_user_state.clone();
                let history = danmu_history.clone();
                this.child(
                    div()
                        .id("user-info-overlay")
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .p_4()
                        .bg(hsla(0.0, 0.0, 0.0, 0.5))
                        .child(
                            div()
                                .relative()
                                .w_full()
                                .max_w(px(300.0))
                                .child(UserInfoCard::render_element(&selected, history))
                                .child(
                                    div()
                                        .id("close-card-btn")
                                        .absolute()
                                        .top(px(-8.0))
                                        .right(px(-8.0))
                                        .size(px(24.0))
                                        .rounded_full()
                                        .cursor_pointer()
                                        .bg(Colors::bg_secondary())
                                        .border_2()
                                        .border_color(Colors::border())
                                        .hover(|s| s.bg(Colors::error()))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_size(px(14.0))
                                        .text_color(Colors::text_secondary())
                                        .child("×")
                                        .on_click(move |_, _, cx| {
                                            *state_for_close.borrow_mut() = None;
                                            cx.refresh_windows();
                                        }),
                                ),
                        ),
                )
            })
    }
}
