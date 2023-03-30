use fj_interop::ext::ArrayExt;
use fj_math::Point;

use crate::{
    geometry::curve::Curve,
    objects::{GlobalEdge, HalfEdge, Objects, Surface, Vertex},
    operations::Insert,
    services::Service,
    storage::Handle,
};

/// Builder API for [`HalfEdge`]
pub struct HalfEdgeBuilder {
    curve: Curve,
    boundary: [Point<1>; 2],
    start_vertex: Option<Handle<Vertex>>,
    global_form: Option<Handle<GlobalEdge>>,
}

impl HalfEdgeBuilder {
    /// Create an instance of `HalfEdgeBuilder`
    pub fn new(curve: Curve, boundary: [Point<1>; 2]) -> Self {
        Self {
            curve,
            boundary,
            start_vertex: None,
            global_form: None,
        }
    }

    /// Create a line segment
    pub fn line_segment(
        points_surface: [impl Into<Point<2>>; 2],
        boundary: Option<[Point<1>; 2]>,
    ) -> Self {
        let boundary =
            boundary.unwrap_or_else(|| [[0.], [1.]].map(Point::from));
        let curve = Curve::line_from_points_with_coords(
            boundary.zip_ext(points_surface),
        );

        Self::new(curve, boundary)
    }

    /// Create a line segment from global points
    pub fn line_segment_from_global_points(
        points_global: [impl Into<Point<3>>; 2],
        surface: &Surface,
        boundary: Option<[Point<1>; 2]>,
    ) -> Self {
        let points_surface = points_global
            .map(|point| surface.geometry().project_global_point(point));
        Self::line_segment(points_surface, boundary)
    }

    /// Build the half-edge with a specific start vertex
    pub fn with_start_vertex(mut self, start_vertex: Handle<Vertex>) -> Self {
        self.start_vertex = Some(start_vertex);
        self
    }

    /// Build the half-edge with a specific global form
    pub fn with_global_form(mut self, global_form: Handle<GlobalEdge>) -> Self {
        self.global_form = Some(global_form);
        self
    }

    /// Build the half-edge
    pub fn build(self, objects: &mut Service<Objects>) -> HalfEdge {
        HalfEdge::new(
            self.curve,
            self.boundary,
            self.start_vertex
                .unwrap_or_else(|| Vertex::new().insert(objects)),
            self.global_form
                .unwrap_or_else(|| GlobalEdge::new().insert(objects)),
        )
    }
}
