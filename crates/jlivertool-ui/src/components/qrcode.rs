//! QR Code rendering component

use crate::theme::Colors;
use gpui::*;
use qrcode::{QrCode, Color as QrColor};

/// QR Code view component
pub struct QrCodeView {
    data: Option<String>,
    matrix: Option<Vec<Vec<bool>>>,
}

impl QrCodeView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            data: None,
            matrix: None,
        }
    }

    /// Set the data to encode as QR code
    pub fn set_data(&mut self, data: String, cx: &mut Context<Self>) {
        if Some(&data) == self.data.as_ref() {
            return;
        }

        self.data = Some(data.clone());

        // Generate QR code matrix
        if let Ok(code) = QrCode::new(data.as_bytes()) {
            let width = code.width();
            let mut matrix = Vec::with_capacity(width);

            for y in 0..width {
                let mut row = Vec::with_capacity(width);
                for x in 0..width {
                    let color = code[(x, y)];
                    row.push(color == QrColor::Dark);
                }
                matrix.push(row);
            }

            self.matrix = Some(matrix);
        } else {
            self.matrix = None;
        }

        cx.notify();
    }

    /// Clear the QR code
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.data = None;
        self.matrix = None;
        cx.notify();
    }
}

impl Render for QrCodeView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let module_size = 4.0; // Size of each QR module in pixels
        let padding = 8.0; // Padding around the QR code

        if let Some(ref matrix) = self.matrix {
            let size = matrix.len() as f32 * module_size + padding * 2.0;

            div()
                .size(px(size))
                .bg(gpui::white())
                .rounded_md()
                .p(px(padding))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_0()
                        .children(matrix.iter().map(|row| {
                            div()
                                .flex()
                                .flex_row()
                                .gap_0()
                                .children(row.iter().map(|&dark| {
                                    div()
                                        .size(px(module_size))
                                        .bg(if dark { gpui::black() } else { gpui::white() })
                                }))
                        })),
                )
                .into_any_element()
        } else {
            // Placeholder when no QR code
            div()
                .size(px(150.0))
                .bg(Colors::bg_secondary())
                .rounded_md()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(Colors::text_muted())
                        .child("生成中..."),
                )
                .into_any_element()
        }
    }
}
