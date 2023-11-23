//! Sweep objects along a path to create new objects
//!
//! Sweeps 1D or 2D objects along a straight path, creating a 2D or 3D object,
//! respectively.

mod face;
mod half_edge;
mod path;
mod sketch;
mod vertex;

pub use self::{
    face::SweepFace, half_edge::SweepHalfEdge, path::SweepSurfacePath,
    sketch::SweepSketch, vertex::SweepVertex,
};

use std::collections::BTreeMap;

use crate::{
    objects::{Curve, Vertex},
    storage::{Handle, ObjectId},
};

/// A cache used for sweeping
#[derive(Default)]
pub struct SweepCache {
    /// Cache for curves
    pub curves: BTreeMap<ObjectId, Handle<Curve>>,

    /// Cache for vertices
    pub vertices: BTreeMap<ObjectId, Handle<Vertex>>,
}
