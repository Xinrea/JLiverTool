//! Emoji playback also runs when the overlay window is not focused.

use gpui::*;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

static PLAYBACK_START: LazyLock<Instant> = LazyLock::new(Instant::now);

fn frame_at(delays: impl IntoIterator<Item = Duration>, elapsed: Duration) -> usize {
    let delays: Vec<_> = delays
        .into_iter()
        .map(|delay| delay.max(Duration::from_millis(10)))
        .collect();
    let total: Duration = delays.iter().copied().sum();
    if total.is_zero() {
        return 0;
    }
    let mut position = elapsed.as_nanos() % total.as_nanos();
    for (index, delay) in delays.iter().enumerate() {
        if position < delay.as_nanos() {
            return index;
        }
        position -= delay.as_nanos();
    }
    0
}

pub(super) fn emoji_image(url: String, side: f32) -> impl IntoElement {
    let resource = Resource::Uri(url.into());
    canvas(
        move |_, window, cx| window.use_asset::<ImgResourceLoader>(&resource, cx),
        |bounds, image, window, cx| {
            let Some(Ok(image)) = image else { return };
            paint_emoji(bounds, image, PLAYBACK_START.elapsed(), window, cx);
        },
    )
    .size(px(side))
    .flex_shrink_0()
}

fn paint_emoji(
    bounds: Bounds<Pixels>,
    image: std::sync::Arc<RenderImage>,
    elapsed: Duration,
    window: &mut Window,
    cx: &mut App,
) -> Option<usize> {
    if image.frame_count() == 0 || !bounds.intersects(&window.content_mask().bounds) {
        return None;
    }
    let animate = image.frame_count() > 1 && !cx.reduce_motion();
    let frame = if animate {
        frame_at(
            (0..image.frame_count()).map(|index| Duration::from(image.delay(index))),
            elapsed,
        )
    } else {
        0
    };
    let image_bounds = ObjectFit::Contain.get_bounds(bounds, image.size(frame));
    if let Err(error) = window.paint_image(
        bounds,
        image_bounds,
        Corners::default(),
        image,
        frame,
        false,
    ) {
        tracing::debug!(%error, "Failed to paint danmu emoji");
    }
    if animate {
        window.request_animation_frame();
    }
    Some(frame)
}

#[cfg(test)]
mod tests {
    use super::{frame_at, paint_emoji};
    use core::prelude::v1::test;
    use gpui::{prelude::*, *};
    use std::time::Duration;
    use std::{cell::Cell, rc::Rc, sync::Arc};

    #[test]
    fn playback_keeps_frame_delays_and_loops() {
        let delays = [Duration::from_millis(30), Duration::from_millis(70)];
        for (elapsed, expected) in [(0, 0), (29, 0), (30, 1), (99, 1), (100, 0), (130, 1)] {
            assert_eq!(frame_at(delays, Duration::from_millis(elapsed)), expected);
        }
    }

    #[test]
    fn playback_handles_empty_and_zero_delay_images() {
        assert_eq!(frame_at([], Duration::from_secs(1)), 0);
        assert_eq!(frame_at([Duration::ZERO; 2], Duration::from_millis(10)), 1);
    }

    #[gpui::test]
    fn unfocused_emoji_advances_and_keeps_its_size_in_a_narrow_flex_row(cx: &mut TestAppContext) {
        let window = cx.add_empty_window();
        window.deactivate_window();
        let frame = image::Frame::from_parts(
            image::RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 0, 255])),
            0,
            0,
            image::Delay::from_numer_denom_ms(30, 1),
        );
        let image = Arc::new(RenderImage::new(vec![frame.clone(), frame]));
        let painted = Rc::new(Cell::new(None));
        let observed = painted.clone();
        struct TestEmoji {
            image: Arc<RenderImage>,
            observed: Rc<Cell<Option<usize>>>,
        }
        impl Render for TestEmoji {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let image = self.image.clone();
                let observed = self.observed.clone();
                div()
                    .flex()
                    .w(px(25.))
                    .child(div().w(px(20.)))
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, cx| {
                                assert!(!window.is_window_active());
                                assert_eq!(bounds.size, size(px(20.), px(20.)));
                                observed.set(paint_emoji(
                                    bounds,
                                    image,
                                    Duration::from_millis(30),
                                    window,
                                    cx,
                                ));
                            },
                        )
                        .size(px(20.))
                        .flex_shrink_0(),
                    )
                    .into_any_element()
            }
        }
        window.draw(point(px(0.), px(0.)), size(px(100.), px(30.)), |_, cx| {
            cx.new(|_| TestEmoji { image, observed }).into_any_element()
        });
        assert_eq!(painted.get(), Some(1));
    }
}
