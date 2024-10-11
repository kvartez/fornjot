//! The geometry that defines a surface

use fj_math::{Aabb, Point, Scalar, Transform, Triangle, Vector};

use super::{
    traits::{GenPolyline, GenTriMesh},
    util::tri_mesh::convert_vector_surface_to_global,
    Path, Tolerance,
};

/// The geometry that defines a surface
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SweptCurve {
    /// The u-axis of the surface
    pub u: Path<3>,

    /// The v-axis of the surface
    pub v: Vector<3>,
}

impl SweptCurve {
    /// Convert a vector in surface coordinates to model coordinates
    pub fn vector_from_surface_coords(
        &self,
        vector: impl Into<Vector<2>>,
        tolerance: impl Into<Tolerance>,
    ) -> Vector<3> {
        convert_vector_surface_to_global(self, vector, tolerance)
    }

    /// Transform the surface geometry
    #[must_use]
    pub fn transform(self, transform: &Transform) -> Self {
        let Self { u, v } = self;

        let u = u.transform(transform);
        let v = transform.transform_vector(&v);
        Self { u, v }
    }
}

impl GenTriMesh for SweptCurve {
    fn origin(&self) -> Point<3> {
        self.u.origin()
    }

    fn triangle_at(
        &self,
        point_surface: Point<2>,
        tolerance: Tolerance,
    ) -> (Triangle<3>, [Scalar; 3]) {
        let [a, b] = self
            .u
            .line_segment_at(Point::from([point_surface.u]), tolerance)
            .points
            .map(|point_global| point_global + self.v * point_surface.v);

        let c = a + (b - a) / 2.;
        let triangle = Triangle::from([a, b, c]);

        let barycentric_coords = [1. / 3.; 3].map(Into::into);
        (triangle, barycentric_coords)
    }

    fn generate_tri_mesh(
        &self,
        boundary: Aabb<2>,
        tolerance: Tolerance,
    ) -> Vec<Point<2>> {
        let boundary_curve = [[boundary.min.u], [boundary.max.u]];
        let points_curve =
            self.u.generate_polyline(boundary_curve.into(), tolerance);

        points_curve
            .iter()
            .copied()
            .map(|point| [point.t, Scalar::ZERO])
            .chain(
                points_curve
                    .iter()
                    .copied()
                    .map(|point| [point.t, self.v.magnitude()]),
            )
            .map(Point::from)
            .collect()
    }
}
