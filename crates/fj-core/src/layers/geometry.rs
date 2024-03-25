//! Layer infrastructure for [`Geometry`]

use crate::{
    geometry::{CurveGeom, Geometry, HalfEdgeGeom, SurfaceGeom},
    storage::Handle,
    topology::{Curve, HalfEdge, Surface},
};

use super::{Command, Event, Layer};

impl Layer<Geometry> {
    /// Define the geometry of the provided curve
    pub fn define_curve(&mut self, curve: Handle<Curve>, geometry: CurveGeom) {
        let mut events = Vec::new();
        self.process(DefineCurve { curve, geometry }, &mut events);
    }

    /// Define the geometry of the provided half-edge
    pub fn define_half_edge(
        &mut self,
        half_edge: Handle<HalfEdge>,
        geometry: HalfEdgeGeom,
    ) {
        let mut events = Vec::new();
        self.process(
            DefineHalfEdge {
                half_edge,
                geometry,
            },
            &mut events,
        );
    }

    /// # Define the geometry of the provided surface
    ///
    /// ## Panics
    ///
    /// Panics, if the surface is a special pre-defined plane, like the basis
    /// planes (xy-, xz-, or yz-plane).
    pub fn define_surface(
        &mut self,
        surface: Handle<Surface>,
        geometry: SurfaceGeom,
    ) {
        let mut events = Vec::new();
        self.process(DefineSurface { surface, geometry }, &mut events);
    }
}

/// Define the geometry of a curve
pub struct DefineCurve {
    curve: Handle<Curve>,
    geometry: CurveGeom,
}

impl Command<Geometry> for DefineCurve {
    type Result = ();
    type Event = Self;

    fn decide(
        self,
        _: &Geometry,
        events: &mut Vec<Self::Event>,
    ) -> Self::Result {
        events.push(self);
    }
}

impl Event<Geometry> for DefineCurve {
    fn evolve(&self, state: &mut Geometry) {
        state.define_curve_inner(self.curve.clone(), self.geometry.clone());
    }
}

/// Define the geometry of a half-edge
pub struct DefineHalfEdge {
    half_edge: Handle<HalfEdge>,
    geometry: HalfEdgeGeom,
}

impl Command<Geometry> for DefineHalfEdge {
    type Result = ();
    type Event = Self;

    fn decide(
        self,
        _: &Geometry,
        events: &mut Vec<Self::Event>,
    ) -> Self::Result {
        events.push(self);
    }
}

impl Event<Geometry> for DefineHalfEdge {
    fn evolve(&self, state: &mut Geometry) {
        state.define_half_edge_inner(self.half_edge.clone(), self.geometry);
    }
}

/// Define the geometry of a surface
pub struct DefineSurface {
    surface: Handle<Surface>,
    geometry: SurfaceGeom,
}

impl Command<Geometry> for DefineSurface {
    type Result = ();
    type Event = Self;

    fn decide(
        self,
        _: &Geometry,
        events: &mut Vec<Self::Event>,
    ) -> Self::Result {
        events.push(self);
    }
}

impl Event<Geometry> for DefineSurface {
    fn evolve(&self, state: &mut Geometry) {
        state.define_surface_inner(self.surface.clone(), self.geometry);
    }
}
