// @reference: https://d3js.org/d3-shape/arc

use std::{f32::consts::PI, fmt::Debug};

use gpui::{point, px, Bounds, Hsla, Path, PathBuilder, Pixels, Point, Window};

const EPSILON: f32 = 1e-12;
const HALF_PI: f32 = PI / 2.;

pub struct ArcData<'a, T> {
    pub data: &'a T,
    pub index: usize,
    pub value: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub pad_angle: f32,
}

impl<T> Debug for ArcData<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ArcData {{ index: {}, value: {}, start_angle: {}, end_angle: {}, pad_angle: {} }}",
            self.index, self.value, self.start_angle, self.end_angle, self.pad_angle
        )
    }
}

pub struct Arc {
    inner_radius: f32,
    outer_radius: f32,
}

impl Default for Arc {
    fn default() -> Self {
        Self {
            inner_radius: 0.,
            outer_radius: 0.,
        }
    }
}

impl Arc {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the inner radius of the Arc.
    pub fn inner_radius(mut self, inner_radius: f32) -> Self {
        self.inner_radius = inner_radius;
        self
    }

    /// Set the outer radius of the Arc.
    pub fn outer_radius(mut self, outer_radius: f32) -> Self {
        self.outer_radius = outer_radius;
        self
    }

    /// Get the centroid of the Arc.
    pub fn centroid<T>(&self, arc: &ArcData<T>) -> Point<f32> {
        let start_angle = arc.start_angle - HALF_PI;
        let end_angle = arc.end_angle - HALF_PI;
        let r = (self.inner_radius + self.outer_radius) / 2.;
        let a = (start_angle + end_angle) / 2.;

        point(r * a.cos(), r * a.sin())
    }

    fn path<T>(&self, arc: &ArcData<T>, bounds: &Bounds<Pixels>) -> Option<Path<Pixels>> {
        let start_angle = arc.start_angle - HALF_PI;
        let end_angle = arc.end_angle - HALF_PI;
        let da = end_angle - start_angle;
        let pad_angle = if da >= PI {
            // Leave some pad angle for full circle.
            // If not, the path start and end will be the same point.
            0.0001
        } else {
            arc.pad_angle
        };
        let r0 = self.inner_radius.max(0.);
        let r1 = self.outer_radius.max(0.);

        // Calculate the center point.
        let center_x = bounds.origin.x.0 + bounds.size.width.0 / 2.;
        let center_y = bounds.origin.y.0 + bounds.size.height.0 / 2.;

        // Angle difference.
        if r1 < EPSILON || da.abs() < EPSILON {
            return None;
        }

        // Handle pad angle.
        let (a0_outer, a1_outer, a0_inner, a1_inner) = if r0 > EPSILON && pad_angle > 0.0 {
            let pad_width = r1 * pad_angle;
            let pad_angle_outer = pad_width / r1;
            let mut pad_angle_inner = pad_width / r0;
            let max_inner_pad = da * 0.8;
            if pad_angle_inner > max_inner_pad {
                pad_angle_inner = max_inner_pad;
            }
            (
                start_angle + pad_angle_outer * 0.5,
                end_angle - pad_angle_outer * 0.5,
                start_angle + pad_angle_inner * 0.5,
                end_angle - pad_angle_inner * 0.5,
            )
        } else {
            let pad = pad_angle * 0.5;
            (
                start_angle + pad,
                end_angle - pad,
                start_angle + pad,
                end_angle - pad,
            )
        };

        let da_outer = a1_outer - a0_outer;
        if da_outer <= 0. {
            return None;
        }

        // Calculate the start and end points of the outer arc.
        let x01 = center_x + r1 * a0_outer.cos();
        let y01 = center_y + r1 * a0_outer.sin();
        let x11 = center_x + r1 * a1_outer.cos();
        let y11 = center_y + r1 * a1_outer.sin();

        let mut builder = PathBuilder::fill();

        // Move to the start point of the outer arc.
        builder.move_to(point(px(x01), px(y01)));

        // Draw the outer arc.
        let large_arc = (a1_outer - a0_outer).abs() > PI;
        builder.arc_to(
            point(px(r1), px(r1)),
            px(0.),
            large_arc,
            true,
            point(px(x11), px(y11)),
        );

        if r0 > EPSILON {
            // End point of the inner arc.
            let x10 = center_x + r0 * a1_inner.cos();
            let y10 = center_y + r0 * a1_inner.sin();
            builder.line_to(point(px(x10), px(y10)));

            // Draw the inner arc.
            let x00 = center_x + r0 * a0_inner.cos();
            let y00 = center_y + r0 * a0_inner.sin();
            let large_arc_inner = (a1_inner - a0_inner).abs() > PI;
            builder.arc_to(
                point(px(r0), px(r0)),
                px(0.),
                large_arc_inner,
                false,
                point(px(x00), px(y00)),
            );
        } else {
            // If there is no inner radius, draw a line to the center.
            builder.line_to(point(px(center_x), px(center_y)));
        }

        builder.build().ok()
    }

    /// Paint the Arc.
    pub fn paint<T>(
        &self,
        arc: &ArcData<T>,
        color: impl Into<Hsla>,
        bounds: &Bounds<Pixels>,
        window: &mut Window,
    ) {
        let path = self.path(arc, bounds);
        if let Some(path) = path {
            window.paint_path(path, color.into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_default() {
        let arc = Arc::default();
        assert_eq!(arc.inner_radius, 0.);
        assert_eq!(arc.outer_radius, 0.);
    }

    #[test]
    fn test_arc_builder() {
        let arc = Arc::new().inner_radius(10.).outer_radius(20.);

        assert_eq!(arc.inner_radius, 10.);
        assert_eq!(arc.outer_radius, 20.);
    }

    #[test]
    fn test_arc_centroid() {
        let arc = Arc::new().inner_radius(10.).outer_radius(20.);

        let arc_data = ArcData {
            data: &(),
            index: 0,
            value: 1.,
            start_angle: 0.,
            end_angle: PI,
            pad_angle: 0.,
        };

        let centroid = arc.centroid(&arc_data);
        let expected_radius = (10. + 20.) / 2.;
        let expected_angle = (0. + PI - 2. * HALF_PI) / 2.;

        assert_eq!(centroid.x, expected_radius * expected_angle.cos());
        assert_eq!(centroid.y, expected_radius * expected_angle.sin());
    }
}
