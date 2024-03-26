use fj_math::{Aabb, Vector};

use crate::{
    geometry::{Geometry, SurfacePath},
    storage::Handle,
    topology::HalfEdge,
};

impl super::BoundingVolume<2> for &Handle<HalfEdge> {
    fn aabb(self, geometry: &Geometry) -> Option<Aabb<2>> {
        let half_edge = self;

        match geometry.of_half_edge(half_edge).path {
            SurfacePath::Circle(circle) => {
                // Just calculate the AABB of the whole circle. This is not the
                // most precise, but it should do for now.

                let center_to_min_max =
                    Vector::from([circle.radius(), circle.radius()]);

                Some(Aabb {
                    min: circle.center() - center_to_min_max,
                    max: circle.center() + center_to_min_max,
                })
            }
            SurfacePath::Line(_) => {
                let geometry = geometry.of_half_edge(half_edge);

                let points = geometry.boundary.inner.map(|point_curve| {
                    geometry.path.point_from_path_coords(point_curve)
                });

                Some(Aabb::<2>::from_points(points))
            }
        }
    }
}
