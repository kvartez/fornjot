//! # Geometry code specific to circles

use std::iter;

use fj_math::{Circle, LineSegment, Point, Scalar, Sign};

use crate::geometry::{traits::GenPolyline, CurveBoundary, Tolerance};

impl<const D: usize> GenPolyline<D> for Circle<D> {
    fn origin(&self) -> Point<D> {
        self.center() + self.a()
    }

    fn line_segment_at(
        &self,
        point_curve: Point<1>,
        tolerance: Tolerance,
    ) -> LineSegment<D> {
        let params = CircleApproxParams::new(self, tolerance);

        // The approximation parameters have an increment, in curve coordinates,
        // that determines the distance between points on the polyline. Let's
        // figure out where `point` is on the curve, in units of this increment.
        let t = point_curve.t / params.increment();

        // Now pick two points on the curve, again in units of approximation
        // increment, where the locations of the two closest approximation
        // points to the provided point are.
        //
        // Since we are calculating this in increment units, those are integer
        // numbers.
        let a = t.floor();
        let b = t.ceil();

        // Next, convert them into actual curve coordinates.
        let points_curve = [a, b]
            .map(|point_curve_in_increment_units| {
                [point_curve_in_increment_units * params.increment()]
            })
            .map(Point::from);

        // And finally, convert those into points of the desired dimensionality.
        let points = points_curve
            .map(|point_curve| self.point_from_circle_coords(point_curve));

        LineSegment {
            points,
            points_line: points_curve,
        }
    }

    fn generate_polyline(
        &self,
        boundary: CurveBoundary<Point<1>>,
        tolerance: Tolerance,
    ) -> Vec<Point<1>> {
        let params = CircleApproxParams::new(self, tolerance);
        params.approx_circle(boundary).collect()
    }
}

/// Path approximation parameters for a circle
#[derive(Debug)]
struct CircleApproxParams {
    increment: Scalar,
}

impl CircleApproxParams {
    /// Compute path approximation parameters for the given circle and tolerance
    pub fn new<const D: usize>(
        circle: &Circle<D>,
        tolerance: impl Into<Tolerance>,
    ) -> Self {
        let radius = circle.a().magnitude();

        let num_vertices_to_approx_full_circle = Scalar::max(
            Scalar::PI
                / (Scalar::ONE - (tolerance.into().inner() / radius)).acos(),
            3.,
        )
        .ceil();

        let increment = Scalar::TAU / num_vertices_to_approx_full_circle;

        Self { increment }
    }

    /// Return the increment
    pub fn increment(&self) -> Scalar {
        self.increment
    }

    /// Generate points to approximate the circle within the boundary
    pub fn approx_circle(
        &self,
        boundary: impl Into<CurveBoundary<Point<1>>>,
    ) -> impl Iterator<Item = Point<1>> + '_ {
        let boundary = boundary.into();

        let [a, b] = boundary.inner.map(|point| point.t / self.increment());
        let direction = (b - a).sign();
        let [min, max] = if a < b { [a, b] } else { [b, a] };

        // We can't generate a point exactly at the boundaries of the range as
        // part of the approximation. Make sure we stay inside the range.
        let min = min.floor() + 1.;
        let max = max.ceil() - 1.;

        let [start, end] = match direction {
            Sign::Negative => [max, min],
            Sign::Positive | Sign::Zero => [min, max],
        };

        let mut i = start;
        iter::from_fn(move || {
            let is_finished = match direction {
                Sign::Negative => i < end,
                Sign::Positive | Sign::Zero => i > end,
            };

            if is_finished {
                return None;
            }

            let t = self.increment() * i;
            i += direction.to_scalar();

            Some(Point::from([t]))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use fj_math::{Point, Scalar};

    use crate::geometry::{
        curves::circle::Circle, traits::GenPolyline, CurveBoundary, Tolerance,
    };

    use super::CircleApproxParams;

    #[test]
    fn increment_for_circle() {
        test_increment(1., 0.5, 3.);
        test_increment(1., 0.1, 7.);
        test_increment(1., 0.01, 23.);

        fn test_increment(
            radius: impl Into<Scalar>,
            tolerance: impl Into<Tolerance>,
            expected_num_vertices: impl Into<Scalar>,
        ) {
            let circle = Circle::from_center_and_radius([0., 0.], radius);
            let params = CircleApproxParams::new(&circle, tolerance);

            let expected_increment = Scalar::TAU / expected_num_vertices;
            assert_eq!(params.increment(), expected_increment);
        }
    }

    #[test]
    fn points_for_circle() {
        // At the chosen values for radius and tolerance (see below), the
        // increment is `PI / 4`, so ~1.57.

        // Empty range
        let empty: [Scalar; 0] = [];
        test_path([[0.], [0.]], empty);

        // Ranges contain all generated points. Start is before the first
        // increment and after the last one in each case.
        test_path([[0.], [TAU]], [1., 2., 3.]);
        test_path([[1.], [TAU]], [1., 2., 3.]);
        test_path([[0.], [TAU - 1.]], [1., 2., 3.]);

        // Here the range is restricted to cut of the first or last increment.
        test_path([[2.], [TAU]], [2., 3.]);
        test_path([[0.], [TAU - 2.]], [1., 2.]);

        // And everything again, but in reverse.
        test_path([[TAU], [0.]], [3., 2., 1.]);
        test_path([[TAU], [1.]], [3., 2., 1.]);
        test_path([[TAU - 1.], [0.]], [3., 2., 1.]);
        test_path([[TAU], [2.]], [3., 2.]);
        test_path([[TAU - 2.], [0.]], [2., 1.]);

        fn test_path(
            boundary: impl Into<CurveBoundary<Point<1>>>,
            expected_coords: impl IntoIterator<Item = impl Into<Scalar>>,
        ) {
            // Choose radius and tolerance such, that we need 4 vertices to
            // approximate a full circle. This is the lowest number that we can
            // still cover all the edge cases with
            let radius = 1.;
            let tolerance = 0.375;

            let circle = Circle::from_center_and_radius([0., 0.], radius);
            let params = CircleApproxParams::new(&circle, tolerance);

            let points = params.approx_circle(boundary).collect::<Vec<_>>();

            let expected_points = expected_coords
                .into_iter()
                .map(|i| Point::from([params.increment() * i]))
                .collect::<Vec<_>>();
            assert_eq!(points, expected_points);
        }
    }

    #[test]
    fn curve_representation_must_be_deterministic() -> anyhow::Result<()> {
        let circle = Circle::from_center_and_radius([0., 0.], 1.);

        // Deliberately choose a very coarse tolerance, so the circle
        // representation degenerates to a predictable triangle.
        let tolerance = Tolerance::from_scalar(1.)?;

        // Sample the circle at two points that are close together, relative to
        // our tolerance. The intent here is to each time sample the same
        // triangle edge, so also make sure they're not around zero, or another
        // point where two edges are likely to meet.
        //
        // Where those edges meet is implementation-dependent of course, so this
        // test might break if that implementation changes. But I don't think
        // that really matters. We just need to make sure that this test doesn't
        // accidentally hit such a point. Where specifically those points are,
        // doesn't matter.
        let a = circle.line_segment_at(Point::from([0.2]), tolerance);
        let b = circle.line_segment_at(Point::from([0.3]), tolerance);

        assert_eq!(
            a, b,
            "Expecting representation of the curve to be deterministic; it \
            must not depend on the specific points that were sampled.",
        );

        Ok(())
    }
}