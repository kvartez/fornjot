//! # Geometric utility code based on triangle meshes

use fj_math::{Point, Vector};

use crate::geometry::{traits::GenTriMesh, Tolerance};

/// # Convert a point in surface coordinates to global coordinates
pub fn convert_point_surface_to_global(
    surface: &dyn GenTriMesh,
    point: impl Into<Point<2>>,
    tolerance: impl Into<Tolerance>,
) -> Point<3> {
    let (triangle, barycentric_coords) =
        surface.triangle_at(point.into(), tolerance.into());
    triangle.point_from_barycentric_coords(barycentric_coords)
}

/// # Convert a vector in surface coordinates to global coordinates
pub fn convert_vector_surface_to_global(
    surface: &dyn GenTriMesh,
    vector: impl Into<Vector<2>>,
    tolerance: impl Into<Tolerance>,
) -> Vector<3> {
    let vector = vector.into();
    let point = convert_point_surface_to_global(
        surface,
        Point { coords: vector },
        tolerance,
    );
    point - surface.origin()
}
