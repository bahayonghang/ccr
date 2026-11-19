use crate::theme;
use gpui_kit::{Bounds, BoxShadow, Canvas, ContentMask, Corners, Styled, canvas, point, px};

/// Native drop shadows also paint under their source. Clip them before the
/// translucent sidebar and sheet are painted; this canvas has no hitbox.
pub(super) fn outside_frame(shadows: [BoxShadow; 2]) -> Canvas<()> {
    canvas(
        |_, _, _| (),
        move |frame, (), window, _| {
            let scale = window.scale_factor();
            // Match GPUI's fill bounds (nearest pixel, half toward zero).
            let snap = |p: gpui_kit::Pixels| {
                let n = f32::from(p) * scale;
                ((n.abs() - 0.5).ceil().copysign(n)) as i32
            };
            let x = snap(frame.left());
            let y = snap(frame.top());
            let radius = px(theme::SHEET_RADIUS);
            let margin = (theme::SCENE_INSET_MAX * scale).ceil() as i32 + 1;
            exterior_bands(
                snap(frame.right()) - x,
                snap(frame.bottom()) - y,
                theme::SHEET_RADIUS * scale,
                margin,
                |left, top, right, bottom| {
                    // GPUI floors the near and ceils the far mask edges. Place
                    // each strictly within its device pixel so fractional DPI
                    // cannot produce an extra row where adjacent masks meet.
                    let near = |edge| px((edge as f32 + 0.25) / scale);
                    let far = |edge| px((edge as f32 - 0.25) / scale);
                    let bounds = Bounds::from_corners(
                        point(near(x + left), near(y + top)),
                        point(far(x + right), far(y + bottom)),
                    );
                    window.with_content_mask(Some(ContentMask { bounds }), |window| {
                        window.paint_drop_shadows(frame, Corners::all(radius), &shadows);
                    });
                },
            );
        },
    )
    .absolute()
    .top(px(0.))
    .left(px(0.))
    .size_full()
}

/// Partition the device pixels outside a rounded frame into disjoint rectangles.
/// Four cover the straight edges. At the corners, join consecutive pixel rows
/// with the same chord so each clip covers as much of the native shadow as possible.
fn exterior_bands(
    width: i32,
    height: i32,
    radius: f32,
    margin: i32,
    mut band: impl FnMut(i32, i32, i32, i32),
) {
    band(-margin, -margin, width + margin, 0);
    band(-margin, height, width + margin, height + margin);
    band(-margin, 0, 0, height);
    band(width, 0, width + margin, height);

    let radius = radius.min(width.min(height) as f32 / 2.);
    let cut_at = |row: i32| {
        let dy = (radius - row as f32 - 0.5).max(0.);
        let chord = (radius * radius - dy * dy).sqrt();
        // Count pixel centres strictly outside the rounded corner.
        (radius - chord - 0.5).ceil() as i32
    };
    let mut top = 0;
    loop {
        let cut = cut_at(top);
        if cut <= 0 {
            break;
        }
        let mut bottom = top + 1;
        while cut_at(bottom) == cut {
            bottom += 1;
        }
        for (left, right) in [(0, cut), (width - cut, width)] {
            band(left, top, right, bottom);
            band(left, height - bottom, right, height - top);
        }
        top = bottom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kit_retains_both_shadow_primitives_in_every_exterior_band() {
        let viewport = gpui_kit::size(px(1280.), px(800.));
        let material = theme::material(gpui_kit::component::ThemeMode::Dark, viewport);
        let frame = Bounds::new(point(px(0.), px(0.)), gpui_kit::size(px(1240.), px(760.)));
        exterior_bands(1240, 760, theme::SHEET_RADIUS, 20, |l, t, r, b| {
            let clip = Bounds::from_corners(
                point(px(l as f32), px(t as f32)),
                point(px(r as f32), px(b as f32)),
            );
            for shadow in &material.shadow {
                // GPUI's scene builder rejects a clipped shadow if its unblurred
                // source misses the mask; the shader never sees its Gaussian tail.
                let source = (frame + shadow.offset).dilate(shadow.spread_radius);
                assert!(!source.intersect(&clip).is_empty(), "culled at {clip:?}");
            }
        });
    }

    #[test]
    fn clips_both_straight_edges_and_round_corners_once_per_outside_pixel() {
        // Check the four quadrants against the independent rounded-rectangle
        // distance at every device pixel, including a fractional-DPI radius.
        for radius in [3., 3.75, 5.5] {
            let (width, height, margin) = (28, 19, 4);
            let mut bands = Vec::new();
            exterior_bands(width, height, radius, margin, |l, t, r, b| {
                bands.push((l, t, r, b));
            });
            for y in -margin..height + margin {
                for x in -margin..width + margin {
                    let dx = ((x as f32 + 0.5 - width as f32 / 2.).abs()
                        - (width as f32 / 2. - radius))
                        .max(0.);
                    let dy = ((y as f32 + 0.5 - height as f32 / 2.).abs()
                        - (height as f32 / 2. - radius))
                        .max(0.);
                    let outside = dx.hypot(dy) > radius;
                    let paints = bands
                        .iter()
                        .filter(|&&(l, t, r, b)| (l..r).contains(&x) && (t..b).contains(&y))
                        .count();
                    assert_eq!(paints, usize::from(outside), "({x}, {y}) radius {radius}");
                }
            }
        }
    }
}
