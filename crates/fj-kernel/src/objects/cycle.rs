use fj_interop::ext::SliceExt;
use fj_math::{Scalar, Winding};
use pretty_assertions::assert_eq;

use crate::{path::SurfacePath, storage::Handle};

use super::{HalfEdge, Surface};

/// A cycle of connected half-edges
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Cycle {
    surface: Handle<Surface>,
    half_edges: Vec<Handle<HalfEdge>>,
}

impl Cycle {
    /// Create a new cycle
    ///
    /// # Panics
    ///
    /// Panics, if `half_edges` does not yield at least one half-edge.
    ///
    /// Panic, if the end of each half-edge does not connect to the beginning of
    /// the next one.
    pub fn new(half_edges: impl IntoIterator<Item = Handle<HalfEdge>>) -> Self {
        let half_edges = half_edges.into_iter().collect::<Vec<_>>();

        let surface = match half_edges.first() {
            Some(half_edge) => half_edge.surface().clone(),
            None => panic!("Cycle must contain at least one half-edge"),
        };

        // Verify, that the curves of all edges are defined in the correct
        // surface.
        for edge in &half_edges {
            assert_eq!(
                surface.id(),
                edge.curve().surface().id(),
                "Edges in cycle not defined in same surface"
            );
        }

        if half_edges.len() != 1 {
            // Verify that all edges connect.
            for [a, b] in half_edges.as_slice().array_windows_ext() {
                let [_, prev] = a.vertices();
                let [next, _] = b.vertices();

                assert_eq!(
                    prev.surface_form().id(),
                    next.surface_form().id(),
                    "Edges in cycle do not connect"
                );
            }
        }

        // Verify that the edges form a cycle
        if let Some(first) = half_edges.first() {
            if let Some(last) = half_edges.last() {
                let [first, _] = first.vertices();
                let [_, last] = last.vertices();

                assert_eq!(
                    first.surface_form().id(),
                    last.surface_form().id(),
                    "Edges do not form a cycle"
                );
            }
        }

        Self {
            surface,
            half_edges,
        }
    }

    /// Access the surface that this cycle is in
    pub fn surface(&self) -> &Handle<Surface> {
        &self.surface
    }

    /// Access the half-edges that make up the cycle
    pub fn half_edges(&self) -> impl Iterator<Item = &Handle<HalfEdge>> + '_ {
        self.half_edges.iter()
    }

    /// Indicate the cycle's winding, assuming a right-handed coordinate system
    ///
    /// Please note that this is not *the* winding of the cycle, only one of the
    /// two possible windings, depending on the direction you look at the
    /// surface that the cycle is defined on from.
    pub fn winding(&self) -> Winding {
        // The cycle could be made up of one or two circles. If that is the
        // case, the winding of the cycle is determined by the winding of the
        // first circle.
        if self.half_edges.len() < 3 {
            let first = self
                .half_edges()
                .next()
                .expect("Invalid cycle: expected at least one half-edge");

            let [a, b] = first.vertices();
            let edge_direction_positive = a.position() < b.position();

            let circle = match first.curve().path() {
                SurfacePath::Circle(circle) => circle,
                SurfacePath::Line(_) => unreachable!(
                    "Invalid cycle: less than 3 edges, but not all are circles"
                ),
            };
            let cross_positive = circle.a().cross2d(&circle.b()) > Scalar::ZERO;

            if edge_direction_positive == cross_positive {
                return Winding::Ccw;
            } else {
                return Winding::Cw;
            }
        }

        // Now that we got the special case out of the way, we can treat the
        // cycle as a polygon:
        // https://stackoverflow.com/a/1165943

        let mut sum = Scalar::ZERO;

        for [a, b] in self.half_edges.as_slice().array_windows_ext() {
            let [a, b] = [a, b].map(|half_edge| {
                let [vertex, _] = half_edge.vertices();
                vertex.surface_form().position()
            });

            sum += (b.u - a.u) * (b.v + a.v);
        }

        if sum > Scalar::ZERO {
            return Winding::Cw;
        }
        if sum < Scalar::ZERO {
            return Winding::Ccw;
        }

        unreachable!("Encountered invalid cycle: {self:#?}");
    }
}
