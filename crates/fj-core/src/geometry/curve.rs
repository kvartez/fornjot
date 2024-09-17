use std::{collections::BTreeMap, sync::Arc};

use fj_math::{Circle, Line, Point};

use crate::{storage::Handle, topology::Surface};

use super::{
    curves::circle::CircleApproxParams, CurveBoundary, Path, Tolerance,
};

/// The geometric definition of a curve
#[derive(Clone, Debug, Default)]
pub struct CurveGeom {
    /// # The redundant local definitions of the curve geometry
    ///
    /// ## Implementation Note
    ///
    /// Having multiple redundant definitions is undesirable. However, we can't
    /// just use one global definition in 3D, as we need the local 2D
    /// definitions to triangulate faces, and we currently don't have the tools
    /// to project a global definition into a local context.
    ///
    /// Eventually, it should be possible to define the geometry of a curve
    /// once, either locally or globally, and then convert that single
    /// definition into (other) local contexts, as needed. There currently is no
    /// issue to track that specifically, but there is the following issue,
    /// which is a prerequisite for making the required tooling practical:
    ///
    /// <https://github.com/hannobraun/fornjot/issues/2118>
    pub definitions: BTreeMap<Handle<Surface>, LocalCurveGeom>,
}

impl CurveGeom {
    /// # Return the local definition on the provided surface
    pub fn local_on(
        &self,
        surface: &Handle<Surface>,
    ) -> Option<&LocalCurveGeom> {
        self.definitions.get(surface)
    }
}

/// The geometric definition of a curve, in 2D surface coordinates
#[derive(Clone, Debug)]
pub struct LocalCurveGeom {
    /// The path that defines the curve on its surface
    pub path: Path<2>,
}

/// # The geometric definition of a curve
///
/// Curves are represented by polylines, their uniform intermediate
/// representation. However, this representation can be 2D (local to a surface)
/// or 3D. This enum distinguishes between the two cases.
///
/// ## Implementation Note
///
/// The name, `CurveGeom2`, is a placeholder. As of this writing, there is an
/// ongoing transition to a new geometry system, and the name `CurveGeom` is
/// still taken by an old-style type.
#[derive(Clone)]
pub enum CurveGeom2 {
    /// # The curve is defined locally on a surface
    Surface {
        /// # The geometric representation of the curve
        geometry: Arc<dyn GenPolyline<2>>,

        /// # The surface that the curve geometry is defined on
        surface: Handle<Surface>,
    },

    /// # The curve is defined globally in 3D space
    Global {
        /// # The geometric representation of the curve
        geometry: Arc<dyn GenPolyline<3>>,
    },
}

/// # Generate polylines, the uniform representation of curve geometry
///
/// This trait provides a generic and uniform interface to curve geometry. It is
/// implemented by types that represent specific kinds of curve geometry.
///
/// It is generic over the dimensionality of the generated polyline. Typically,
/// two variants should be implemented per curve geometry type:
///
/// - `CurveGeom2<2>` for surface-local geometry.
/// - `CurveGeom2<3>` for global 3D geometry.
///
///
/// ## Determinism
///
/// For a given curve and a given tolerance, the uniform representation of a
/// curve must be deterministic. This means that the same representation must be
/// returned, regardless of which points on the curve are queried, and in what
/// order.
pub trait GenPolyline<const D: usize> {
    /// # Access the origin of the curve
    fn origin(&self) -> Point<D>;

    /// # Compute a line segment to approximate the curve at this point
    ///
    /// ## Degenerate Case
    ///
    /// If the curve requires no approximation (meaning it is a line), then per
    /// convention, a degenerate line segment is returned, that collapses to the
    /// provided point.
    fn line_segment_at(
        &self,
        point: Point<1>,
        tolerance: Tolerance,
    ) -> [Point<D>; 2];

    /// # Generate a polyline within the provided boundary
    fn generate_polyline(
        &self,
        boundary: CurveBoundary<Point<1>>,
        tolerance: Tolerance,
    ) -> Vec<Point<1>>;
}

impl<const D: usize> GenPolyline<D> for Circle<D> {
    fn origin(&self) -> Point<D> {
        self.center() + self.a()
    }

    fn line_segment_at(
        &self,
        point: Point<1>,
        tolerance: Tolerance,
    ) -> [Point<D>; 2] {
        let params = CircleApproxParams::new(self, tolerance);

        // The approximation parameters have an increment, in curve coordinates,
        // that determines the distance between points on the polyline. Let's
        // figure out where `point` is on the curve, in units of this increment.
        let t = point.t / params.increment();

        // Now pick two points on the curve, again in units of approximation
        // increment, where the locations of the two closest approximation
        // points to the provided point are.
        //
        // Since we are calculating this in increment units, those are integer
        // numbers.
        let a = t.floor();
        let b = t.ceil();

        // Next, convert them into actual curve coordinates.
        let points_curve = [a, b].map(|point_curve_in_increment_units| {
            point_curve_in_increment_units * params.increment()
        });

        // And finally, convert those into points of the desired dimensionality.
        points_curve
            .map(|point_curve| self.point_from_circle_coords([point_curve]))
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

impl<const D: usize> GenPolyline<D> for Line<D> {
    fn origin(&self) -> Point<D> {
        self.origin()
    }

    fn line_segment_at(&self, point: Point<1>, _: Tolerance) -> [Point<D>; 2] {
        // Collapse line segment into a point, as per documentation.
        let point = self.origin() + self.direction() * point.t;

        [point, point]
    }

    fn generate_polyline(
        &self,
        boundary: CurveBoundary<Point<1>>,
        _: Tolerance,
    ) -> Vec<Point<1>> {
        boundary.inner.into()
    }
}

// This implementation is temporary, to ease the transition towards a curve
// geometry trait. Eventually, `CurveGeom2` is expected to replace `Path`.
impl<const D: usize> GenPolyline<D> for Path<D> {
    fn origin(&self) -> Point<D> {
        match self {
            Self::Circle(circle) => circle.origin(),
            Self::Line(line) => line.origin(),
        }
    }

    fn line_segment_at(
        &self,
        point: Point<1>,
        tolerance: Tolerance,
    ) -> [Point<D>; 2] {
        match self {
            Self::Circle(circle) => circle.line_segment_at(point, tolerance),
            Self::Line(line) => line.line_segment_at(point, tolerance),
        }
    }

    fn generate_polyline(
        &self,
        boundary: CurveBoundary<Point<1>>,
        tolerance: Tolerance,
    ) -> Vec<Point<1>> {
        match self {
            Self::Circle(circle) => {
                circle.generate_polyline(boundary, tolerance)
            }
            Self::Line(line) => line.generate_polyline(boundary, tolerance),
        }
    }
}

#[cfg(test)]
mod tests {
    use fj_math::{Circle, Point};

    use crate::geometry::Tolerance;

    use super::GenPolyline;

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
